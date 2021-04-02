#[cfg(feature = "v6")]
compile_error!("Feature flag 'v6' is not yet implemented.");

use crate::{player::*, sector::*, sprite::*};
use byteorder::{ReadBytesExt, LE};
use log::info;
use std::{
    fs::read,
    io::{Cursor, Read},
};
use thiserror::Error;

pub mod player;
pub mod sector;
pub mod sprite;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unsupported MAP file version: {0}")]
    UnsupportedVersion(i32),

    /// IO error.
    #[error("MAP IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// The contents of a *Build Engine* MAP file.
#[derive(Debug)]
pub struct Map {
    /// MAP file version.
    pub version: i32,

    /// Player starting information.
    pub player: Player,

    /// MAP file geometry.
    pub sectors: Sectors,

    /// MAP sprites.
    pub sprites: Vec<Sprite>,
}

/// Parse MAP file from a byte slice.
pub fn from_slice(slice: &[u8]) -> Result<Map, Error> {
    from_reader(&mut Cursor::new(slice))
}

/// Parse MAP file from a reader.
pub fn from_reader<R: Read>(reader: &mut R) -> Result<Map, Error> {
    // crate supports versions from 7 to 9.
    // according to some wiki, 8 and 9 are the same as version 7.
    let version = reader.read_i32::<LE>()?;
    info!("MAP file version: {}", version);
    match version {
        7 | 8 | 9 => {}
        version => return Err(Error::UnsupportedVersion(version)),
    }

    Ok(Map {
        version,
        player: Player::from_reader(reader)?,
        sectors: Sectors::from_reader(reader)?,
        sprites: sprite::from_reader(reader)?,
    })
}
