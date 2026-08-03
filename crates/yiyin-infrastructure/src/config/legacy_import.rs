use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use yiyin_application::{ApplicationError, ConfigRepository, ImportOutcome};
use yiyin_domain::{Config, FieldContentKind, ResourceId};

use crate::{DirectoryEntryKind, FileSystem};

use super::{
    JsonConfigRepository,
    json_repository::{DecodedLegacyConfig, LegacyImageReference, decode_legacy_config},
};

const COMPLETED_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug)]
pub struct LegacyImportOptions {
    pub owned_resources_root: PathBuf,
    pub candidates: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationMarker {
    pub source_fingerprint: String,
    pub completed_schema_version: u32,
}

#[derive(Serialize, Deserialize)]
struct LegacyResourceManifest {
    version: u32,
    resources: Vec<LegacyResourceManifestEntry>,
}

#[derive(Serialize, Deserialize)]
struct LegacyResourceManifestEntry {
    id: String,
    kind: String,
    display_name: String,
    relative_path: String,
}

#[must_use]
pub fn macos_candidates(application_support: &std::path::Path) -> [PathBuf; 2] {
    [
        application_support.join("壹印"),
        application_support.join("yiyin"),
    ]
}

#[must_use]
pub fn windows_candidates(app_data: &std::path::Path) -> [PathBuf; 2] {
    [app_data.join("壹印"), app_data.join("yiyin")]
}

pub(super) fn import_if_needed(
    repository: &JsonConfigRepository,
) -> Result<ImportOutcome, ApplicationError> {
    let Some(options) = repository.legacy_options() else {
        return Ok(ImportOutcome::clean());
    };
    let Some(source) = select_source(repository.filesystem(), &options.candidates) else {
        return Ok(ImportOutcome::clean());
    };
    let marker_path = marker_path(repository.path())?;

    if repository.filesystem().exists(repository.path()) {
        if repository
            .filesystem()
            .exists(&options.owned_resources_root)
            && !repository.filesystem().exists(&marker_path)
        {
            write_marker(repository.filesystem(), &marker_path, source)?;
        }
        return Ok(ImportOutcome::clean());
    }

    let config_bytes =
        read_contained_file(repository.filesystem(), source, &source.join("config.json"))?;
    let DecodedLegacyConfig {
        mut config,
        image_references,
        mut warnings,
    } = decode_legacy_config(&config_bytes);
    let app_data = repository
        .path()
        .parent()
        .ok_or_else(|| ApplicationError::internal("configuration path has no parent"))?;
    let (staging, staged_resources, resource_warnings) = stage_legacy_resources(
        repository.filesystem(),
        app_data,
        source,
        image_references,
        &mut config,
    )?;
    warnings.extend(resource_warnings);

    if repository
        .filesystem()
        .exists(&options.owned_resources_root)
    {
        repository
            .filesystem()
            .remove_dir_all(&options.owned_resources_root)
            .map_err(internal_io)?;
    }
    let owned_parent = options
        .owned_resources_root
        .parent()
        .ok_or_else(|| ApplicationError::internal("resource path has no parent"))?;
    repository
        .filesystem()
        .create_dir_all(owned_parent)
        .map_err(internal_io)?;
    repository
        .filesystem()
        .rename(&staged_resources, &options.owned_resources_root)
        .map_err(internal_io)?;

    config.normalize();
    repository.store(&config)?;
    write_marker(repository.filesystem(), &marker_path, source)?;
    if repository.filesystem().exists(&staging) {
        repository
            .filesystem()
            .remove_dir_all(&staging)
            .map_err(internal_io)?;
    }
    Ok(ImportOutcome::with_warnings(warnings))
}

fn stage_legacy_resources(
    filesystem: &dyn FileSystem,
    app_data: &Path,
    source: &Path,
    image_references: Vec<LegacyImageReference>,
    config: &mut Config,
) -> Result<(PathBuf, PathBuf, Vec<String>), ApplicationError> {
    let staging = app_data.join(".legacy-import.tmp");
    if filesystem.exists(&staging) {
        filesystem.remove_dir_all(&staging).map_err(internal_io)?;
    }
    let staged_resources = staging.join("resources");
    filesystem
        .create_dir_all(&staged_resources)
        .map_err(internal_io)?;

    let mut warnings = Vec::new();
    copy_optional_file(
        filesystem,
        source,
        &source.join("font.json"),
        &staged_resources.join("font.json"),
    )?;
    copy_optional_tree(
        filesystem,
        source,
        &source.join("font"),
        &staged_resources.join("font"),
        &mut warnings,
    )?;
    copy_optional_tree(
        filesystem,
        source,
        &source.join("static"),
        &staged_resources.join("static"),
        &mut warnings,
    )?;
    let (manifest, valid_image_ids) = build_legacy_resource_manifest(
        filesystem,
        source,
        &staged_resources,
        image_references,
        &mut warnings,
    )?;
    retain_valid_image_references(config, &valid_image_ids);
    if !manifest.resources.is_empty() {
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|error| ApplicationError::internal(error.to_string()))?;
        filesystem
            .write_and_sync(
                &staged_resources.join("legacy-resources.json"),
                &manifest_bytes,
            )
            .map_err(internal_io)?;
    }
    Ok((staging, staged_resources, warnings))
}

