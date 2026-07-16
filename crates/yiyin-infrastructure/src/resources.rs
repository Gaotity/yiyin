use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use image::GenericImageView;
use yiyin_application::{
    ApplicationError, IdGenerator, MetadataReader, ResourceRecord, ResourceRepository,
};
use yiyin_domain::{
    ImageDensity, ImageDimensions, ImageOrientation, ResourceId, ResourceKind, TaskId,
};

use crate::{DirectoryEntryKind, ExifMetadataReader, FileSystem, StdFileSystem};

pub struct ResourceRegistry {
    records: RwLock<HashMap<ResourceId, ResourceRecord>>,
    owned_root: PathBuf,
    filesystem: Arc<dyn FileSystem>,
}

impl ResourceRegistry {
    /// Creates an empty registry rooted in application-owned storage.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when the owned root cannot be created or canonicalized.
    pub fn new(owned_root: impl AsRef<Path>) -> Result<Self, ApplicationError> {
        Self::with_filesystem(owned_root.as_ref(), Arc::new(StdFileSystem))
    }

    /// Creates a registry with an injected filesystem adapter.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when the owned root cannot be created or canonicalized.
    pub fn with_filesystem(
        owned_root: &Path,
        filesystem: Arc<dyn FileSystem>,
    ) -> Result<Self, ApplicationError> {
        filesystem.create_dir_all(owned_root).map_err(internal_io)?;
        let owned_root = filesystem.canonicalize(owned_root).map_err(internal_io)?;
        Ok(Self {
            records: RwLock::new(HashMap::new()),
            owned_root,
            filesystem,
        })
    }

    #[must_use]
    pub fn owned_root(&self) -> &Path {
        &self.owned_root
    }

    /// Registers an already atomically published Rust-generated image.
    ///
    /// # Errors
    ///
    /// Returns `FORBIDDEN` for non-generated kinds and stable file errors for
    /// missing or invalid image bytes.
    pub fn register_generated(
        &self,
        kind: ResourceKind,
        source: &Path,
        display_name: &str,
    ) -> Result<ResourceRecord, ApplicationError> {
        if !matches!(kind, ResourceKind::Output | ResourceKind::Preview) {
            return Err(ApplicationError::forbidden());
        }
        let parent = source.parent().ok_or_else(ApplicationError::file_invalid)?;
        let allowed_root = self.filesystem.canonicalize(parent).map_err(internal_io)?;
        let (canonical, inspected) = self.inspect_source(source, kind)?;
        if !canonical.starts_with(&allowed_root) {
            return Err(ApplicationError::forbidden());
        }
        let record = build_record(
            self.next_resource_id(),
            kind,
            display_name,
            canonical,
            &inspected,
            allowed_root,
        );
        self.insert(record)
    }

    fn inspect_source(
        &self,
        source: &Path,
        kind: ResourceKind,
    ) -> Result<(PathBuf, InspectedFile), ApplicationError> {
        if !self.filesystem.exists(source) {
            return Err(ApplicationError::file_not_found());
        }
        if self.filesystem.entry_kind(source).map_err(internal_io)? != DirectoryEntryKind::File {
            return Err(ApplicationError::file_invalid());
        }
        let canonical = self.filesystem.canonicalize(source).map_err(internal_io)?;
        let bytes = self.filesystem.read(&canonical).map_err(internal_io)?;
        let mut inspected = inspect_file(&bytes, &canonical, kind)?;
        populate_image_density(&mut inspected, &canonical);
        Ok((canonical, inspected))
    }

    fn insert(&self, record: ResourceRecord) -> Result<ResourceRecord, ApplicationError> {
        self.records
            .write()
            .map_err(|_| ApplicationError::internal("resource registry lock poisoned"))?
            .insert(record.id().clone(), record.clone());
        Ok(record)
    }

