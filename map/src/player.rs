use crate::Error;
use byteorder::{ReadBytesExt, LE};
use std::io::Read;

#[derive(Debug)]
#[repr(C)]
pub struct Player {
    // position
    pub pos_x: i32,
    pub pos_y: i32,
    pub pos_z: i32,

    /// Starting player orientation.
    pub angle: i16,

    /// starting sector index.
    pub sector: i16,
}

impl Player {
    pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            pos_x: reader.read_i32::<LE>()?,
            pos_y: reader.read_i32::<LE>()?,
            pos_z: reader.read_i32::<LE>()?,
            angle: reader.read_i16::<LE>()?,
            sector: reader.read_i16::<LE>()?,
        })
    }
}
