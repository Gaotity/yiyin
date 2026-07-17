#![allow(
    clippy::needless_pass_by_value,
    reason = "I/O errors are consumed by map_err adapter functions"
)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::{AppHandle, Manager, Wry};
use yiyin_application::{
    Bootstrap, CancelTask, ClearTasks, ConfigRepository, PreviewTask, ReadTaskExif, RegisterFont,
    RegisterImages, RegisterOverlay, RemoveFont, ResetConfig, ResourceRepository, StartTasks,
    TaskQueue, UpdateConfig,
};
use yiyin_infrastructure::{
    ExifMetadataReader, JsonConfigRepository, LegacyImportOptions, ResourceRegistry,
    RustImageRenderer, TokioTaskQueue,
};

#[cfg(target_os = "macos")]
use yiyin_infrastructure::macos_candidates;
#[cfg(target_os = "windows")]
use yiyin_infrastructure::windows_candidates;

use crate::{events::TauriTaskEventSink, native::NativeOutputDirectory};

const BUNDLED_IMAGES: [(&str, &[u8]); 28] = [
    (
        "zs-wx.jpg",
        include_bytes!("../../assets/donation/zs-wx.jpg"),
    ),
    (
        "zs-zfb.jpg",
        include_bytes!("../../assets/donation/zs-zfb.jpg"),
    ),
    (
        "canon-b.png",
        include_bytes!("../../assets/logos/canon-b.png"),
    ),
    (
        "canon-w.png",
        include_bytes!("../../assets/logos/canon-w.png"),
    ),
    ("dji-b.png", include_bytes!("../../assets/logos/dji-b.png")),
    ("dji-w.png", include_bytes!("../../assets/logos/dji-w.png")),
    (
        "fujifilm-b.png",
        include_bytes!("../../assets/logos/fujifilm-b.png"),
    ),
    (
        "fujifilm-w.png",
        include_bytes!("../../assets/logos/fujifilm-w.png"),
    ),
    (
        "hasselblad-b.png",
        include_bytes!("../../assets/logos/hasselblad-b.png"),
    ),
    (
        "hasselblad-w.png",
        include_bytes!("../../assets/logos/hasselblad-w.png"),
    ),
    (
        "leica-b.png",
        include_bytes!("../../assets/logos/leica-b.png"),
    ),
    (
        "leica-w.png",
        include_bytes!("../../assets/logos/leica-w.png"),
    ),
    (
        "nikon-b.png",
        include_bytes!("../../assets/logos/nikon-b.png"),
    ),
    (
        "nikon-w.png",
        include_bytes!("../../assets/logos/nikon-w.png"),
    ),
    (
        "olympus-b.png",
        include_bytes!("../../assets/logos/olympus-b.png"),
    ),
    (
        "olympus-w.png",
        include_bytes!("../../assets/logos/olympus-w.png"),
    ),
    (
        "panasonic-b.png",
        include_bytes!("../../assets/logos/panasonic-b.png"),
    ),
    (
        "panasonic-w.png",
        include_bytes!("../../assets/logos/panasonic-w.png"),
    ),
    (
        "pentax-b.png",
        include_bytes!("../../assets/logos/pentax-b.png"),
    ),
    (
        "pentax-w.png",
        include_bytes!("../../assets/logos/pentax-w.png"),
    ),
    (
        "ricoh-b.png",
        include_bytes!("../../assets/logos/ricoh-b.png"),
    ),
    (
        "ricoh-w.png",
        include_bytes!("../../assets/logos/ricoh-w.png"),
    ),
    (
        "sigma-b.png",
        include_bytes!("../../assets/logos/sigma-b.png"),
    ),
    (
        "sigma-w.png",
        include_bytes!("../../assets/logos/sigma-w.png"),
    ),
    (
        "songdian-b.png",
        include_bytes!("../../assets/logos/songdian-b.png"),
    ),
    (
        "songdian-w.png",
        include_bytes!("../../assets/logos/songdian-w.png"),
    ),
    (
        "sony-b.png",
        include_bytes!("../../assets/logos/sony-b.png"),
    ),
    (
        "sony-w.png",
        include_bytes!("../../assets/logos/sony-w.png"),
    ),
];

