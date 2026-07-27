use serde::{Deserialize, Serialize};
use yiyin_application::ResourceSnapshot;
use yiyin_domain::ResourceKind;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub enum ResourceKindDto {
    BundledAsset,
    Font,
    Overlay,
    Input,
    Preview,
    Output,
}

impl From<ResourceKind> for ResourceKindDto {
    fn from(kind: ResourceKind) -> Self {
        match kind {
            ResourceKind::BundledAsset => Self::BundledAsset,
            ResourceKind::Font => Self::Font,
            ResourceKind::Overlay => Self::Overlay,
            ResourceKind::Input => Self::Input,
            ResourceKind::Preview => Self::Preview,
            ResourceKind::Output => Self::Output,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct ResourceDescriptorDto {
    pub id: String,
    pub kind: ResourceKindDto,
    pub display_name: String,
    pub url: String,
}

impl From<&ResourceSnapshot> for ResourceDescriptorDto {
    fn from(resource: &ResourceSnapshot) -> Self {
        Self {
            id: resource.id().as_str().to_owned(),
            kind: resource.kind().into(),
            display_name: resource.display_name().to_owned(),
            url: format!("yiyin://resource/{}", resource.id().as_str()),
        }
    }
}

impl From<ResourceSnapshot> for ResourceDescriptorDto {
    fn from(resource: ResourceSnapshot) -> Self {
        Self::from(&resource)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct ResourceIdRequestDto {
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct RegisterFontRequestDto {
    pub name: String,
}
