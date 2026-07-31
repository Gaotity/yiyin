//! A `FileSystem` decorator that injects one failure at a chosen stage of the
//! durable publish sequence, shared by the persistence fault-injection suites.

#![allow(
    dead_code,
    reason = "each test binary consumes only the fault points it asserts on"
)]

use std::{
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use yiyin_infrastructure::{DirectoryEntry, FileSystem, StdFileSystem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPoint {
    WriteAndSync,
    Copy,
    Rename,
    SyncParent,
}

#[derive(Clone, Default)]
pub struct FaultyFileSystem {
    inner: StdFileSystem,
    fault: Arc<Mutex<Option<FaultPoint>>>,
}

impl FaultyFileSystem {
    pub fn fail_once(&self, point: FaultPoint) {
        *self.fault.lock().expect("fault lock") = Some(point);
    }

    fn check(&self, point: FaultPoint) -> io::Result<()> {
        let mut fault = self.fault.lock().expect("fault lock");
        if fault.as_ref() == Some(&point) {
            *fault = None;
            Err(io::Error::other(format!("injected {point:?}")))
        } else {
            Ok(())
        }
    }
}

impl FileSystem for FaultyFileSystem {
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
        self.check(FaultPoint::WriteAndSync)?;
        self.inner.write_and_sync(path, contents)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
        self.check(FaultPoint::Copy)?;
        self.inner.copy(from, to)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.check(FaultPoint::Rename)?;
        self.inner.rename(from, to)
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
        self.check(FaultPoint::SyncParent)?;
        self.inner.sync_parent(path)
    }
}