fn select_source<'a>(filesystem: &dyn FileSystem, candidates: &'a [PathBuf]) -> Option<&'a Path> {
    candidates
        .iter()
        .find(|candidate| filesystem.exists(&candidate.join("config.json")))
        .map(PathBuf::as_path)
}

fn marker_path(config_path: &Path) -> Result<PathBuf, ApplicationError> {
    config_path
        .parent()
        .map(|parent| parent.join("migration-v1.json"))
        .ok_or_else(|| ApplicationError::internal("configuration path has no parent"))
}

fn write_marker(
    filesystem: &dyn FileSystem,
    marker_path: &Path,
    source: &Path,
) -> Result<(), ApplicationError> {
    let marker = MigrationMarker {
        source_fingerprint: fingerprint(source),
        completed_schema_version: COMPLETED_SCHEMA_VERSION,
    };
    let contents = serde_json::to_vec_pretty(&marker)
        .map_err(|error| ApplicationError::internal(error.to_string()))?;
    crate::durable::durable_publish(
        filesystem,
        marker_path,
        &contents,
        Some(&|prior| serde_json::from_slice::<MigrationMarker>(prior).is_ok()),
        None,
    )
}

fn fingerprint(path: &Path) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in path.to_string_lossy().as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn build_legacy_resource_manifest(
    filesystem: &dyn FileSystem,
    source_root: &Path,
    staged_resources: &Path,
    references: Vec<LegacyImageReference>,
    warnings: &mut Vec<String>,
) -> Result<(LegacyResourceManifest, BTreeSet<ResourceId>), ApplicationError> {
    let mut resources = Vec::new();
    let mut valid_ids = BTreeSet::new();
    for reference in references {
        let Some(relative_path) = legacy_static_relative_path(source_root, &reference.source)
        else {
            warnings.push("Skipped an invalid legacy image reference.".to_owned());
            continue;
        };
        let owned_relative = Path::new("static").join(&relative_path);
        let staged_path = staged_resources.join(&owned_relative);
        if !filesystem.exists(&staged_path) {
            let display_name = relative_path
                .file_name()
                .map_or_else(|| "resource".into(), |name| name.to_string_lossy());
            warnings.push(format!("Skipped missing legacy resource: {display_name}"));
            continue;
        }
        let display_name = relative_path
            .file_name()
            .ok_or_else(|| ApplicationError::internal("legacy resource has no file name"))?
            .to_string_lossy()
            .into_owned();
        resources.push(LegacyResourceManifestEntry {
            id: reference.id.as_str().to_owned(),
            kind: "overlay".to_owned(),
            display_name,
            relative_path: path_to_portable_string(&owned_relative)?,
        });
        valid_ids.insert(reference.id);
    }
    Ok((
        LegacyResourceManifest {
            version: COMPLETED_SCHEMA_VERSION,
            resources,
        },
        valid_ids,
    ))
}

