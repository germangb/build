use crate::Error;
use byteorder::{ReadBytesExt, LE};
use std::{io::Read, iter::ExactSizeIterator, num::NonZeroI16};

bitflags::bitflags! {
    /// Wall flags (cstat)
    pub struct WallFlags: i16 {
        /// Blocking wall (used with clipmove, getzrange).
        const BLOCKING_CLIPMOVE_GETZRANGE       = 0b0000000001;
        const BOTTOMS_SWAPPED                   = 0b0000000010;
        const ALIGN_PICTURE_ON_BOTTOM           = 0b0000000100;
        const X_FLIPPED                         = 0b0000001000;
        const MASKING_WALL                      = 0b0000010000;
        const ONE_WAY_WALL                      = 0b0000100000;

        /// Blocking wall (used with hitscan / cliptype 1).
        const BLOCKING_WALL_HITSCAN_CLIPTYPEONE = 0b0001000000;
        const TRANSLUCENCE                      = 0b0010000000;
        const Y_FLIPPED                         = 0b0100000000;
        const TRANSLUCENCE_REVERSING            = 0b1000000000;
    }
}

/// MAP wall data.
#[derive(Debug)]
pub struct Wall {
    // wall position of the left side of the wall
    pub x: i32,
    pub y: i32,

    // next wall index (-1 if none) in the same sector.
    // always to the right.
    next_in_sector: i16,
    // pointer to next one although this one might not be in the same sector.
    // used for global iterator of walls.
    next: i16,

    /// Wall state flags.
    pub cstat: WallFlags,

    // texturing & sampling parameters
    pub picnum: i16,
    pub picnum_over: i16,
    pub shade: i8,
    pub pal: u8,
    pub x_repeat: u8,
    pub y_repeat: u8,
    pub x_panning: u8,
    pub y_panning: u8,

    // game-specific data
    pub lotag: i16,
    pub hitag: i16,
    pub extra: i16,
}

impl Wall {
    pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            x: reader.read_i32::<LE>()?,
            y: reader.read_i32::<LE>()?,
            next_in_sector: reader.read_i16::<LE>()?,
            next: reader.read_i16::<LE>()?,
            cstat: WallFlags::from_bits(reader.read_i16::<LE>()?)
                .expect("Error parsing wall bits."),
            picnum: reader.read_i16::<LE>()?,
            picnum_over: reader.read_i16::<LE>()?,
            shade: reader.read_i8()?,
            pal: reader.read_u8()?,
            x_repeat: reader.read_u8()?,
            y_repeat: reader.read_u8()?,
            x_panning: reader.read_u8()?,
            y_panning: reader.read_u8()?,
            lotag: reader.read_i16::<LE>()?,
            hitag: reader.read_i16::<LE>()?,
            extra: reader.read_i16::<LE>()?,
        })
    }
}

/// Iterator of Walls.
#[derive(Debug)]
pub struct Walls<'a> {
    pub(crate) same_sector: bool,
    pub(crate) walls: &'a [Wall],
    pub(crate) curr: Option<usize>,
}

impl<'a> Iterator for Walls<'a> {
    type Item = &'a Wall;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(curr) = self.curr {
            let wall = &self.walls[curr];
            let next = if self.same_sector {
                wall.next_in_sector
            } else {
                wall.next
            };
            self.curr = NonZeroI16::new(next).map(|n| n.get() as _);
            Some(wall)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        todo!()
    }
}

impl ExactSizeIterator for Walls<'_> {}
