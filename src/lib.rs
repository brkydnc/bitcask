use bytes::Bytes;
use std::{
    fs::{DirBuilder, File},
    io,
    path::Path,
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
    crc: u32,
    timestamp: u32,
    ksize: u32,
    vsize: u32,
    bytes: [u8],
}

struct KeyDir {}

/// Represents a Bitcask handle.
pub struct Bitcask {
    file: File,
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

        let bitcask = Self { file };

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
        todo!();
    }

    pub fn get(&self, key: &[u8]) -> Result<Bytes> {
        todo!()
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
