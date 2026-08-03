use std::{collections::HashMap, sync::Arc};

use cosmic_text::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, Style, SwashCache, Weight, Wrap,
    fontdb,
};
use image::{Rgba, RgbaImage, imageops::FilterType};
use yiyin_application::{ApplicationError, ResourceRepository};
use yiyin_domain::{
    BackgroundKind, FontSpec, RowSlot, TextMeasurement, TextRowPlan, VerticalAlign,
};

use crate::{FileSystem, ResourceRegistry};

pub struct RenderedRow {
    pub image: RgbaImage,
    pub measurement: TextMeasurement,
}

pub struct RasterContext<'a> {
    pub background: BackgroundKind,
    pub background_height: u32,
    pub text_margin_percent: f64,
    pub default_family: &'a str,
    pub bundled_fonts: &'a [Vec<u8>],
    pub resources: &'a ResourceRegistry,
    pub filesystem: &'a dyn FileSystem,
}

pub fn rasterize_rows(
    plans: &[TextRowPlan],
    context: &RasterContext<'_>,
    forced_measurements: &[TextMeasurement],
) -> Result<Vec<RenderedRow>, ApplicationError> {
    if !forced_measurements.is_empty() && forced_measurements.len() != plans.len() {
        return Err(ApplicationError::internal(
            "forced text measurements do not match planned rows",
        ));
    }
    let mut fonts =
        build_font_catalog(context.bundled_fonts, context.resources, context.filesystem)?;
    let mut cache = SwashCache::new();

    let default_color = match context.background {
        BackgroundKind::Light => Rgba([0, 0, 0, 255]),
        BackgroundKind::Dark => Rgba([255, 255, 255, 255]),
    };

    plans
        .iter()
        .enumerate()
        .map(|(index, plan)| {
            let row = rasterize_row(
                plan,
                context.background_height,
                context.text_margin_percent,
                context.default_family,
                default_color,
                context.resources,
                context.filesystem,
                &fonts.aliases,
                &mut fonts.system,
                &mut cache,
            )?;
            if let Some(measurement) = forced_measurements.get(index).copied() {
                let height = checked_dimension(measurement.height().ceil())?;
                let image = image::imageops::resize(
                    &row,
                    measurement.width(),
                    height,
                    FilterType::Lanczos3,
                );
                Ok(RenderedRow { image, measurement })
            } else {
                let measurement = TextMeasurement::new(row.width(), f64::from(row.height()))
                    .map_err(|_| ApplicationError::internal("invalid text measurement"))?;
                Ok(RenderedRow {
                    image: row,
                    measurement,
                })
            }
        })
        .collect()
}

struct FontCatalog {
    system: FontSystem,
    aliases: HashMap<String, String>,
}

fn build_font_catalog(
    bundled_fonts: &[Vec<u8>],
    resources: &ResourceRegistry,
    filesystem: &dyn FileSystem,
) -> Result<FontCatalog, ApplicationError> {
    let mut sources = bundled_fonts
        .iter()
        .cloned()
        .map(|bytes| fontdb::Source::Binary(Arc::new(bytes)))
        .collect::<Vec<_>>();
    let mut aliases = HashMap::from([
        ("春风楷".to_owned(), "Slidechunfeng".to_owned()),
        ("千图小兔".to_owned(), "QTxiaotu".to_owned()),
        (
            "FrederickatheGreat".to_owned(),
            "Fredericka the Great".to_owned(),
        ),
    ]);
    for resource in resources.snapshot() {
        if resource.kind() == yiyin_domain::ResourceKind::Font {
            let bytes = filesystem
                .read(resource.source())
                .map_err(|error| ApplicationError::internal(error.to_string()))?;
            if let Some(family) = embedded_family(&bytes) {
                aliases.insert(resource.display_name().to_owned(), family);
            }
            sources.push(fontdb::Source::Binary(Arc::new(bytes)));
        }
    }
    Ok(FontCatalog {
        system: FontSystem::new_with_fonts(sources),
        aliases,
    })
}

