use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BuiltInField {
    PersonalSign,
    Make,
    Model,
    LensMake,
    LensModel,
    ExposureTime,
    FNumber,
    Iso,
    FocalLength,
    FocalLengthIn35mmFormat,
    ExposureProgram,
    DateTimeOriginal,
    ExposureCompensation,
    MeteringMode,
    WhiteBalance,
}

impl BuiltInField {
    pub const ALL: [Self; 15] = [
        Self::PersonalSign,
        Self::Make,
        Self::Model,
        Self::LensMake,
        Self::LensModel,
        Self::ExposureTime,
        Self::FNumber,
        Self::Iso,
        Self::FocalLength,
        Self::FocalLengthIn35mmFormat,
        Self::ExposureProgram,
        Self::DateTimeOriginal,
        Self::ExposureCompensation,
        Self::MeteringMode,
        Self::WhiteBalance,
    ];

    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::PersonalSign => "PersonalSign",
            Self::Make => "Make",
            Self::Model => "Model",
            Self::LensMake => "LensMake",
            Self::LensModel => "LensModel",
            Self::ExposureTime => "ExposureTime",
            Self::FNumber => "FNumber",
            Self::Iso => "ISO",
            Self::FocalLength => "FocalLength",
            Self::FocalLengthIn35mmFormat => "FocalLengthIn35mmFormat",
            Self::ExposureProgram => "ExposureProgram",
            Self::DateTimeOriginal => "DateTimeOriginal",
            Self::ExposureCompensation => "ExposureCompensation",
            Self::MeteringMode => "MeteringMode",
            Self::WhiteBalance => "WhiteBalance",
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PersonalSign => "个性签名",
            Self::Make => "Logo",
            Self::Model => "型号",
            Self::LensMake => "镜头Logo",
            Self::LensModel => "镜头型号",
            Self::ExposureTime => "快门",
            Self::FNumber => "光圈",
            Self::Iso => "ISO",
            Self::FocalLength => "焦距",
            Self::FocalLengthIn35mmFormat => "等效焦距",
            Self::ExposureProgram => "档位",
            Self::DateTimeOriginal => "拍摄日期",
            Self::ExposureCompensation => "曝光补偿",
            Self::MeteringMode => "测光模式",
            Self::WhiteBalance => "白平衡",
        }
    }

    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|field| field.key() == key)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Metadata {
    values: BTreeMap<BuiltInField, String>,
}

impl Metadata {
    pub fn set(&mut self, field: BuiltInField, value: impl Into<String>) {
        let value = value.into();
        if value.trim().is_empty() {
            self.values.remove(&field);
        } else {
            self.values.insert(field, value);
        }
    }

    #[must_use]
    pub fn value(&self, field: BuiltInField) -> Option<&str> {
        self.values.get(&field).map(String::as_str)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{BuiltInField, Metadata};

    #[test]
    fn normalized_metadata_omits_empty_values() {
        let mut metadata = Metadata::default();
        metadata.set(BuiltInField::Make, "   ");
        assert!(metadata.is_empty());

        metadata.set(BuiltInField::Make, "Nikon");
        assert_eq!(metadata.value(BuiltInField::Make), Some("Nikon"));
        assert!(!metadata.is_empty());
    }
}
