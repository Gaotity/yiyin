#[path = "support/faulty_filesystem.rs"]
mod faulty_filesystem;

use std::{fs, path::PathBuf};

use faulty_filesystem::{FaultPoint, FaultyFileSystem};
use tempfile::TempDir;
use yiyin_application::{ApplicationError, CancellationProbe, ErrorCode};
use yiyin_infrastructure::durable::durable_publish;

const PRIOR: &[u8] = b"valid prior contents";
const NEW: &[u8] = b"fresh durable contents";

fn accepts_prior(bytes: &[u8]) -> bool {
    bytes == PRIOR
}

struct AlwaysCancelled;

impl CancellationProbe for AlwaysCancelled {
    fn is_cancelled(&self) -> bool {
        true
    }
}

struct Harness {
    _temp: TempDir,
    destination: PathBuf,
    filesystem: FaultyFileSystem,
}

impl Harness {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        Self {
            destination: temp.path().join("nested/output.bin"),
            _temp: temp,
            filesystem: FaultyFileSystem::default(),
        }
    }

    fn temporary(&self) -> PathBuf {
        PathBuf::from(format!("{}.tmp", self.destination.display()))
    }

    fn backup(&self) -> PathBuf {
        PathBuf::from(format!("{}.bak", self.destination.display()))
    }

    fn write_prior(&self, contents: &[u8]) {
        let parent = self.destination.parent().expect("destination parent");
        fs::create_dir_all(parent).expect("destination parent");
        fs::write(&self.destination, contents).expect("prior destination");
    }

    #[allow(
        clippy::type_complexity,
        reason = "the helper mirrors the durable_publish signature under test"
    )]
    fn publish(
        &self,
        contents: &[u8],
        prior_is_valid: Option<&dyn Fn(&[u8]) -> bool>,
        cancellation: Option<&dyn CancellationProbe>,
    ) -> Result<(), ApplicationError> {
        durable_publish(
            &self.filesystem,
            &self.destination,
            contents,
            prior_is_valid,
            cancellation,
        )
    }
}

#[test]
fn a_fresh_publish_writes_the_destination_and_clears_the_temporary() {
    let harness = Harness::new();

    harness.publish(NEW, None, None).expect("publish");

    assert_eq!(fs::read(&harness.destination).expect("destination"), NEW);
    assert!(!harness.temporary().exists());
}

#[test]
fn a_stale_temporary_is_removed_before_publishing() {
    let harness = Harness::new();
    let parent = harness.destination.parent().expect("destination parent");
    fs::create_dir_all(parent).expect("destination parent");
    fs::write(harness.temporary(), b"stale interrupted write").expect("stale tmp");

    harness.publish(NEW, None, None).expect("publish");

    assert_eq!(fs::read(&harness.destination).expect("destination"), NEW);
    assert!(!harness.temporary().exists());
}

#[test]
fn republishing_over_a_valid_prior_keeps_a_backup_of_the_prior_contents() {
    let harness = Harness::new();
    harness.publish(PRIOR, None, None).expect("initial publish");

    harness
        .publish(NEW, Some(&accepts_prior), None)
        .expect("republish");

    assert_eq!(fs::read(&harness.destination).expect("destination"), NEW);
    assert_eq!(fs::read(harness.backup()).expect("backup"), PRIOR);
    assert!(!harness.temporary().exists());
}

#[test]
fn republishing_over_an_invalid_prior_keeps_no_backup() {
    let harness = Harness::new();
    harness.write_prior(b"corrupted prior contents");

    harness
        .publish(NEW, Some(&accepts_prior), None)
        .expect("republish");

    assert_eq!(fs::read(&harness.destination).expect("destination"), NEW);
    assert!(!harness.backup().exists());
}

#[test]
fn write_and_sync_failure_removes_the_temporary_and_never_creates_the_destination() {
    let harness = Harness::new();
    harness.filesystem.fail_once(FaultPoint::WriteAndSync);

    assert!(harness.publish(NEW, None, None).is_err());

    assert!(!harness.destination.exists());
    assert!(!harness.temporary().exists());
}

#[test]
fn backup_copy_failure_keeps_the_original_destination_untouched() {
    let harness = Harness::new();
    harness.write_prior(PRIOR);
    harness.filesystem.fail_once(FaultPoint::Copy);

    assert!(harness.publish(NEW, Some(&accepts_prior), None).is_err());

    assert_eq!(fs::read(&harness.destination).expect("destination"), PRIOR);
    assert!(!harness.temporary().exists());
    assert!(!harness.backup().exists());
}

#[test]
fn rename_failure_keeps_the_original_destination_untouched() {
    let harness = Harness::new();
    harness.write_prior(PRIOR);
    harness.filesystem.fail_once(FaultPoint::Rename);

    assert!(harness.publish(NEW, Some(&accepts_prior), None).is_err());

    assert_eq!(fs::read(&harness.destination).expect("destination"), PRIOR);
    assert!(!harness.temporary().exists());
}

#[test]
fn sync_parent_failure_without_a_prior_removes_the_destination() {
    let harness = Harness::new();
    harness.filesystem.fail_once(FaultPoint::SyncParent);

    assert!(harness.publish(NEW, None, None).is_err());

    assert!(!harness.destination.exists());
    assert!(!harness.temporary().exists());
}

#[test]
fn sync_parent_failure_with_an_invalid_prior_removes_the_destination() {
    let harness = Harness::new();
    harness.write_prior(b"corrupted prior contents");
    harness.filesystem.fail_once(FaultPoint::SyncParent);

    assert!(harness.publish(NEW, Some(&accepts_prior), None).is_err());

    assert!(!harness.destination.exists());
    assert!(!harness.temporary().exists());
}

#[test]
fn sync_parent_failure_with_a_valid_backup_restores_the_prior_contents() {
    let harness = Harness::new();
    harness.write_prior(PRIOR);
    harness.filesystem.fail_once(FaultPoint::SyncParent);

    assert!(harness.publish(NEW, Some(&accepts_prior), None).is_err());

    assert_eq!(
        fs::read(&harness.destination).expect("restored destination"),
        PRIOR
    );
    assert!(!harness.temporary().exists());
}

#[test]
fn cancellation_before_the_rename_removes_the_temporary_and_publishes_nothing() {
    let harness = Harness::new();

    let error = harness
        .publish(NEW, None, Some(&AlwaysCancelled))
        .expect_err("cancelled publish");

    assert_eq!(error.code(), ErrorCode::Cancelled);
    assert!(!harness.destination.exists());
    assert!(!harness.temporary().exists());
}
