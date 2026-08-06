use std::{
    fs::File,
    io::{BufReader, ErrorKind},
    path::Path,
};

use exif::{Exif, Field, In, Tag, Value};
use yiyin_application::{ApplicationError, MetadataReader};
use yiyin_domain::{
    BuiltInField, ImageDensity, ImageOrientation, Metadata, decimal, format_shutter,
    normalize_date_time, normalize_exposure_program, normalize_metering_mode,
    normalize_white_balance, rounded_display, signed_decimal,
};

pub struct ExifMetadataReader;

impl MetadataReader for ExifMetadataReader {
    fn read(&self, source: &Path) -> Result<Option<Metadata>, ApplicationError> {
        let file = File::open(source).map_err(map_open_error)?;
        let mut reader = BufReader::new(file);
        let exif = match exif::Reader::new()
            .continue_on_error(true)
            .read_from_container(&mut reader)
        {
            Ok(exif) => exif,
            Err(exif::Error::PartialResult(partial)) => partial.into_inner().0,
            Err(exif::Error::NotFound(_)) => return Ok(None),
            Err(exif::Error::Io(error)) if error.kind() == ErrorKind::NotFound => {
                return Err(ApplicationError::file_not_found());
            }
            Err(exif::Error::Io(error)) => {
                return Err(ApplicationError::internal(error.to_string()));
            }
            Err(exif::Error::InvalidFormat(_)) => {
                if let Some(exif) = read_webp_exif(source)? {
                    exif
                } else if is_supported_image(source) {
                    return Ok(None);
                } else {
                    return Err(ApplicationError::file_invalid());
                }
            }
            Err(_) => return Err(ApplicationError::file_invalid()),
        };

        Ok(normalize_metadata(&exif))
    }
}

fn read_webp_exif(source: &Path) -> Result<Option<Exif>, ApplicationError> {
    let bytes = std::fs::read(source).map_err(map_open_error)?;
    if bytes.len() < 12 || !bytes.starts_with(b"RIFF") || &bytes[8..12] != b"WEBP" {
        return Ok(None);
    }
    let mut offset = 12_usize;
    while offset.checked_add(8).is_some_and(|end| end <= bytes.len()) {
        let chunk_name = &bytes[offset..offset + 4];
        let chunk_size = u32::from_le_bytes(
            bytes[offset + 4..offset + 8]
                .try_into()
                .map_err(|_| ApplicationError::file_invalid())?,
        );
        let chunk_size =
            usize::try_from(chunk_size).map_err(|_| ApplicationError::file_invalid())?;
        let data_start = offset + 8;
        let data_end = data_start
            .checked_add(chunk_size)
            .ok_or_else(ApplicationError::file_invalid)?;
        if data_end > bytes.len() {
            return Err(ApplicationError::file_invalid());
        }
        if chunk_name == b"EXIF" {
            let data = bytes[data_start..data_end]
                .strip_prefix(b"Exif\0\0")
                .unwrap_or(&bytes[data_start..data_end]);
            let result = exif::Reader::new()
                .continue_on_error(true)
                .read_raw(data.to_vec());
            return match result {
                Ok(exif) => Ok(Some(exif)),
                Err(exif::Error::PartialResult(partial)) => Ok(Some(partial.into_inner().0)),
                Err(_) => Ok(None),
            };
        }
        let padded = chunk_size
            .checked_add(chunk_size % 2)
            .ok_or_else(ApplicationError::file_invalid)?;
        offset = data_start
            .checked_add(padded)
            .ok_or_else(ApplicationError::file_invalid)?;
    }
    Ok(None)
}

