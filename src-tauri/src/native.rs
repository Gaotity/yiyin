#![allow(
    clippy::needless_pass_by_value,
    reason = "I/O errors are consumed by map_err adapter functions"
)]

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use yiyin_application::{ApplicationError, OutputDirectoryGateway};
use yiyin_domain::OutputDirectory;

use crate::dto::ExternalDestinationDto;

pub struct NativeOutputDirectory {
    home: PathBuf,
    root: Arc<RwLock<PathBuf>>,
    reservations: Mutex<BTreeSet<String>>,
}

impl NativeOutputDirectory {
    /// Creates the initial output directory shared with the renderer,
    /// capturing the home directory used to resolve relative roots.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when the directory cannot be created.
    pub fn new(home: PathBuf, configured: &OutputDirectory) -> Result<Self, ApplicationError> {
        let root = resolve_output_root(&home, configured);
        fs::create_dir_all(&root).map_err(internal_io)?;
        Ok(Self {
            home,
            root: Arc::new(RwLock::new(root)),
            reservations: Mutex::new(BTreeSet::new()),
        })
    }

    #[must_use]
    pub fn shared_root(&self) -> Arc<RwLock<PathBuf>> {
        Arc::clone(&self.root)
    }

    /// Returns the current native output root.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when synchronized state is unavailable.
    pub fn root(&self) -> Result<PathBuf, ApplicationError> {
        self.root
            .read()
            .map(|root| root.clone())
            .map_err(|_| ApplicationError::internal("output root lock poisoned"))
    }

    fn resolve(&self, configured: &OutputDirectory) -> PathBuf {
        resolve_output_root(&self.home, configured)
    }
}

fn resolve_output_root(home: &Path, configured: &OutputDirectory) -> PathBuf {
    let path = PathBuf::from(configured.as_str());
    if path.is_absolute() {
        path
    } else {
        home.join(path)
    }
}

impl OutputDirectoryGateway for NativeOutputDirectory {
    fn existing_names(&self) -> Result<BTreeSet<String>, ApplicationError> {
        let root = self.root()?;
        let mut names = fs::read_dir(root)
            .map_err(internal_io)?
            .filter_map(Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect::<BTreeSet<_>>();
        names.extend(
            self.reservations
                .lock()
                .map_err(|_| ApplicationError::internal("output reservations lock poisoned"))?
                .iter()
                .cloned(),
        );
        Ok(names)
    }

    fn reserve(&self, file_name: &str) -> Result<(), ApplicationError> {
        if Path::new(file_name).components().count() != 1 {
            return Err(ApplicationError::file_invalid());
        }
        self.reservations
            .lock()
            .map_err(|_| ApplicationError::internal("output reservations lock poisoned"))?
            .insert(file_name.to_owned());
        Ok(())
    }

    fn release(&self, file_name: &str) -> Result<(), ApplicationError> {
        self.reservations
            .lock()
            .map_err(|_| ApplicationError::internal("output reservations lock poisoned"))?
            .remove(file_name);
        Ok(())
    }

    fn ensure_root(&self, root: &OutputDirectory) -> Result<(), ApplicationError> {
        fs::create_dir_all(self.resolve(root)).map_err(internal_io)
    }

    fn change_root(&self, root: &OutputDirectory) -> Result<(), ApplicationError> {
        let resolved = self.resolve(root);
        fs::create_dir_all(&resolved).map_err(internal_io)?;
        *self
            .root
            .write()
            .map_err(|_| ApplicationError::internal("output root lock poisoned"))? = resolved;
        self.reservations
            .lock()
            .map_err(|_| ApplicationError::internal("output reservations lock poisoned"))?
            .clear();
        Ok(())
    }
}

#[must_use]
pub const fn external_url(destination: ExternalDestinationDto) -> &'static str {
    match destination {
        ExternalDestinationDto::Repository => "https://github.com/ggchivalrous/yiyin",
        ExternalDestinationDto::Issues => "https://github.com/ggchivalrous/yiyin/issues",
        ExternalDestinationDto::CurrentRelease => {
            "https://github.com/ggchivalrous/yiyin/releases/tag/v1.6.0"
        }
        ExternalDestinationDto::BilibiliProfile => "https://space.bilibili.com/94829489",
        ExternalDestinationDto::BilibiliFeedback => {
            "https://message.bilibili.com/#/whisper/mid94829489"
        }
    }
}

fn internal_io(error: std::io::Error) -> ApplicationError {
    ApplicationError::internal(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_roots_resolve_against_the_captured_home() {
        let home = tempfile::tempdir().expect("home");
        let output = OutputDirectory::try_from("Pictures/watermark").expect("output directory");

        let directory = NativeOutputDirectory::new(home.path().to_path_buf(), &output)
            .expect("create output directory");

        assert_eq!(
            directory.root().expect("root"),
            home.path().join("Pictures/watermark")
        );
        assert!(home.path().join("Pictures/watermark").is_dir());
    }

    #[test]
    fn absolute_roots_pass_through() {
        let home = tempfile::tempdir().expect("home");
        let target = tempfile::tempdir().expect("target");
        let absolute = target.path().to_str().expect("utf-8 path");
        let output = OutputDirectory::try_from(absolute).expect("output directory");

        let directory = NativeOutputDirectory::new(home.path().to_path_buf(), &output)
            .expect("create output directory");

        assert_eq!(directory.root().expect("root"), target.path());
    }

    #[test]
    fn change_root_creates_swaps_and_drops_reservations() {
        let home = tempfile::tempdir().expect("home");
        let first = OutputDirectory::try_from("first").expect("output directory");
        let directory = NativeOutputDirectory::new(home.path().to_path_buf(), &first)
            .expect("create output directory");
        directory.reserve("photo-1.jpg").expect("reserve");

        let second = OutputDirectory::try_from("second").expect("output directory");
        directory.change_root(&second).expect("change root");

        assert_eq!(directory.root().expect("root"), home.path().join("second"));
        assert!(home.path().join("second").is_dir());
        assert!(
            !directory
                .existing_names()
                .expect("names")
                .contains("photo-1.jpg")
        );
    }
}
