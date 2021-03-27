use byteorder::{ReadBytesExt, LE};
use std::io::Read;
use thiserror::Error;

#[cfg(feature = "v6")]
pub mod v6;
#[cfg(feature = "v7")]
pub mod v7;

/// Map parsing error types.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Unsupported MAP file version: {0}")]
    UnsupportedVersion(i32),

    /// IO error.
    #[error("MAP IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Data structure hoding contents of a MAP file.
#[derive(Debug)]
pub struct Map {
    /// MAP file version.
    pub version: i32,

    player: Player,
    sectors: Vec<()>,
    sprites: Vec<()>,
    walls: Vec<v7::wall::Wall>,
}

impl Map {
    /// Create a map from a reader from a MAP file.
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            version: Self::read_version(reader)?,
            player: Player::from_reader(reader)?,
            sectors: todo!(),
            sprites: todo!(),
            walls: Self::read_walls(reader)?,
        })
    }

    fn read_version<R: Read>(reader: &mut R) -> Result<i32, Error> {
        match reader.read_i32::<LE>()? {
            version @ 7 | version @ 8 | version @ 9 => Ok(version),
            version => Err(Error::UnsupportedVersion(version)),
        }
    }

    fn read_sectors<R: Read>(reader: &mut R) -> Result<Vec<()>, Error> {
        todo!()
    }

    fn read_walls<R: Read>(reader: &mut R) -> Result<Vec<v7::wall::Wall>, Error> {
        todo!()
    }

    #[cfg(todo)]
    /// Returns an iterator over the sectors of this MAP file.
    pub fn sectors(&self) -> impl Iterator<Item = ()> {
        todo!()
    }

    /// Returns an iterator over the walls of this MAP file.
    pub fn walls(&self) -> v7::wall::Walls<'_> {
        v7::wall::Walls {
            same_sector: false,
            walls: &self.walls[..],
            curr: self.walls.first().map(|_| 0),
        }
    }

    /// Returns the player starting information (position, angle, and sector
    /// index).
    pub fn player(&self) -> &Player {
        &self.player
    }
}

/// Player starting position/orientation and sector information.
#[derive(Debug)]
pub struct Player {
    // position
    pub pos_x: i32,
    pub pos_y: i32,
    pub pos_z: i32,

    // orientation
    // sector index
    pub angle: i16,

    // starting sector index
    pub sector: i16,
}

impl Player {
    fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            pos_x: reader.read_i32::<LE>()?,
            pos_y: reader.read_i32::<LE>()?,
            pos_z: reader.read_i32::<LE>()?,
            angle: reader.read_i16::<LE>()?,
            sector: reader.read_i16::<LE>()?,
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn player() {
        todo!()
    }
}
