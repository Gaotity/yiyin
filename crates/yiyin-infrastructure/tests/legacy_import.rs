use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use tempfile::TempDir;
use yiyin_application::ConfigRepository;
use yiyin_infrastructure::{
    DirectoryEntry, FileSystem, JsonConfigRepository, LegacyImportOptions, StdFileSystem,
    macos_candidates, windows_candidates,
};

#[derive(Clone, Default)]
struct FailNextRename {
    inner: StdFileSystem,
    fail: Arc<AtomicBool>,
}

impl FailNextRename {
    fn arm(&self) {
        self.fail.store(true, Ordering::SeqCst);
    }
}

impl FileSystem for FailNextRename {
    fn exists(&self, path: &Path) -> bool {
        self.inner.exists(path)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.inner.canonicalize(path)
    }

    fn entry_kind(&self, path: &Path) -> io::Result<yiyin_infrastructure::DirectoryEntryKind> {
        self.inner.entry_kind(path)
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        self.inner.create_dir_all(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.inner.read(path)
    }

    fn write_and_sync(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        self.inner.write_and_sync(path, contents)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
        self.inner.copy(from, to)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        if self.fail.swap(false, Ordering::SeqCst) {
            Err(io::Error::other("injected rename failure"))
        } else {
            self.inner.rename(from, to)
        }
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        self.inner.remove_file(path)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        self.inner.remove_dir_all(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>> {
        self.inner.read_dir(path)
    }

    fn sync_parent(&self, path: &Path) -> io::Result<()> {
        self.inner.sync_parent(path)
    }
}

struct Harness {
    _temp: TempDir,
    support: PathBuf,
    app_data: PathBuf,
    resources: PathBuf,
    filesystem: FailNextRename,
}

impl Harness {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let support = temp.path().join("Application Support");
        let app_data = temp.path().join("tauri-app-data");
        Self {
            resources: app_data.join("resources"),
            _temp: temp,
            support,
            app_data,
            filesystem: FailNextRename::default(),
        }
    }

    fn repository(&self) -> JsonConfigRepository {
        JsonConfigRepository::with_filesystem(
            self.app_data.join("config.json"),
            Arc::new(self.filesystem.clone()),
        )
        .with_legacy_import(LegacyImportOptions {
            owned_resources_root: self.resources.clone(),
            candidates: macos_candidates(&self.support).into(),
        })
    }

    fn write_legacy(&self, name: &str, output: &str) -> PathBuf {
        let root = self.support.join(name);
        fs::create_dir_all(root.join("font")).expect("font dir");
        fs::create_dir_all(root.join("static")).expect("static dir");
        let image_reference = root.join("static/overlay.png");
        fs::write(
            root.join("config.json"),
            format!(
                r#"{{
  "version": "1.6.0",
  "dir": "/private/legacy/config.json",
  "output": "{output}",
  "cacheDir": "/private/legacy/cache",
  "staticDir": "/private/legacy/static",
  "font": {{"path":"/private/legacy/font.json","dir":"/private/legacy/font","map":{{}}}},
  "options": {{"iot":true,"quality":77}},
  "tempFields": [{{"key":"Make","name":"Logo","type":"img","bImg":"{}","wImg":""}}],
  "temps": [{{"key":"make-model","name":"Logo型号模版","temp":"{{Make}} {{Model}}","use":true,"type":"system","font":{{"size":3,"bold":true}},"position":{{"left":null,"right":null,"top":null,"bottom":null}},"verticalAlign":"baseline"}}],
  "versionUpdateInfo": {{"version":"1.6.0","downloadLink":"","checkDate":0}}
}}"#,
                image_reference.to_string_lossy()
            ),
        )
        .expect("legacy config");
        fs::write(root.join("font.json"), br#"{"Body":"body.ttf"}"#).expect("font metadata");
        fs::write(root.join("font/body.ttf"), b"font bytes").expect("font file");
        fs::write(root.join("static/overlay.png"), b"overlay bytes").expect("static file");
        root
    }
}

fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, current: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        let mut entries = fs::read_dir(current)
            .expect("read directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("directory entries");
        entries.sort_by_key(fs::DirEntry::path);
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(root)
                        .expect("relative path")
                        .to_path_buf(),
                    fs::read(path).expect("file bytes"),
                );
            }
        }
    }

    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

