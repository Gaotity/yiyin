use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainError {
    Empty(&'static str),
    IllegalTaskTransition,
    InvalidValue(&'static str),
    OutOfRange(&'static str),
    ProtectedSystemTemplate,
}

impl fmt::Display for DomainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(field) => write!(formatter, "{field} must not be empty"),
            Self::IllegalTaskTransition => formatter.write_str("illegal task transition"),
            Self::InvalidValue(field) => write!(formatter, "{field} is invalid"),
            Self::OutOfRange(field) => write!(formatter, "{field} is out of range"),
            Self::ProtectedSystemTemplate => {
                formatter.write_str("system templates cannot be deleted")
            }
        }
    }
}

impl std::error::Error for DomainError {}
