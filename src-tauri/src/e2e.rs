use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::state::AppState;

const FIXTURE_DIRECTORY_ENV: &str = "YIYIN_E2E_FIXTURE_DIR";

/// Registers CI-provided images through the production application use case.
///
/// # Errors
///
/// Returns a stable application error when the fixture directory is missing,
/// unreadable, or contains no supported image files.
pub fn seed_registered_images(state: &AppState) -> Result<(), yiyin_application::ApplicationError> {
    let root = env::var_os(FIXTURE_DIRECTORY_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| {
            yiyin_application::ApplicationError::internal(
                "the E2E fixture directory environment variable is missing",
            )
        })?;
    let paths = fixture_paths(&root)?;
    state.register_images.execute(&paths)?;
    Ok(())
}

fn fixture_paths(root: &Path) -> Result<Vec<PathBuf>, yiyin_application::ApplicationError> {
    let entries = fs::read_dir(root)
        .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?
            .path();
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase);
        if path.is_file() && matches!(extension.as_deref(), Some("jpg" | "jpeg" | "png" | "webp")) {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(yiyin_application::ApplicationError::invalid_request(
            "The E2E fixture directory has no supported images.",
        ));
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_only_supported_images_in_stable_order() {
        let directory = tempfile::tempdir().expect("create fixture directory");
        for name in ["second.WEBP", "ignored.txt", "first.jpg", "third.png"] {
            fs::write(directory.path().join(name), b"fixture").expect("write fixture");
        }

        let paths = fixture_paths(directory.path()).expect("select fixture images");
        let names = paths
            .iter()
            .filter_map(|path| path.file_name()?.to_str())
            .collect::<Vec<_>>();

        assert_eq!(names, ["first.jpg", "second.WEBP", "third.png"]);
    }
}