fn legacy_static_relative_path(source_root: &Path, value: &str) -> Option<PathBuf> {
    let raw = value.strip_prefix("file://").unwrap_or(value);
    let path = Path::new(raw);
    let relative = path
        .strip_prefix(source_root.join("static"))
        .ok()
        .map(Path::to_path_buf)
        .or_else(|| path.file_name().map(PathBuf::from))?;
    relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
        .then_some(relative)
}

fn retain_valid_image_references(config: &mut Config, valid_ids: &BTreeSet<ResourceId>) {
    for field in config
        .temp_fields
        .iter_mut()
        .chain(&mut config.custom_temp_fields)
    {
        if field.content_kind() != FieldContentKind::Image {
            continue;
        }
        let dark = field
            .dark_image()
            .filter(|id| valid_ids.contains(*id))
            .cloned();
        let light = field
            .light_image()
            .filter(|id| valid_ids.contains(*id))
            .cloned();
        field.set_image_variants(dark, light);
    }
}

fn path_to_portable_string(path: &Path) -> Result<String, ApplicationError> {
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(value) = component else {
            return Err(ApplicationError::forbidden());
        };
        components.push(value.to_string_lossy());
    }
    Ok(components.join("/"))
}

fn copy_optional_file(
    filesystem: &dyn FileSystem,
    allowed_root: &Path,
    source: &Path,
    destination: &Path,
) -> Result<(), ApplicationError> {
    if !filesystem.exists(source) {
        return Ok(());
    }
    copy_file(filesystem, allowed_root, source, destination)
}

fn copy_optional_tree(
    filesystem: &dyn FileSystem,
    allowed_root: &Path,
    source: &Path,
    destination: &Path,
    warnings: &mut Vec<String>,
) -> Result<(), ApplicationError> {
    if !filesystem.exists(source) {
        return Ok(());
    }
    filesystem
        .create_dir_all(destination)
        .map_err(internal_io)?;
    let mut entries = filesystem.read_dir(source).map_err(internal_io)?;
    entries.sort_by(|left, right| left.path().cmp(right.path()));
    for entry in entries {
        let Some(name) = entry.path().file_name() else {
            warnings.push("Skipped an unnamed legacy resource.".to_owned());
            continue;
        };
        let target = destination.join(name);
        match entry.kind() {
            DirectoryEntryKind::File => {
                copy_file(filesystem, allowed_root, entry.path(), &target)?;
            }
            DirectoryEntryKind::Directory => {
                copy_optional_tree(filesystem, allowed_root, entry.path(), &target, warnings)?;
            }
            DirectoryEntryKind::Symlink | DirectoryEntryKind::Other => warnings.push(format!(
                "Skipped unsupported legacy resource: {}",
                name.to_string_lossy()
            )),
        }
    }
    Ok(())
}

fn copy_file(
    filesystem: &dyn FileSystem,
    allowed_root: &Path,
    source: &Path,
    destination: &Path,
) -> Result<(), ApplicationError> {
    let parent = destination
        .parent()
        .ok_or_else(|| ApplicationError::internal("resource path has no parent"))?;
    filesystem.create_dir_all(parent).map_err(internal_io)?;
    let contents = read_contained_file(filesystem, allowed_root, source)?;
    filesystem
        .write_and_sync(destination, &contents)
        .map_err(internal_io)?;
    let copied = filesystem.read(destination).map_err(internal_io)?;
    if copied != contents {
        return Err(ApplicationError::internal(
            "legacy resource validation failed",
        ));
    }
    Ok(())
}

fn read_contained_file(
    filesystem: &dyn FileSystem,
    allowed_root: &Path,
    source: &Path,
) -> Result<Vec<u8>, ApplicationError> {
    if filesystem.entry_kind(source).map_err(internal_io)? != DirectoryEntryKind::File {
        return Err(ApplicationError::forbidden());
    }
    let canonical_root = filesystem.canonicalize(allowed_root).map_err(internal_io)?;
    let canonical_source = filesystem.canonicalize(source).map_err(internal_io)?;
    if !canonical_source.starts_with(&canonical_root) {
        return Err(ApplicationError::forbidden());
    }
    filesystem.read(&canonical_source).map_err(internal_io)
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the function is passed directly to Result::map_err"
)]
fn internal_io(error: std::io::Error) -> ApplicationError {
    ApplicationError::internal(error.to_string())
}