    fn publish_owned(
        &self,
        kind: ResourceKind,
        source: &Path,
        display_name: &str,
        inspected: &InspectedFile,
    ) -> Result<ResourceRecord, ApplicationError> {
        let directory = self.owned_root.join(match kind {
            ResourceKind::Font => "fonts",
            ResourceKind::Overlay => "overlays",
            _ => return Err(ApplicationError::forbidden()),
        });
        self.filesystem
            .create_dir_all(&directory)
            .map_err(internal_io)?;
        let allowed_root = self
            .filesystem
            .canonicalize(&directory)
            .map_err(internal_io)?;
        let id = self.next_resource_id();
        let destination = directory.join(format!("{}.{}", id.as_str(), inspected.extension));
        let temporary = with_suffix(&destination, ".tmp");
        let bytes = self.filesystem.read(source).map_err(internal_io)?;
        if let Err(error) = self.filesystem.write_and_sync(&temporary, &bytes) {
            let _ = remove_if_present(self.filesystem.as_ref(), &temporary);
            return Err(internal_io(error));
        }
        if let Err(error) = self.filesystem.rename(&temporary, &destination) {
            let _ = remove_if_present(self.filesystem.as_ref(), &temporary);
            return Err(internal_io(error));
        }
        self.filesystem
            .sync_parent(&destination)
            .map_err(internal_io)?;
        let canonical = self
            .filesystem
            .canonicalize(&destination)
            .map_err(internal_io)?;
        let record = build_record(id, kind, display_name, canonical, inspected, allowed_root);
        self.insert(record)
    }

    fn revalidate(&self, record: &ResourceRecord) -> Result<ResourceRecord, ApplicationError> {
        if !self.filesystem.exists(record.source()) {
            return Err(ApplicationError::file_not_found());
        }
        let canonical = self
            .filesystem
            .canonicalize(record.source())
            .map_err(internal_io)?;
        if !canonical.starts_with(record.allowed_root()) {
            return Err(ApplicationError::forbidden());
        }
        if self
            .filesystem
            .entry_kind(&canonical)
            .map_err(internal_io)?
            != DirectoryEntryKind::File
        {
            return Err(ApplicationError::file_invalid());
        }
        let bytes = self.filesystem.read(&canonical).map_err(internal_io)?;
        let mut inspected = inspect_file(&bytes, &canonical, record.kind())?;
        populate_image_density(&mut inspected, &canonical);
        if inspected.mime_type != record.mime_type() {
            return Err(ApplicationError::file_invalid());
        }
        Ok(build_record(
            record.id().clone(),
            record.kind(),
            record.display_name(),
            canonical,
            &inspected,
            record.allowed_root().to_path_buf(),
        ))
    }
}

impl ResourceRepository for ResourceRegistry {
    fn register_input(&self, source: &Path) -> Result<ResourceRecord, ApplicationError> {
        let parent = source.parent().ok_or_else(ApplicationError::file_invalid)?;
        let allowed_root = self.filesystem.canonicalize(parent).map_err(internal_io)?;
        let (canonical, inspected) = self.inspect_source(source, ResourceKind::Input)?;
        if !canonical.starts_with(&allowed_root) {
            return Err(ApplicationError::forbidden());
        }
        let display_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(ApplicationError::file_invalid)?;
        let record = build_record(
            self.next_resource_id(),
            ResourceKind::Input,
            display_name,
            canonical,
            &inspected,
            allowed_root,
        );
        self.insert(record)
    }

    fn register_owned(
        &self,
        kind: ResourceKind,
        source: &Path,
        display_name: &str,
    ) -> Result<ResourceRecord, ApplicationError> {
        if display_name.trim().is_empty() {
            return Err(ApplicationError::invalid_request(
                "The resource name is required.",
            ));
        }
        let (canonical, inspected) = self.inspect_source(source, kind)?;
        self.publish_owned(kind, &canonical, display_name, &inspected)
    }

    fn remove(&self, id: &ResourceId) -> Result<(), ApplicationError> {
        let record = self
            .records
            .write()
            .map_err(|_| ApplicationError::internal("resource registry lock poisoned"))?
            .remove(id)
            .ok_or_else(ApplicationError::resource_not_found)?;
        let owned_resource = matches!(record.kind(), ResourceKind::Font | ResourceKind::Overlay)
            && record.source().starts_with(&self.owned_root);
        let generated_preview = record.kind() == ResourceKind::Preview
            && record.source().starts_with(record.allowed_root());
        if owned_resource || generated_preview {
            remove_if_present(self.filesystem.as_ref(), record.source())?;
        }
        Ok(())
    }

    fn resolve(&self, id: &ResourceId) -> Result<ResourceRecord, ApplicationError> {
        let record = self
            .records
            .read()
            .map_err(|_| ApplicationError::internal("resource registry lock poisoned"))?
            .get(id)
            .cloned()
            .ok_or_else(ApplicationError::resource_not_found)?;
        self.revalidate(&record)
    }

    fn snapshot(&self) -> Vec<ResourceRecord> {
        self.records.read().map_or_else(
            |_| Vec::new(),
            |records| records.values().cloned().collect(),
        )
    }
}

impl IdGenerator for ResourceRegistry {
    fn next_resource_id(&self) -> ResourceId {
        ResourceId::try_from(uuid::Uuid::new_v4().to_string()).expect("UUID is never empty")
    }