fn embedded_family(bytes: &[u8]) -> Option<String> {
    let mut database = fontdb::Database::new();
    database.load_font_data(bytes.to_vec());
    database
        .faces()
        .next()
        .and_then(|face| face.families.first())
        .map(|(name, _)| name.clone())
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::too_many_arguments,
    reason = "the arguments are explicit rendering context and bounded row geometry is rounded for compositing"
)]
fn rasterize_row(
    plan: &TextRowPlan,
    background_height: u32,
    text_margin_percent: f64,
    default_family: &str,
    default_color: Rgba<u8>,
    resources: &ResourceRegistry,
    filesystem: &dyn FileSystem,
    aliases: &HashMap<String, String>,
    font_system: &mut FontSystem,
    cache: &mut SwashCache,
) -> Result<RgbaImage, ApplicationError> {
    let mut slots = Vec::new();
    for slot in plan.slots() {
        match slot {
            RowSlot::Text { value, font } => slots.push(rasterize_text(
                value,
                font,
                background_height,
                default_family,
                default_color,
                aliases,
                font_system,
                cache,
            )?),
            RowSlot::Image { resource, font } => {
                let record = resources.resolve(resource)?;
                let bytes = filesystem
                    .read(record.source())
                    .map_err(|_| ApplicationError::file_invalid())?;
                let source = image::load_from_memory(&bytes)
                    .map_err(|_| ApplicationError::file_invalid())?
                    .to_rgba8();
                let height = font_pixels(font, background_height).ceil().max(1.0);
                let width = height * f64::from(source.width()) / f64::from(source.height());
                slots.push(image::imageops::resize(
                    &source,
                    checked_dimension(width)?,
                    checked_dimension(height)?,
                    FilterType::Lanczos3,
                ));
            }
        }
    }
    let padding = 30_u32;
    let margin = f64::from(background_height) * (text_margin_percent / 100.0);
    let content_height = slots.iter().map(RgbaImage::height).max().unwrap_or(1);
    let computed_height = checked_dimension(f64::from(content_height) + margin * 2.0)?;
    let height = plan
        .height()
        .map(checked_dimension)
        .transpose()?
        .unwrap_or(computed_height)
        .max(1);
    let width = slots
        .iter()
        .try_fold(padding, |width, slot| width.checked_add(slot.width()))
        .ok_or_else(|| ApplicationError::internal("text row is too wide"))?;
    let mut row = RgbaImage::from_pixel(width, height, Rgba([0, 0, 0, 0]));
    let mut x = i64::from(padding / 2);
    for slot in slots {
        let y = match plan.vertical_align() {
            VerticalAlign::Center => (i64::from(height) - i64::from(slot.height())) / 2,
            VerticalAlign::Baseline => {
                let baseline = f64::from(height) * 0.72;
                (baseline - f64::from(slot.height()) * 0.72).round() as i64
            }
        };
        super::composite::overlay_rgba(&mut row, &slot, x, y);
        x += i64::from(slot.width());
    }
    Ok(row)
}

#[allow(
    clippy::too_many_arguments,
    reason = "the arguments are the explicit deterministic glyph rendering context"
)]
fn rasterize_text(
    value: &str,
    font: &FontSpec,
    background_height: u32,
    default_family: &str,
    default_color: Rgba<u8>,
    aliases: &HashMap<String, String>,
    font_system: &mut FontSystem,
    cache: &mut SwashCache,
) -> Result<RgbaImage, ApplicationError> {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "bounded font pixels are intentionally converted for cosmic-text metrics"
    )]
    let size = font_pixels(font, background_height).max(1.0) as f32;
    let line_height = (size * 1.2).max(1.0);
    let mut buffer = Buffer::new(font_system, Metrics::new(size, line_height));
    buffer.set_size(None, None);
    buffer.set_wrap(Wrap::None);
    let family = if font.family().is_empty() {
        default_family
    } else {
        font.family()
    };
    let family = aliases.get(family).map_or(family, String::as_str);
    let color = parse_color(font.color()).unwrap_or(default_color);
    let mut attrs = Attrs::new()
        .family(Family::Name(family))
        .color(Color::rgba(color[0], color[1], color[2], color[3]));
    if font.bold() {
        attrs = attrs.weight(Weight::BOLD);
    }
    if font.italic() {
        attrs = attrs.style(Style::Italic);
    }
    buffer.set_text(value, &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(font_system, false);
    let width = buffer
        .layout_runs()
        .map(|run| f64::from(run.line_w))
        .fold(0.0_f64, f64::max)
        .ceil()
        .max(1.0);
    let height = f64::from(line_height).ceil().max(1.0);
    let mut image = RgbaImage::from_pixel(
        checked_dimension(width)?,
        checked_dimension(height)?,
        Rgba([0, 0, 0, 0]),
    );
    buffer.draw(
        font_system,
        cache,
        Color::rgba(color[0], color[1], color[2], color[3]),
        |x, y, width, height, glyph_color| {
            let glyph = Rgba(glyph_color.as_rgba());
            for offset_y in 0..height {
                for offset_x in 0..width {
                    let target_x = i64::from(x) + i64::from(offset_x);
                    let target_y = i64::from(y) + i64::from(offset_y);
                    let (Ok(target_x), Ok(target_y)) =
                        (u32::try_from(target_x), u32::try_from(target_y))
                    else {
                        continue;
                    };
                    if target_x < image.width() && target_y < image.height() {
                        blend_pixel(image.get_pixel_mut(target_x, target_y), glyph);
                    }
                }
            }
        },
    );
    Ok(image)
}

