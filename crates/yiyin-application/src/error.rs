use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    Cancelled,
    ConfigInvalid,
    FileInvalid,
    FileNotFound,
    Forbidden,
    Internal,
    InvalidRequest,
    ResourceNotFound,
    TaskNotFound,
}

impl ErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cancelled => "CANCELLED",
            Self::ConfigInvalid => "CONFIG_INVALID",
            Self::FileInvalid => "FILE_INVALID",
            Self::FileNotFound => "FILE_NOT_FOUND",
            Self::Forbidden => "FORBIDDEN",
            Self::Internal => "INTERNAL",
            Self::InvalidRequest => "INVALID_REQUEST",
            Self::ResourceNotFound => "RESOURCE_NOT_FOUND",
            Self::TaskNotFound => "TASK_NOT_FOUND",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationError {
    code: ErrorCode,
    safe_message: &'static str,
    internal_source: Option<String>,
}

impl ApplicationError {
    #[must_use]
    pub const fn new(code: ErrorCode, safe_message: &'static str) -> Self {
        Self {
            code,
            safe_message,
            internal_source: None,
        }
    }

    #[must_use]
    pub fn internal(source: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Internal,
            safe_message: "An internal error occurred.",
            internal_source: Some(source.into()),
        }
    }

    #[must_use]
    pub const fn cancelled() -> Self {
        Self::new(ErrorCode::Cancelled, "The operation was cancelled.")
    }

    #[must_use]
    pub const fn config_invalid() -> Self {
        Self::new(ErrorCode::ConfigInvalid, "The configuration is invalid.")
    }

    #[must_use]
    pub const fn file_invalid() -> Self {
        Self::new(ErrorCode::FileInvalid, "The selected file is invalid.")
    }

    #[must_use]
    pub const fn file_not_found() -> Self {
        Self::new(ErrorCode::FileNotFound, "The selected file was not found.")
    }

    #[must_use]
    pub const fn forbidden() -> Self {
        Self::new(ErrorCode::Forbidden, "The operation is not allowed.")
    }

    #[must_use]
    pub const fn invalid_request(message: &'static str) -> Self {
        Self::new(ErrorCode::InvalidRequest, message)
    }

    #[must_use]
    pub const fn resource_not_found() -> Self {
        Self::new(ErrorCode::ResourceNotFound, "The resource was not found.")
    }

    #[must_use]
    pub const fn task_not_found() -> Self {
        Self::new(ErrorCode::TaskNotFound, "The task was not found.")
    }

    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    #[must_use]
    pub const fn safe_message(&self) -> &'static str {
        self.safe_message
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.safe_message)
    }
}

impl std::error::Error for ApplicationError {}
