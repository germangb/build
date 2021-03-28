use byteorder::{ReadBytesExt, LE};
use std::{io, io::Read, iter::ExactSizeIterator};

bitflags::bitflags! {
    pub struct WallStat: i16 {
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

#[derive(Debug)]
pub struct Wall {
    // wall position of the left side of the wall
    pub x: i32,
    pub y: i32,

    // next wall index (-1 if none) in the same sector.
    // always to the right.
    point2: i16,
    // pointer to next one although this one might not be in the same sector.
    // used for global iterator of walls.
    next: i16,

    /// Wall attribute flags.
    pub wall_stat: WallStat,

    /// Sector connected to this wall.
    pub next_sector: i16,

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
    pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Self, io::Error> {
        #[cfg(feature = "v7")]
        Ok(Self {
            x: reader.read_i32::<LE>()?,
            y: reader.read_i32::<LE>()?,
            point2: reader.read_i16::<LE>()?,
            next: reader.read_i16::<LE>()?,
            next_sector: reader.read_i16::<LE>()?,
            wall_stat: WallStat::from_bits(reader.read_i16::<LE>()?)
                .expect("Error parsing wall stat bits."),
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

#[derive(Debug)]
pub struct SectorWalls<'a> {
    pub(crate) len: usize,
    pub(crate) index: usize,
    pub(crate) first: usize,
    pub(crate) walls: &'a [Wall],
    pub(crate) curr: Option<usize>,
}

impl<'a> Iterator for SectorWalls<'a> {
    type Item = (&'a Wall, &'a Wall);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(curr) = self.curr {
            let left = &self.walls[curr];
            let right = &self.walls[left.point2 as usize];
            self.curr = if left.point2 as usize == self.first {
                None
            } else {
                Some(left.point2 as _)
            };
            self.index += 1;
            Some((left, right))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;
        (size, Some(size))
    }
}

impl ExactSizeIterator for SectorWalls<'_> {}

#[cfg(test)]
mod test {}