#[test]
fn platform_candidates_preserve_both_legacy_names_in_priority_order() {
    let root = Path::new("root");
    assert_eq!(
        macos_candidates(root),
        [root.join("壹印"), root.join("yiyin")]
    );
    assert_eq!(
        windows_candidates(root),
        [root.join("壹印"), root.join("yiyin")]
    );
}

#[test]
fn preferred_legacy_state_copies_once_without_mutating_its_source() {
    let harness = Harness::new();
    let preferred = harness.write_legacy("壹印", "/preferred/output");
    harness.write_legacy("yiyin", "/fallback/output");
    let before = snapshot(&preferred);

    let outcome = harness
        .repository()
        .import_legacy_if_needed()
        .expect("import legacy");

    assert!(outcome.warnings().is_empty());
    let config = harness.repository().load().expect("load config");
    assert_eq!(config.output, "/preferred/output");
    let make = config
        .temp_fields
        .iter()
        .find(|field| field.key().as_str() == "Make")
        .expect("make field");
    let image_id = make.dark_image().expect("opaque migrated image");
    assert!(image_id.as_str().starts_with("legacy-"));
    assert!(!image_id.as_str().contains("Application Support"));
    assert_eq!(
        fs::read(harness.resources.join("font.json")).expect("copied font metadata"),
        br#"{"Body":"body.ttf"}"#
    );
    assert_eq!(
        fs::read(harness.resources.join("font/body.ttf")).expect("copied font"),
        b"font bytes"
    );
    assert_eq!(
        fs::read(harness.resources.join("static/overlay.png")).expect("copied static"),
        b"overlay bytes"
    );
    let resource_manifest = fs::read_to_string(harness.resources.join("legacy-resources.json"))
        .expect("legacy resource manifest");
    assert!(resource_manifest.contains(image_id.as_str()));
    assert!(resource_manifest.contains("static/overlay.png"));
    assert!(!resource_manifest.contains(preferred.to_string_lossy().as_ref()));
    let marker =
        fs::read_to_string(harness.app_data.join("migration-v1.json")).expect("migration marker");
    assert!(marker.contains("\"completed_schema_version\": 1"));
    assert!(!marker.contains(preferred.to_string_lossy().as_ref()));
    assert_eq!(snapshot(&preferred), before);

    fs::write(preferred.join("font/body.ttf"), b"changed after import")
        .expect("change source after import");
    harness
        .repository()
        .import_legacy_if_needed()
        .expect("idempotent import");
    assert_eq!(
        fs::read(harness.resources.join("font/body.ttf")).expect("owned font"),
        b"font bytes"
    );
}

#[test]
fn interrupted_resource_publication_retries_without_committing_partial_config() {
    let harness = Harness::new();
    harness.write_legacy("壹印", "/legacy/output");
    harness.filesystem.arm();

    assert!(harness.repository().import_legacy_if_needed().is_err());
    assert!(!harness.app_data.join("config.json").exists());

    harness
        .repository()
        .import_legacy_if_needed()
        .expect("retry import");

    assert_eq!(
        harness.repository().load().expect("load imported").output,
        "/legacy/output"
    );
    assert!(harness.resources.join("font/body.ttf").exists());
    assert!(harness.app_data.join("migration-v1.json").exists());
}

#[cfg(unix)]
#[test]
fn unsupported_symlink_is_skipped_with_a_safe_warning() {
    use std::os::unix::fs::symlink;

    let harness = Harness::new();
    let legacy = harness.write_legacy("壹印", "/legacy/output");
    symlink("/private/external", legacy.join("static/external-link"))
        .expect("create external symlink");

    let outcome = harness
        .repository()
        .import_legacy_if_needed()
        .expect("import with warning");

    assert_eq!(outcome.warnings().len(), 1);
    assert!(outcome.warnings()[0].contains("external-link"));
    assert!(!outcome.warnings()[0].contains("/private/external"));
}
