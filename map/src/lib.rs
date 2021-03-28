#![deny(unused)]
use crate::{sector::*, wall::*};
use byteorder::{ReadBytesExt, LE};
use std::{
    io,
    io::{Cursor, Read},
};
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
            sectors: Sectors {
                sectors: Self::read_sectors(reader)?,
                walls: Self::read_walls(reader)?,
            },
        })
    }

    pub fn sectors(&self) -> &Sectors {
        &self.sectors
    }

    fn read_version<R: Read>(reader: &mut R) -> Result<i32, Error> {
        match reader.read_i32::<LE>()? {
            version @ 7 | version @ 8 | version @ 9 => Ok(version),
            version => Err(Error::UnsupportedVersion(version)),
        }
    }

    fn read_sectors<R: Read>(reader: &mut R) -> Result<Vec<Sector>, io::Error> {
        let num_sectors = reader.read_u16::<LE>()? as usize;
        let mut sectors = Vec::with_capacity(num_sectors);
        for _ in 0..num_sectors {
            sectors.push(Sector::from_reader(reader)?);
        }
        Ok(sectors)
    }

    fn read_walls<R: Read>(reader: &mut R) -> Result<Vec<Wall>, io::Error> {
        let num_walls = reader.read_u16::<LE>()? as usize;
        let mut walls = Vec::with_capacity(num_walls);
        for _ in 0..num_walls {
            walls.push(Wall::from_reader(reader)?);
        }
        Ok(walls)
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