fn font_pixels(font: &FontSpec, background_height: u32) -> f64 {
    f64::from(background_height) * (font.size() / 100.0)
}

fn parse_color(value: &str) -> Option<Rgba<u8>> {
    let digits = value.strip_prefix('#')?;
    if digits.len() != 6 {
        return None;
    }
    let channel = |offset| u8::from_str_radix(&digits[offset..offset + 2], 16).ok();
    Some(Rgba([channel(0)?, channel(2)?, channel(4)?, 255]))
}

fn blend_pixel(bottom: &mut Rgba<u8>, top: Rgba<u8>) {
    let alpha = u16::from(top[3]);
    let inverse = 255 - alpha;
    for channel in 0..3 {
        let value = u16::from(top[channel]) * alpha + u16::from(bottom[channel]) * inverse;
        bottom[channel] = u8::try_from((value + 127) / 255).unwrap_or(u8::MAX);
    }
    bottom[3] =
        u8::try_from(alpha + (u16::from(bottom[3]) * inverse + 127) / 255).unwrap_or(u8::MAX);
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "positive finite image dimensions are bounds checked before conversion"
)]
fn checked_dimension(value: f64) -> Result<u32, ApplicationError> {
    if !value.is_finite() || value <= 0.0 || value > f64::from(u32::MAX) {
        Err(ApplicationError::internal("invalid text surface size"))
    } else {
        Ok(value.ceil() as u32)
    }
}

#[cfg(test)]
mod tests {
    use crate::StdFileSystem;

    use super::*;

    #[test]
    fn loads_every_bundled_font_family() {
        let directory = tempfile::tempdir().expect("create resource directory");
        let resources = ResourceRegistry::new(directory.path()).expect("create resources");
        let bundled_fonts = vec![
            include_bytes!("../../../../assets/fonts/春风楷.ttf").to_vec(),
            include_bytes!("../../../../assets/fonts/千图小兔体.ttf").to_vec(),
            include_bytes!("../../../../assets/fonts/FrederickatheGreat.ttf").to_vec(),
            include_bytes!("../../../../assets/fonts/Neoneon.otf").to_vec(),
        ];

        let catalog =
            build_font_catalog(&bundled_fonts, &resources, &StdFileSystem).expect("load fonts");
        let families = catalog
            .system
            .db()
            .faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.as_str()))
            .collect::<Vec<_>>();

        assert!(families.contains(&"Slidechunfeng"));
        assert!(families.contains(&"QTxiaotu"));
        assert!(families.contains(&"Fredericka the Great"), "{families:?}");
        assert!(families.contains(&"Neoneon"), "{families:?}");
        assert_eq!(catalog.aliases["春风楷"], "Slidechunfeng");
        assert_eq!(catalog.aliases["千图小兔"], "QTxiaotu");
        assert_eq!(
            catalog.aliases["FrederickatheGreat"],
            "Fredericka the Great"
        );
    }

    #[test]
    fn maps_a_custom_display_name_to_its_embedded_family() {
        let directory = tempfile::tempdir().expect("create resource directory");
        let source = directory.path().join("custom.otf");
        std::fs::write(
            &source,
            include_bytes!("../../../../assets/fonts/Neoneon.otf"),
        )
        .expect("write custom font");
        let resources =
            ResourceRegistry::new(directory.path().join("owned")).expect("create resources");
        resources
            .register_owned(yiyin_domain::ResourceKind::Font, &source, "My Neon")
            .expect("register custom font");

        let catalog = build_font_catalog(
            &[include_bytes!("../../../../assets/fonts/千图小兔体.ttf").to_vec()],
            &resources,
            &StdFileSystem,
        )
        .expect("load fonts");

        assert_eq!(catalog.aliases["My Neon"], "Neoneon");
    }
}