    fn next_task_id(&self) -> TaskId {
        TaskId::try_from(uuid::Uuid::new_v4().to_string()).expect("UUID is never empty")
    }
}

#[derive(Clone, Debug)]
struct InspectedFile {
    mime_type: &'static str,
    extension: &'static str,
    dimensions: Option<ImageDimensions>,
    density: Option<ImageDensity>,
}

fn inspect_file(
    bytes: &[u8],
    path: &Path,
    kind: ResourceKind,
) -> Result<InspectedFile, ApplicationError> {
    match kind {
        ResourceKind::Input
        | ResourceKind::Overlay
        | ResourceKind::Output
        | ResourceKind::Preview => inspect_image(bytes, path),
        ResourceKind::Font => inspect_font(bytes, path),
        ResourceKind::BundledAsset => Err(ApplicationError::forbidden()),
    }
}

fn inspect_image(bytes: &[u8], path: &Path) -> Result<InspectedFile, ApplicationError> {
    let (format, mime_type, extension) = if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        (image::ImageFormat::Jpeg, "image/jpeg", "jpg")
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        (image::ImageFormat::Png, "image/png", "png")
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        (image::ImageFormat::WebP, "image/webp", "webp")
    } else {
        return Err(ApplicationError::file_invalid());
    };
    validate_extension(path, extension, format == image::ImageFormat::Jpeg)?;
    let decoded = image::load_from_memory_with_format(bytes, format)
        .map_err(|_| ApplicationError::file_invalid())?;
    let (width, height) = decoded.dimensions();
    Ok(InspectedFile {
        mime_type,
        extension,
        dimensions: Some(
            ImageDimensions::new(width, height).map_err(|_| ApplicationError::file_invalid())?,
        ),
        density: None,
    })
}

fn inspect_font(bytes: &[u8], path: &Path) -> Result<InspectedFile, ApplicationError> {
    let (mime_type, extension) = if bytes.starts_with(&[0, 1, 0, 0])
        || bytes.starts_with(b"true")
        || bytes.starts_with(b"typ1")
    {
        ("font/ttf", "ttf")
    } else if bytes.starts_with(b"OTTO") {
        ("font/otf", "otf")
    } else {
        return Err(ApplicationError::file_invalid());
    };
    validate_extension(path, extension, false)?;
    Ok(InspectedFile {
        mime_type,
        extension,
        dimensions: None,
        density: None,
    })
}

fn validate_extension(
    path: &Path,
    expected: &str,
    jpeg_alias: bool,
) -> Result<(), ApplicationError> {
    let actual = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(ApplicationError::file_invalid)?;
    if actual == expected || (jpeg_alias && actual == "jpeg") {
        Ok(())
    } else {
        Err(ApplicationError::file_invalid())
    }
}

fn build_record(
    id: ResourceId,
    kind: ResourceKind,
    display_name: &str,
    source: PathBuf,
    inspected: &InspectedFile,
    allowed_root: PathBuf,
) -> ResourceRecord {
    let record = ResourceRecord::new(id, kind, display_name, source)
        .with_security_context(inspected.mime_type, allowed_root);
    if let Some(dimensions) = inspected.dimensions {
        record.with_image_info(dimensions, inspected.density)
    } else {
        record
    }
}

fn populate_image_density(inspected: &mut InspectedFile, source: &Path) {
    if inspected.dimensions.is_some()
        && let Some(metadata) = ExifMetadataReader.read(source).ok().flatten()
    {
        inspected.density = if inspected.mime_type == "image/webp" {
            None
        } else {
            metadata.density()
        };
        if matches!(
            metadata.orientation(),
            Some(
                ImageOrientation::MirrorHorizontalRotate270
                    | ImageOrientation::Rotate90
                    | ImageOrientation::MirrorHorizontalRotate90
                    | ImageOrientation::Rotate270
            )
        ) && let Some(dimensions) = inspected.dimensions
        {
            inspected.dimensions = ImageDimensions::new(dimensions.height, dimensions.width).ok();
        }
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map_or_else(OsString::new, std::ffi::OsStr::to_os_string);
    name.push(suffix);
    path.with_file_name(name)
}

fn remove_if_present(filesystem: &dyn FileSystem, path: &Path) -> Result<(), ApplicationError> {
    if filesystem.exists(path) {
        filesystem.remove_file(path).map_err(internal_io)?;
    }
    Ok(())
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the function is passed directly to Result::map_err"
)]
fn internal_io(error: std::io::Error) -> ApplicationError {
    ApplicationError::internal(error.to_string())
}
