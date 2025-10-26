use bytes::Bytes;
use chrono::TimeZone;
use std::{
    collections::{HashMap, HashSet},
    fs::{DirBuilder, File},
    io,
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
};
use thiserror::Error as ThisError;

#[derive(Clone, Copy, Debug, Default)]
pub enum OpenOptions {
    #[default]
    ReadWrite,
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[repr(C)]
struct Log {
    // TODO: Add CRC
    timestamp: i64,
    key: Bytes,
    value: Option<Bytes>,
}

struct KeyDirEntry {
    /// The path to the file datafile.
    datafile: PathBuf,

    /// The beginning position of the value bytes.
    vpos: usize,

    /// The size of the value.
    vsize: usize,

    /// The stored timestamp.
    timestamp: u32,
}

struct KeyDir {
    keys: HashMap<Bytes, KeyDirEntry>,
}

impl KeyDir {
    fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    fn get<T>(&self, key: T) -> Option<&KeyDirEntry>
    where
        T: AsRef<[u8]>,
    {
        self.keys.get(key.as_ref())
    }
}

/// Represents a Bitcask handle.
pub struct Bitcask {
    file: File,
    keydir: KeyDir,
}

impl Bitcask {
    const ACTIVE_DATAFILE_NAME: &str = "active.bitcask.data";

    /// Open a new or existing Bitcask datastore with additional options.
    ///
    /// Valid options include read write (if this process is going to be a
    /// writer and not just a reader) and sync on put (if this writer would
    /// prefer to sync the write file after every write operation).
    ///
    /// The directory must be readable and writable by this process, and
    /// only one process may open a Bitcask with read write at a time.
    pub fn open(dir: impl AsRef<Path>, opts: OpenOptions) -> Result<Self> {
        DirBuilder::new().recursive(true).create(dir.as_ref())?;

        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(dir.as_ref().join(Self::ACTIVE_DATAFILE_NAME))?;

        file.lock()?;

        let keydir = KeyDir::new();

        let bitcask = Self { file, keydir };

        Ok(bitcask)
    }

    /// TODO: Close a Bitcask data store and flush all pending writes
    /// (if any) to disk.
    pub fn close(self) -> Result<()> {
        self.file.sync_all()?;
        Ok(())
    }

    /// Force any writes to sync to disk.
    pub fn sync(&self) -> Result<()> {
        self.file.sync_data()?;
        Ok(())
    }

    pub fn merge(&mut self) -> Result<()> {
        todo!()
    }

    pub fn fold<F, A>(f: F, acc: A)
    where
        F: FnMut(Bytes, Bytes, A) -> A,
    {
        todo!()
    }

    pub fn list_keys(&self) -> Result<impl Iterator<Item = Bytes>> {
        Ok(std::iter::empty())
    }

    pub fn put(&self, key: Bytes, value: Bytes) -> Result<()> {
        let log = Log {
            timestamp: chrono::offset::Utc::now().timestamp(),
            key: key.clone(),
            value: Some(value),
        };

        Ok(())
    }

    pub fn get<T>(&self, key: T) -> Result<Option<Bytes>>
    where
        T: AsRef<[u8]>,
    {
        let Some(entry) = self.keydir.get(key) else {
            return Ok(None);
        };

        let file = File::options().read(true).open(&entry.datafile)?;
        let mut buf = Vec::with_capacity(entry.vsize);
        file.read_at(&mut buf, entry.vpos as u64)?;
        Ok(Some(Bytes::from_owner(buf)))
    }

    pub fn delete(&self, key: &[u8]) -> Result<Bytes> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