pub struct AppState {
    pub bootstrap: Bootstrap,
    pub update_config: UpdateConfig,
    pub reset_config: ResetConfig,
    pub register_images: RegisterImages,
    pub register_font: RegisterFont,
    pub remove_font: RemoveFont,
    pub register_overlay: RegisterOverlay,
    pub read_task_exif: ReadTaskExif,
    pub start_tasks: StartTasks,
    pub preview_task: PreviewTask,
    pub cancel_task: CancelTask,
    pub clear_tasks: ClearTasks,
    pub config: Arc<dyn ConfigRepository>,
    pub resources: Arc<dyn ResourceRepository>,
    pub tasks: Arc<dyn TaskQueue>,
    pub output: Arc<NativeOutputDirectory>,
}

impl AppState {
    /// Composes all inward-facing use cases and Rust-owned adapters.
    ///
    /// # Errors
    ///
    /// Returns a stable application error when native storage or adapters cannot initialize.
    pub fn compose(app: &AppHandle<Wry>) -> Result<Self, yiyin_application::ApplicationError> {
        let app_data = app
            .path()
            .app_data_dir()
            .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
        let app_cache = app
            .path()
            .app_cache_dir()
            .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
        fs::create_dir_all(&app_data).map_err(internal_io)?;
        fs::create_dir_all(&app_cache).map_err(internal_io)?;

        let legacy_base = app_data.parent().unwrap_or(&app_data);
        #[cfg(target_os = "macos")]
        let legacy_candidates = macos_candidates(legacy_base).to_vec();
        #[cfg(target_os = "windows")]
        let legacy_candidates = windows_candidates(legacy_base).to_vec();
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let legacy_candidates = Vec::new();

        let resource_root = app_data.join("resources");
        let config_repository: Arc<dyn ConfigRepository> = Arc::new(
            JsonConfigRepository::new(app_data.join("config.json")).with_legacy_import(
                LegacyImportOptions {
                    owned_resources_root: resource_root.clone(),
                    candidates: legacy_candidates,
                },
            ),
        );
        config_repository.import_legacy_if_needed()?;
        let config = config_repository.load()?;
        let output_root = resolve_output_root(app, &config.output)?;
        let output = Arc::new(NativeOutputDirectory::new(output_root)?);
        let resources = compose_resources(&resource_root)?;

        let bundled_fonts = write_bundled_fonts(&app_data.join("bundled-fonts"))?;
        let renderer = Arc::new(
            RustImageRenderer::with_shared_output_root_and_bundled_fonts(
                Arc::clone(&resources),
                output.shared_root(),
                app_cache.join("previews"),
                &bundled_fonts,
            )?,
        );
        let events = Arc::new(TauriTaskEventSink::new(app.clone()));
        let tasks: Arc<dyn TaskQueue> = Arc::new(TokioTaskQueue::new(renderer, events)?);
        let resource_repository: Arc<dyn ResourceRepository> = resources.clone();
        let ids = resources;
        let metadata = Arc::new(ExifMetadataReader);

        Ok(Self {
            bootstrap: Bootstrap::new(
                Arc::clone(&config_repository),
                Arc::clone(&resource_repository),
                Arc::clone(&tasks),
            ),
            update_config: UpdateConfig::new(Arc::clone(&config_repository)),
            reset_config: ResetConfig::new(Arc::clone(&config_repository)),
            register_images: RegisterImages::new(
                Arc::clone(&resource_repository),
                ids,
                Arc::clone(&tasks),
            ),
            register_font: RegisterFont::new(Arc::clone(&resource_repository)),
            remove_font: RemoveFont::new(Arc::clone(&resource_repository)),
            register_overlay: RegisterOverlay::new(Arc::clone(&resource_repository)),
            read_task_exif: ReadTaskExif::new(
                Arc::clone(&tasks),
                Arc::clone(&resource_repository),
                metadata.clone(),
            ),
            start_tasks: StartTasks::new(
                Arc::clone(&config_repository),
                Arc::clone(&resource_repository),
                metadata.clone(),
                output.clone(),
                Arc::clone(&tasks),
            ),
            preview_task: PreviewTask::new(
                Arc::clone(&config_repository),
                Arc::clone(&resource_repository),
                metadata,
                Arc::clone(&tasks),
            ),
            cancel_task: CancelTask::new(Arc::clone(&tasks)),
            clear_tasks: ClearTasks::new(Arc::clone(&tasks)),
            config: config_repository,
            resources: resource_repository,
            tasks,
            output,
        })
    }
}

