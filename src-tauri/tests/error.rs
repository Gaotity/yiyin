use yiyin_application::ApplicationError;
use yiyin_desktop::dto::CommandErrorDto;

#[test]
fn command_errors_preserve_codes_and_redact_internal_sources() {
    let error = ApplicationError::internal(
        "failed to open /Users/private/secret.jpg with platform details",
    );
    let dto = CommandErrorDto::from(error);
    let serialized = serde_json::to_string(&dto).expect("serialize command error");

    assert_eq!(dto.code, "INTERNAL");
    assert_eq!(dto.message, "An internal error occurred.");
    assert!(!serialized.contains("/Users/private"));
    assert!(!serialized.contains("secret.jpg"));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&serialized)
            .expect("error JSON")
            .as_object()
            .expect("error object")
            .len(),
        2
    );
}

#[test]
fn every_application_error_code_maps_one_to_one() {
    use yiyin_application::ErrorCode;

    for code in [
        ErrorCode::Cancelled,
        ErrorCode::ConfigInvalid,
        ErrorCode::FileInvalid,
        ErrorCode::FileNotFound,
        ErrorCode::Forbidden,
        ErrorCode::Internal,
        ErrorCode::InvalidRequest,
        ErrorCode::ResourceNotFound,
        ErrorCode::TaskNotFound,
    ] {
        let dto = CommandErrorDto::from(ApplicationError::new(code, "Safe message."));
        assert_eq!(dto.code, code.as_str());
        assert_eq!(dto.message, "Safe message.");
    }
}
