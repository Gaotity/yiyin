#[path = "../src/dto/mod.rs"]
mod dto_under_test;

#[test]
fn generated_types_are_current_and_have_no_private_path_fields() {
    let generated = dto_under_test::generated_types();
    let checked_in = include_str!("../../src/platform/types.ts");
    assert_eq!(generated, checked_in);
    for forbidden in [
        "path:",
        "dir:",
        "output:",
        "cacheDir:",
        "cache_dir:",
        "staticDir:",
        "static_dir:",
    ] {
        assert!(!generated.contains(forbidden), "private field {forbidden}");
    }
    assert!(generated.contains("export type BootstrapDto"));
    assert!(generated.contains("export type TaskStatusEventDto"));
    assert!(generated.contains("export type CommandErrorDto"));
}

#[test]
fn generated_types_do_not_expose_rust_owned_dead_fields() {
    let generated = dto_under_test::generated_types();
    for forbidden in ["originalDimensions", "original_dimensions"] {
        assert!(
            !generated.contains(forbidden),
            "rust-owned field {forbidden} must not cross the DTO"
        );
    }
}

#[test]
fn public_config_round_trip_preserves_rust_owned_output_and_system_invariants() {
    let current = yiyin_domain::Config::default();
    let mut dto = dto_under_test::PublicConfigDto::from(&current);
    dto.options.quality = 82;
    let mapped = dto.apply_to(&current).expect("valid public configuration");

    assert_eq!(mapped.output, current.output);
    assert_eq!(mapped.options.quality.get(), 82);
    assert_eq!(mapped.temp_fields.len(), current.temp_fields.len());
    assert_eq!(mapped.templates.len(), current.templates.len());

    let mut invalid = dto_under_test::PublicConfigDto::from(&current);
    invalid.template_fields.pop();
    assert_eq!(
        invalid
            .apply_to(&current)
            .expect_err("missing system field")
            .code(),
        yiyin_application::ErrorCode::ConfigInvalid
    );
}

#[test]
fn bootstrap_dto_serializes_numeric_constraints_with_camel_case_keys() {
    let snapshot = yiyin_application::BootstrapSnapshot::new(
        yiyin_domain::Config::default(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let dto = dto_under_test::BootstrapDto::from(&snapshot);
    let serialized = serde_json::to_value(&dto).expect("serialize bootstrap dto");
    let constraints = serialized
        .get("constraints")
        .expect("constraints field");

    assert_eq!(constraints.as_object().expect("object").len(), 7);
    assert_eq!(
        constraints["radius"],
        serde_json::json!({"minimum": 0.0, "maximum": 50.0, "decimals": 1})
    );
    assert_eq!(
        constraints["textMargin"],
        serde_json::json!({"minimum": 0.0, "maximum": 10000.0, "decimals": 2})
    );
    assert_eq!(
        constraints["backgroundBlur"],
        serde_json::json!({"minimum": 0.0, "maximum": 100.0, "decimals": 0})
    );
}

#[test]
fn drag_drop_notification_reuses_the_path_free_task_status_contract() {
    let task = dto_under_test::TaskDescriptorDto {
        id: "opaque-task-id".to_owned(),
        display_name: "input.jpg".to_owned(),
        state: dto_under_test::TaskStateDto::Registered,
        progress: 0,
        preview: false,
        resource: None,
    };

    let event = dto_under_test::TaskStatusEventDto::from(&task);
    let serialized = serde_json::to_string(&event).expect("serialize task status event");

    assert_eq!(event.task_id, "opaque-task-id");
    assert_eq!(event.state, dto_under_test::TaskStateDto::Registered);
    assert_eq!(event.cancellation_reason, None);
    assert!(!serialized.contains("input.jpg"));
    assert!(!serialized.contains('/'));
}
