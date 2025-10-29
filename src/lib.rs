use bincode::{Decode, Encode};
use bytes::Bytes;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{DirBuilder, DirEntry, File},
    io::{self, Seek},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
    result::Result as StdResult,
    time::SystemTime,
};
use thiserror::Error as ThisError;
use uuid::{Builder, Timestamp, Uuid};

#[derive(Clone, Copy, Debug, Default)]
pub enum OpenOptions {
    #[default]
    ReadWrite,
}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("io error")]
    Io(#[from] io::Error),

    #[error("encode error")]
    Encode(#[from] bincode::error::EncodeError),

    #[error("decode error")]
    Decode(#[from] bincode::error::DecodeError),
}

type Result<T> = std::result::Result<T, Error>;

#[repr(C)]
struct Log<T: AsRef<[u8]>> {
    // TODO: Add CRC
    timestamp: i64,
    key: T,
    value: Option<T>,
}

impl<T: AsRef<[u8]>> Encode for Log<T> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> std::result::Result<(), bincode::error::EncodeError> {
        Encode::encode(&self.timestamp, encoder)?;
        Encode::encode(self.key.as_ref(), encoder)?;
        Encode::encode(&self.value.as_ref().map(AsRef::as_ref), encoder)?;
        Ok(())
    }
}

impl<Context, T> Decode<Context> for Log<T>
where
    T: From<Vec<u8>> + AsRef<[u8]>,
{
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> std::result::Result<Self, bincode::error::DecodeError> {
        let timestamp = Decode::decode(decoder)?;
        let key: Vec<u8> = Decode::decode(decoder)?;
        let value: Option<Vec<u8>> = Decode::decode(decoder)?;

        Ok(Self {
            timestamp,
            key: T::from(key),
            value: value.map(T::from),
        })
    }
}

struct KeyDirEntry {
    /// The path to the file datafile.
    file_id: FileId,

    // TODO: Store onnly the position of the value here.
    /// The start position of the log.
    pos: u64,

    /// The size of the log.
    size: u64,

    /// The stored timestamp.
    timestamp: i64,
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

    fn put(&mut self, key: Bytes, entry: KeyDirEntry) {
        self.keys.insert(key, entry);
    }

    fn delete<T>(&mut self, key: T) -> Option<KeyDirEntry>
    where
        T: AsRef<[u8]>,
    {
        self.keys.remove(key.as_ref())
    }
}

/// Represents a Bitcask handle.
pub struct Bitcask {
    file: File,
    file_id: FileId,
    keydir: KeyDir,
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
        DirBuilder::new().recursive(true).create(dir.as_ref())?;

        let file_id = get_latest_datafile_id(&dir).unwrap_or(FileId::new());

        let file = File::options()
            .read(true)
            .append(true)
            .create(true)
            .open(dir.as_ref().join(file_id.datafile_path()))?;

        file.lock()?;

        let keydir = KeyDir::new();

        let bitcask = Self {
            file,
            file_id,
            keydir,
        };

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

    pub fn put(&mut self, key: Bytes, value: Bytes) -> Result<()> {
        let (size, pos, timestamp) = self.append(key.clone(), Some(value))?;

        self.sync()?;

        let entry = KeyDirEntry {
            file_id: self.file_id,
            timestamp,
            pos,
            size,
        };

        self.keydir.put(key, entry);

        Ok(())
    }

    pub fn get<T>(&mut self, key: T) -> Result<Option<Bytes>>
    where
        T: AsRef<[u8]>,
    {
        let Some(entry) = self.keydir.get(&key) else {
            return Ok(None);
        };

        let file = if entry.file_id == self.file_id {
            &mut self.file
        } else {
            &mut File::options()
                .read(true)
                .open(&entry.file_id.datafile_path())?
        };

        let mut bytes = vec![0; 4 * 30 + key.as_ref().len() + entry.size as usize];
        file.read_at(&mut bytes, entry.pos)?;

        let (log, _): (Log<Bytes>, usize) =
            bincode::decode_from_slice(&bytes, bincode::config::standard())?;

        Ok(log.value)
    }

    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        let Some(_) = self.keydir.get(key) else {
            return Ok(());
        };

        let _ = self.append(key, None)?;

        self.keydir.delete(key);

        Ok(())
    }

    pub fn append<T: AsRef<[u8]>>(&mut self, key: T, value: Option<T>) -> Result<(u64, u64, i64)> {
        let pos = self.file.stream_position()?;
        let timestamp = timestamp();

        let log: Log<&[u8]> = Log {
            key: key.as_ref(),
            value: value.as_ref().map(AsRef::as_ref),
            timestamp,
        };

        let size =
            bincode::encode_into_std_write(log, &mut self.file, bincode::config::standard())?;

        Ok((size as u64, pos, timestamp))
    }
}

fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("system time before Unix epoch")
        .as_nanos() as i64
}

fn get_latest_datafile_id(dir: impl AsRef<Path>) -> Option<FileId> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(StdResult::ok)
        .filter_map(|entry| {
            let filename = entry.file_name();
            let id = filename.to_str()?.strip_suffix(".bitcask.data")?;
            id.try_into().ok()
        })
        .max()
}

/// A sortable file identifier.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct FileId {
    id: Uuid,
}

impl FileId {
    fn new() -> Self {
        // TODO: Use UUIDv7 later
        Self { id: Uuid::now_v7() }
    }

    fn datafile_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.bitcask.data", self.id))
    }

    fn hintfile_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.bitcask.hint", self.id))
    }
}

impl<'a> TryFrom<&'a str> for FileId {
    type Error = io::Error;

    fn try_from(value: &'a str) -> StdResult<Self, Self::Error> {
        Ok(Self {
            id: value.parse().or(Err(io::ErrorKind::InvalidFilename))?,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
