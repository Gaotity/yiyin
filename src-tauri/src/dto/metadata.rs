use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use yiyin_domain::{BuiltInField, Metadata};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct MetadataDto {
    pub fields: BTreeMap<String, String>,
}

impl From<&Metadata> for MetadataDto {
    fn from(metadata: &Metadata) -> Self {
        Self {
            fields: BuiltInField::ALL
                .into_iter()
                .filter_map(|field| {
                    metadata
                        .value(field)
                        .map(|value| (field.key().to_owned(), value.to_owned()))
                })
                .collect(),
        }
    }
}

impl From<Metadata> for MetadataDto {
    fn from(metadata: Metadata) -> Self {
        Self::from(&metadata)
    }
}
