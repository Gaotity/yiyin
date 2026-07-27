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

use crate::dto::ExternalDestinationDto;

pub struct NativeOutputDirectory {
    root: Arc<RwLock<PathBuf>>,
    reservations: Mutex<BTreeSet<String>>,
}

impl NativeOutputDirectory {
    /// Creates the initial output directory shared with the renderer.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when the directory cannot be created.
    pub fn new(root: PathBuf) -> Result<Self, ApplicationError> {
        fs::create_dir_all(&root).map_err(internal_io)?;
        Ok(Self {
            root: Arc::new(RwLock::new(root)),
            reservations: Mutex::new(BTreeSet::new()),
        })
    }

    #[must_use]
    pub fn shared_root(&self) -> Arc<RwLock<PathBuf>> {
        Arc::clone(&self.root)
    }

    /// Updates the shared renderer and naming root.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when the directory or synchronized state is unavailable.
    pub fn set_root(&self, root: PathBuf) -> Result<(), ApplicationError> {
        fs::create_dir_all(&root).map_err(internal_io)?;
        *self
            .root
            .write()
            .map_err(|_| ApplicationError::internal("output root lock poisoned"))? = root;
        self.reservations
            .lock()
            .map_err(|_| ApplicationError::internal("output reservations lock poisoned"))?
            .clear();
        Ok(())
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
