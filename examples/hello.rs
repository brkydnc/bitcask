use std::error::Error;

use bitcask::{Bitcask, OpenOptions};
use bytes::Bytes;

fn main() -> Result<(), Box<dyn Error>> {
    let bitcask = Bitcask::open("data", OpenOptions::ReadWrite)?;

    let hello = Bytes::from_static(b"hello");
    let world = Bytes::from_static(b"world");

    bitcask.put(hello.clone(), world.clone())?;
    assert_eq!(bitcask.get(&hello)?, world);

    Ok(())
}
