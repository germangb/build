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
    pub angle: Angle,

    /// starting sector index.
    pub sector: i16,
}

impl Player {
    pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            pos_x: reader.read_i32::<LE>()?,
            pos_y: reader.read_i32::<LE>()?,
            pos_z: reader.read_i32::<LE>()?,
            angle: Angle(reader.read_i16::<LE>()?),
            sector: reader.read_i16::<LE>()?,
        })
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Angle(pub i16);

impl Angle {
    pub fn to_radians(&self) -> f32 {
        // All angles are between 0..2047 inclusive. 0 is "north", parallel to the
        // Y-axis, moving away from the X-axis. 512 is "east", parallel to the X-axis
        // moving away from the Y-axis.
        const PI2: f64 = std::f64::consts::PI * 2.0;
        const RANGE: i16 = 0x7ff;
        let a = (self.0 & RANGE) as f64 / (RANGE as f64) * PI2 - std::f64::consts::PI / 2.0;
        a as _
    }
}
