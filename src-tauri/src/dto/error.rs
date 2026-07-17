use serde::{Deserialize, Serialize};
use yiyin_application::ApplicationError;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct CommandErrorDto {
    pub code: String,
    pub message: String,
}

impl From<ApplicationError> for CommandErrorDto {
    fn from(error: ApplicationError) -> Self {
        Self {
            code: error.code().as_str().to_owned(),
            message: error.safe_message().to_owned(),
        }
    }
}

impl From<&ApplicationError> for CommandErrorDto {
    fn from(error: &ApplicationError) -> Self {
        Self {
            code: error.code().as_str().to_owned(),
            message: error.safe_message().to_owned(),
        }
    }
}
