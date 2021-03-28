use crate::sector::Sectors;
use byteorder::{ReadBytesExt, LE};
use std::io::{Cursor, Read};
use thiserror::Error;

pub mod sector;
pub mod sprite;
pub mod wall;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unsupported MAP file version: {0}")]
    UnsupportedVersion(i32),

    /// IO error.
    #[error("MAP IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct Map {
    /// MAP file version.
    pub version: i32,

    // position
    pub pos_x: i32,
    pub pos_y: i32,
    pub pos_z: i32,

    // orientation
    // sector index
    pub angle: i16,

    // starting sector index
    pub sector: i16,

    sectors: Sectors,
}

impl Map {
    pub fn from_slice(slice: &[u8]) -> Result<Self, Error> {
        Self::from_reader(&mut Cursor::new(slice))
    }

    /// Create a map from a reader from a MAP file.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            version: Self::read_version(reader)?,
            pos_x: reader.read_i32::<LE>()?,
            pos_y: reader.read_i32::<LE>()?,
            pos_z: reader.read_i32::<LE>()?,
            angle: reader.read_i16::<LE>()?,
            sector: reader.read_i16::<LE>()?,
            sectors: Sectors::from_reader(reader)?,
        })
    }

    pub fn sectors(&self) -> &Sectors {
        &self.sectors
    }

    fn read_version<R: Read>(reader: &mut R) -> Result<i32, Error> {
        match reader.read_i32::<LE>()? {
            7 => Ok(7),
            // according to the wiki, source ports use versions 8 and 9, but doesn't mention any
            // differences from version 7...
            8 => Ok(8),
            9 => Ok(9),
            version => Err(Error::UnsupportedVersion(version)),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[ignore]
    fn map() {
        todo!()
    }
}