fn resolve_output_root(
    app: &AppHandle<Wry>,
    configured: &str,
) -> Result<PathBuf, yiyin_application::ApplicationError> {
    let configured = PathBuf::from(configured);
    if configured.is_absolute() {
        return Ok(configured);
    }
    let home = app
        .path()
        .home_dir()
        .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
    Ok(home.join(configured))
}

fn compose_resources(
    resource_root: &Path,
) -> Result<Arc<ResourceRegistry>, yiyin_application::ApplicationError> {
    let resources = Arc::new(ResourceRegistry::new(resource_root)?);
    for (name, contents) in BUNDLED_IMAGES {
        register_bundled_image(resources.as_ref(), resource_root, name, contents)?;
    }
    Ok(resources)
}

fn register_bundled_image(
    resources: &ResourceRegistry,
    resource_root: &Path,
    name: &str,
    contents: &[u8],
) -> Result<(), yiyin_application::ApplicationError> {
    let directory = resource_root.join("bundled");
    fs::create_dir_all(&directory).map_err(internal_io)?;
    let destination = directory.join(name);
    if fs::read(&destination).map_or(true, |current| current != contents) {
        fs::write(&destination, contents).map_err(internal_io)?;
    }
    resources.register_bundled(&destination, name)?;
    Ok(())
}

fn write_bundled_fonts(root: &Path) -> Result<Vec<PathBuf>, yiyin_application::ApplicationError> {
    [
        (
            "chunfengkai.ttf",
            include_bytes!("../../assets/fonts/春风楷.ttf").as_slice(),
        ),
        (
            "qiantuxiaotu.ttf",
            include_bytes!("../../assets/fonts/千图小兔体.ttf").as_slice(),
        ),
        (
            "frederickathegreat.ttf",
            include_bytes!("../../assets/fonts/FrederickatheGreat.ttf").as_slice(),
        ),
        (
            "neoneon.otf",
            include_bytes!("../../assets/fonts/Neoneon.otf").as_slice(),
        ),
    ]
    .into_iter()
    .map(|(name, contents)| write_bundled_font(root, name, contents))
    .collect()
}

fn write_bundled_font(
    root: &Path,
    name: &str,
    contents: &[u8],
) -> Result<PathBuf, yiyin_application::ApplicationError> {
    fs::create_dir_all(root).map_err(internal_io)?;
    let destination = root.join(name);
    if fs::read(&destination).map_or(true, |current| current != contents) {
        fs::write(&destination, contents).map_err(internal_io)?;
    }
    Ok(destination)
}

fn internal_io(error: std::io::Error) -> yiyin_application::ApplicationError {
    yiyin_application::ApplicationError::internal(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_every_bundled_font_into_rust_owned_storage() {
        let directory = tempfile::tempdir().expect("create app data");

        let paths = write_bundled_fonts(directory.path()).expect("write bundled fonts");

        assert_eq!(paths.len(), 4);
        assert!(paths.iter().all(|path| path.starts_with(directory.path())));
        assert!(paths.iter().all(|path| fs::metadata(path).is_ok()));
    }

    #[test]
    fn registers_donation_images_and_every_vendor_logo_variant() {
        let directory = tempfile::tempdir().expect("create resource root");
        let resources = compose_resources(directory.path()).expect("compose resources");
        let names = resources
            .snapshot()
            .into_iter()
            .map(|record| record.display_name().to_owned())
            .collect::<std::collections::BTreeSet<_>>();

        assert_eq!(names.len(), 28);
        assert!(names.contains("zs-wx.jpg"));
        assert!(names.contains("zs-zfb.jpg"));
        for vendor in [
            "canon",
            "dji",
            "fujifilm",
            "hasselblad",
            "leica",
            "nikon",
            "olympus",
            "panasonic",
            "pentax",
            "ricoh",
            "sigma",
            "songdian",
            "sony",
        ] {
            assert!(names.contains(&format!("{vendor}-b.png")));
            assert!(names.contains(&format!("{vendor}-w.png")));
        }
    }
}
