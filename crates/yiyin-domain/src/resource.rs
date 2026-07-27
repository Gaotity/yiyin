use crate::DomainError;

macro_rules! opaque_id {
    ($name:ident, $field:literal) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = DomainError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                if value.trim().is_empty() {
                    Err(DomainError::Empty($field))
                } else {
                    Ok(Self(value.to_owned()))
                }
            }
        }

        impl TryFrom<String> for $name {
            type Error = DomainError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                if value.trim().is_empty() {
                    Err(DomainError::Empty($field))
                } else {
                    Ok(Self(value))
                }
            }
        }
    };
}

opaque_id!(ResourceId, "resource_id");
opaque_id!(TaskId, "task_id");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    BundledAsset,
    Font,
    Overlay,
    Input,
    Preview,
    Output,
}

#[cfg(test)]
mod tests {
    use super::{ResourceId, ResourceKind, TaskId};
    use crate::DomainError;

    #[test]
    fn opaque_ids_reject_empty_values_without_reinterpreting_content() {
        assert_eq!(
            ResourceId::try_from(""),
            Err(DomainError::Empty("resource_id"))
        );
        assert_eq!(TaskId::try_from("   "), Err(DomainError::Empty("task_id")));

        let id = ResourceId::try_from("opaque:/not-a-path").expect("non-empty opaque id");
        assert_eq!(id.as_str(), "opaque:/not-a-path");
    }

    #[test]
    fn resource_kinds_are_closed_over_supported_capabilities() {
        let kinds = [
            ResourceKind::BundledAsset,
            ResourceKind::Font,
            ResourceKind::Overlay,
            ResourceKind::Input,
            ResourceKind::Preview,
            ResourceKind::Output,
        ];

        assert_eq!(kinds.len(), 6);
    }
}
