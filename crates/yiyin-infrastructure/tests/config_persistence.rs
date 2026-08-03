#[path = "support/faulty_filesystem.rs"]
mod faulty_filesystem;

use std::{fs, path::PathBuf, sync::Arc};

use faulty_filesystem::{FaultPoint, FaultyFileSystem};
use tempfile::TempDir;
use yiyin_application::{ConfigRepository, ErrorCode};
use yiyin_domain::{
    CaseConversion, Config, FontSpec, OutputDirectory, Quality, Template, TemplateField,
};
use yiyin_infrastructure::JsonConfigRepository;

struct Harness {
    _temp: TempDir,
    config_path: PathBuf,
    filesystem: FaultyFileSystem,
}

impl Harness {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        Self {
            config_path: temp.path().join("app-data/config.json"),
            _temp: temp,
            filesystem: FaultyFileSystem::default(),
        }
    }

    fn repository(&self) -> JsonConfigRepository {
        JsonConfigRepository::with_filesystem(
            self.config_path.clone(),
            Arc::new(self.filesystem.clone()),
        )
    }

    fn reload(&self) -> Config {
        JsonConfigRepository::new(self.config_path.clone())
            .load()
            .expect("reload config")
    }
}

fn changed_config() -> Config {
    let mut config = Config {
        output: OutputDirectory::try_from("/chosen/output").expect("output directory"),
        ..Config::default()
    };
    config.options.quality = Quality::try_from(82_u8).expect("quality");
    config.options.iot = true;

    let mut field = TemplateField::custom("Copyright", "Copyright").expect("custom field");
    field.set_visible(false);
    field.set_custom_text("Gaotity", true);
    field.set_font_override(Some(
        FontSpec::new(
            "Custom Sans",
            2.4,
            true,
            true,
            CaseConversion::Uppercase,
            "#123456",
        )
        .expect("font"),
    ));
    config.custom_temp_fields.push(field);

    config.templates.push(
        Template::custom(
            "copyright-line",
            "Copyright line",
            "{Copyright}",
            true,
            FontSpec::new(
                "Custom Sans",
                2.5,
                false,
                true,
                CaseConversion::Lowercase,
                "#abcdef",
            )
            .expect("font"),
        )
        .expect("template"),
    );
    config
}

#[test]
fn complete_config_round_trips_through_versioned_json() {
    let harness = Harness::new();
    let expected = changed_config();

    harness.repository().store(&expected).expect("store config");

    assert_eq!(harness.reload(), expected);
    let stored = fs::read_to_string(&harness.config_path).expect("stored json");
    assert!(stored.contains("\"version\": 1"));
    assert!(stored.contains("\"tempFields\""));
    assert!(stored.contains("\"customTempFields\""));
    assert!(stored.contains("\"temps\""));
    assert!(!stored.contains("\"cacheDir\""));
    assert!(!stored.contains("\"staticDir\""));
    assert!(!stored.contains("versionUpdateInfo"));
}

#[test]
fn replacing_a_valid_config_preserves_a_loadable_backup() {
    let harness = Harness::new();
    let original = Config::default();
    harness
        .repository()
        .store(&original)
        .expect("store original");

    harness
        .repository()
        .store(&changed_config())
        .expect("store changed");

    let backup_path = PathBuf::from(format!("{}.bak", harness.config_path.display()));
    assert_eq!(
        JsonConfigRepository::new(backup_path)
            .load()
            .expect("load backup"),
        original
    );
}

#[test]
fn invalid_json_is_preserved_before_defaults_are_recovered() {
    let harness = Harness::new();
    fs::create_dir_all(harness.config_path.parent().expect("config parent"))
        .expect("create config parent");
    fs::write(&harness.config_path, b"{ definitely not json").expect("write invalid json");

    assert_eq!(
        harness.repository().load().expect("recover config"),
        Config::default()
    );

    let invalid_backup = PathBuf::from(format!("{}.invalid.bak", harness.config_path.display()));
    assert_eq!(
        fs::read(invalid_backup).expect("invalid backup"),
        b"{ definitely not json"
    );
    assert_eq!(harness.reload(), Config::default());
}

#[test]
fn a_contradictory_backup_recovers_with_landscape_forced_off() {
    let harness = Harness::new();
    let mut contradictory = Config::default();
    contradictory.options.landscape = true;
    contradictory.options.background_ratio_visible = true;
    harness
        .repository()
        .store(&contradictory)
        .expect("store contradictory config");
    let backup_path = PathBuf::from(format!("{}.bak", harness.config_path.display()));
    fs::copy(&harness.config_path, &backup_path).expect("seed a pre-rule backup");
    fs::write(&harness.config_path, b"{ definitely not json").expect("corrupt config");

    let recovered = harness.repository().load().expect("recover config");

    assert!(recovered.options.background_ratio_visible);
    assert!(!recovered.options.landscape);
    let reloaded = harness.reload();
    assert!(reloaded.options.background_ratio_visible);
    assert!(!reloaded.options.landscape);
}

#[test]
fn future_schema_is_rejected_without_interpretation() {
    let harness = Harness::new();
    fs::create_dir_all(harness.config_path.parent().expect("config parent"))
        .expect("create config parent");
    fs::write(
        &harness.config_path,
        br#"{"version":2,"config":{"version":"9.0.0"}}"#,
    )
    .expect("write future config");

    let error = harness.repository().load().unwrap_err();

    assert_eq!(error.code(), ErrorCode::ConfigInvalid);
}

#[test]
fn every_failed_durable_publish_stage_keeps_the_prior_config_loadable() {
    for fault in [
        FaultPoint::WriteAndSync,
        FaultPoint::Copy,
        FaultPoint::Rename,
        FaultPoint::SyncParent,
    ] {
        let harness = Harness::new();
        let original = Config::default();
        harness
            .repository()
            .store(&original)
            .expect("store original");
        harness.filesystem.fail_once(fault.clone());

        assert!(harness.repository().store(&changed_config()).is_err());
        assert_eq!(harness.reload(), original, "fault: {fault:?}");
    }
}

#[test]
fn a_config_written_by_an_older_app_version_is_stamped_on_load() {
    let harness = Harness::new();
    let mut older = Config {
        output: OutputDirectory::try_from("/kept/output").expect("output directory"),
        ..Config::default()
    };
    older.version = "1.6.0".to_owned();
    harness
        .repository()
        .store(&older)
        .expect("store older config");

    let loaded = harness.reload();

    assert_eq!(loaded.version, yiyin_domain::CURRENT_VERSION);
    assert_eq!(loaded.output.as_str(), "/kept/output");
}