fn is_supported_image(source: &Path) -> bool {
    let Ok(bytes) = std::fs::read(source) else {
        return false;
    };
    let supported_signature = bytes.starts_with(&[0xff, 0xd8, 0xff])
        || bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || (bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP");
    supported_signature && image::load_from_memory(&bytes).is_ok()
}

fn normalize_metadata(exif: &Exif) -> Option<Metadata> {
    let mut metadata = Metadata::default();
    set_ascii(&mut metadata, BuiltInField::Make, exif, Tag::Make);
    set_ascii(&mut metadata, BuiltInField::Model, exif, Tag::Model);
    set_ascii(&mut metadata, BuiltInField::LensMake, exif, Tag::LensMake);
    set_ascii(&mut metadata, BuiltInField::LensModel, exif, Tag::LensModel);

    if let Some(field) = field(exif, Tag::FNumber) {
        metadata.set(BuiltInField::FNumber, rational_decimal(field));
    }
    if let Some(value) = uint(exif, Tag::PhotographicSensitivity)
        .or_else(|| uint(exif, Tag::ISOSpeed))
        .filter(|value| *value != 0)
    {
        metadata.set(BuiltInField::Iso, value.to_string());
    }
    if let Some(value) = rational(exif, Tag::FocalLength).filter(|value| *value > 0.0) {
        metadata.set(BuiltInField::FocalLength, rounded_display(value));
    }
    let equivalent = uint(exif, Tag::FocalLengthIn35mmFilm)
        .filter(|value| *value != 0)
        .map(f64::from)
        .or_else(|| rational(exif, Tag::FocalLength).filter(|value| *value > 0.0));
    if let Some(value) = equivalent {
        metadata.set(
            BuiltInField::FocalLengthIn35mmFormat,
            rounded_display(value),
        );
    }
    if let Some(Field {
        value: Value::Rational(values),
        ..
    }) = field(exif, Tag::ExposureTime)
        && let Some(value) = values.first()
    {
        metadata.set(
            BuiltInField::ExposureTime,
            format_shutter(value.num, value.denom),
        );
    }
    if let Some(value) = ascii(exif, Tag::DateTimeOriginal) {
        metadata.set(BuiltInField::DateTimeOriginal, normalize_date_time(&value));
    }
    if let Some(value) = signed_rational(exif, Tag::ExposureBiasValue) {
        metadata.set(BuiltInField::ExposureCompensation, signed_decimal(value));
    }
    if let Some(value) = uint(exif, Tag::WhiteBalance) {
        metadata.set(BuiltInField::WhiteBalance, normalize_white_balance(value));
    }
    if let Some(value) = uint(exif, Tag::ExposureProgram) {
        metadata.set(
            BuiltInField::ExposureProgram,
            normalize_exposure_program(value),
        );
    }
    if let Some(value) = uint(exif, Tag::MeteringMode) {
        metadata.set(BuiltInField::MeteringMode, normalize_metering_mode(value));
    }

    metadata.set_orientation(
        uint(exif, Tag::Orientation).and_then(|value| ImageOrientation::try_from(value).ok()),
    );
    metadata.set_density(normalized_density(exif));

    let has_render_metadata =
        !metadata.is_empty() || metadata.orientation().is_some() || metadata.density().is_some();
    has_render_metadata.then_some(metadata)
}

fn field(exif: &Exif, tag: Tag) -> Option<&Field> {
    exif.get_field(tag, In::PRIMARY)
}

fn ascii(exif: &Exif, tag: Tag) -> Option<String> {
    let Field {
        value: Value::Ascii(values),
        ..
    } = field(exif, tag)?
    else {
        return None;
    };
    let value = values.first()?;
    let value = String::from_utf8_lossy(value).trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn set_ascii(metadata: &mut Metadata, target: BuiltInField, exif: &Exif, tag: Tag) {
    if let Some(value) = ascii(exif, tag) {
        metadata.set(target, value);
    }
}

fn uint(exif: &Exif, tag: Tag) -> Option<u32> {
    field(exif, tag)?.value.get_uint(0)
}

fn rational(exif: &Exif, tag: Tag) -> Option<f64> {
    let Field {
        value: Value::Rational(values),
        ..
    } = field(exif, tag)?
    else {
        return None;
    };
    let value = values.first()?;
    (value.denom != 0).then(|| f64::from(value.num) / f64::from(value.denom))
}

fn signed_rational(exif: &Exif, tag: Tag) -> Option<f64> {
    let Field {
        value: Value::SRational(values),
        ..
    } = field(exif, tag)?
    else {
        return None;
    };
    let value = values.first()?;
    (value.denom != 0).then(|| f64::from(value.num) / f64::from(value.denom))
}

fn rational_decimal(field: &Field) -> String {
    let Value::Rational(values) = &field.value else {
        return String::new();
    };
    let Some(value) = values.first() else {
        return String::new();
    };
    if value.denom == 0 || value.num == 0 {
        String::new()
    } else {
        decimal(f64::from(value.num) / f64::from(value.denom))
    }
}

fn normalized_density(exif: &Exif) -> Option<ImageDensity> {
    let resolution = rational(exif, Tag::XResolution)?;
    let pixels_per_inch =
        match field(exif, Tag::ResolutionUnit).and_then(|field| field.value.get_uint(0)) {
            Some(3) => resolution * 2.54,
            Some(2) | None => resolution,
            _ => return None,
        };
    if !pixels_per_inch.is_finite() || pixels_per_inch <= 0.0 {
        return None;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "positive finite EXIF density is rounded after a u32 bounds check"
    )]
    let rounded = if pixels_per_inch > f64::from(u32::MAX) {
        return None;
    } else {
        pixels_per_inch.round() as u32
    };
    ImageDensity::new(rounded).ok()
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the function is passed directly to Result::map_err"
)]
fn map_open_error(error: std::io::Error) -> ApplicationError {
    if error.kind() == ErrorKind::NotFound {
        ApplicationError::file_not_found()
    } else {
        ApplicationError::internal(error.to_string())
    }
}
