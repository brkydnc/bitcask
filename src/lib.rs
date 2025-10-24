use bytes::Bytes;
use std::path::Path;
use thiserror::Error as ThisError;

#[derive(Clone, Copy, Debug, Default)]
enum OpenOptions {
    #[default]
    Read,
    ReadWrite,
}

#[derive(ThisError, Debug)]
enum Error {}

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
struct Bitcask {}

impl Drop for Bitcask {
    fn drop(&mut self) {}
}

impl Bitcask {
    /// Open a new or existing Bitcask datastore with additional options.
    ///
    /// Valid options include read write (if this process is going to be a
    /// writer and not just a reader) and sync on put (if this writer would
    /// prefer to sync the write file after every write operation).
    ///
    /// The directory must be readable and writable by this process, and
    /// only one process may open a Bitcask with read write at a time.
    pub fn open(dir: impl AsRef<Path>, opts: OpenOptions) -> Result<Self> {
        todo!()
    }

    /// TODO: Close a Bitcask data store and flush all pending writes
    /// (if any) to disk.
    pub fn close(self) -> Result<()> {
        todo!()
    }

    /// Force any writes to sync to disk.
    pub fn sync(&self) -> Result<()> {
        todo!()
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

    pub fn put<K, V>(&self, key: Bytes, value: Bytes) -> Result<()> {
        todo!();
    }

    pub fn get<K>(&mut self, key: K) -> Result<Bytes>
    where
        K: AsRef<[u8]>,
    {
        todo!()
    }

    pub fn delete<K>(&mut self, key: K) -> Result<Bytes>
    where
        K: AsRef<[u8]> + Send + 'static,
    {
        todo!()
    }
}

impl Bitcask {}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
