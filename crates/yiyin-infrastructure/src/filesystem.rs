use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectoryEntryKind {
    File,
    Directory,
    Symlink,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoryEntry {
    path: PathBuf,
    kind: DirectoryEntryKind,
}

impl DirectoryEntry {
    #[must_use]
    pub const fn new(path: PathBuf, kind: DirectoryEntryKind) -> Self {
        Self { path, kind }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub const fn kind(&self) -> DirectoryEntryKind {
        self.kind
    }
}

#[allow(
    clippy::missing_errors_doc,
    reason = "filesystem implementations preserve the concrete I/O error contract"
)]
pub trait FileSystem: Send + Sync {
    fn exists(&self, path: &Path) -> bool;
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
    fn entry_kind(&self, path: &Path) -> io::Result<DirectoryEntryKind>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write_and_sync(&self, path: &Path, contents: &[u8]) -> io::Result<()>;
    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;
    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>>;
    fn sync_parent(&self, path: &Path) -> io::Result<()>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        fs::canonicalize(path)
    }

    fn entry_kind(&self, path: &Path) -> io::Result<DirectoryEntryKind> {
        let file_type = fs::symlink_metadata(path)?.file_type();
        Ok(if file_type.is_file() {
            DirectoryEntryKind::File
        } else if file_type.is_dir() {
            DirectoryEntryKind::Directory
        } else if file_type.is_symlink() {
            DirectoryEntryKind::Symlink
        } else {
            DirectoryEntryKind::Other
        })
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn write_and_sync(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        file.write_all(contents)?;
        file.flush()?;
        file.sync_all()
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
        fs::copy(from, to)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::remove_dir_all(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>> {
        fs::read_dir(path)?
            .map(|entry| {
                let entry = entry?;
                let file_type = entry.file_type()?;
                let kind = if file_type.is_file() {
                    DirectoryEntryKind::File
                } else if file_type.is_dir() {
                    DirectoryEntryKind::Directory
                } else if file_type.is_symlink() {
                    DirectoryEntryKind::Symlink
                } else {
                    DirectoryEntryKind::Other
                };
                Ok(DirectoryEntry::new(entry.path(), kind))
            })
            .collect()
    }

    fn sync_parent(&self, path: &Path) -> io::Result<()> {
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no parent"))?;
        match File::open(parent).and_then(|directory| directory.sync_all()) {
            Ok(()) => Ok(()),
            #[cfg(windows)]
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::InvalidInput | io::ErrorKind::PermissionDenied
                ) =>
            {
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}
