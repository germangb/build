use crate::Error;
use byteorder::{ReadBytesExt, LE};
use std::io::Read;

pub type SectorId = i16;

bitflags::bitflags! {
    pub struct SectorStat: u16 {
        const PARALLAXING                 = 0b0000_0000_0000_0001;
        const SLOPPED                     = 0b0000_0000_0000_0010;
        const SWAP_X_Y                    = 0b0000_0000_0000_0100;
        const DOUBLE_SMOOSHINESS          = 0b0000_0000_0000_1000;
        const X_FLIP                      = 0b0000_0000_0001_0000;
        const Y_FLIP                      = 0b0000_0000_0010_0000;
        const ALIGN_TEXTURE_TO_FIRST_WALL = 0b0000_0000_0100_0000;
        #[doc(hidden)]
        const RESERVED                    = 0b1111_1111_1000_0000;
    }
}

bitflags::bitflags! {
    pub struct WallStat: u16 {
        /// Blocking wall (used with clipmove, getzrange).
        const BLOCKING_CLIPMOVE_GETZRANGE    = 0b0000_0000_0000_0001;
        const BOTTOMS_SWAPPED                = 0b0000_0000_0000_0010;
        const ALIGN_PICTURE_ON_BOTTOM        = 0b0000_0000_0000_0100;
        const X_FLIPPED                      = 0b0000_0000_0000_1000;
        const MASKING_WALL                   = 0b0000_0000_0001_0000;
        const ONE_WAY_WALL                   = 0b0000_0000_0010_0000;

        /// Blocking wall (used with hitscan / cliptype 1).
        const BLOCKING_WALL_HITSCAN_CLIPTYPE = 0b0000_0000_0100_0000;
        const TRANSLUCENCE                   = 0b0000_0000_1000_0000;
        const Y_FLIPPED                      = 0b0000_0001_0000_0000;
        const TRANSLUCENCE_REVERSING         = 0b0000_0010_0000_0000;
        #[doc(hidden)]
        const RESERVED                       = 0b1111_1100_0000_0000;
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Sector {
    // wall pointer and # of walls in the sector (in wall units)
    wallptr: u16,
    wallnum: u16,

    /// Z-coordinate (height) of ceiling at first point of sector.
    pub ceiling_z: i32,

    /// Z-coordinate (height) of floor at first point of sector.
    pub floor_z: i32,

    pub ceiling_stat: SectorStat,
    pub floor_stat: SectorStat,

    // ceiling & floor texturing
    pub ceiling_picnum: i16,

    /// Slope value (rise/run; 0 = parallel to floor, 4096 = 45 degrees).
    pub ceiling_heinum: i16,

    pub ceiling_shade: i8,
    pub ceiling_pal: u8,
    pub ceiling_xpanning: u8,
    pub ceiling_ypanning: u8,

    pub floor_picnum: i16,
    pub floor_heinum: i16,
    pub floor_shade: i8,
    pub floor_pal: u8,
    pub floor_xpanning: u8,
    pub floor_ypanning: u8,

    /// How fast an area changes shade relative to distance.
    pub visibility: u8,

    filler: [u8; 1],

    // game-specific data
    pub lotag: i16,
    pub hitag: i16,
    pub extra: i16,
}

#[derive(Debug)]
#[repr(C)]
pub struct Wall {
    // wall position of the left side of the wall
    pub x: i32,
    pub y: i32,

    // next wall index (-1 if none) in the same sector.
    // always to the right.
    pub point2: i16,

    /// Index to wall on other side of wall (-1 if there is no sector there).
    pub next_wall: i16,

    /// Index to sector on other side of wall (-1 if there is no sector).
    pub next_sector: i16,

    /// Wall attribute flags.
    pub wall_stat: WallStat,

    // texturing & sampling parameters
    pub picnum: i16,
    pub over_picnum: i16,
    pub shade: i8,
    pub pal: u8,

    // Change pixel size to stretch/shrink textures
    pub x_repeat: u8,
    pub y_repeat: u8,

    // Offset for aligning textures
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
            point2: reader.read_i16::<LE>()?,
            next_wall: reader.read_i16::<LE>()?,
            next_sector: reader.read_i16::<LE>()?,
            wall_stat: WallStat::from_bits(reader.read_u16::<LE>()?)
                .expect("Error parsing wall stat bits."),
            picnum: reader.read_i16::<LE>()?,
            over_picnum: reader.read_i16::<LE>()?,
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

impl Sector {
    pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            wallptr: reader.read_u16::<LE>()?,
            wallnum: reader.read_u16::<LE>()?,
            ceiling_z: reader.read_i32::<LE>()?,
            floor_z: reader.read_i32::<LE>()?,
            ceiling_stat: SectorStat::from_bits(reader.read_u16::<LE>()?)
                .expect("Error parsing ceiling stat bits."),
            floor_stat: SectorStat::from_bits(reader.read_u16::<LE>()?)
                .expect("Error parsing floor stat bits."),
            ceiling_picnum: reader.read_i16::<LE>()?,
            ceiling_heinum: reader.read_i16::<LE>()?,
            ceiling_shade: reader.read_i8()?,
            ceiling_pal: reader.read_u8()?,
            ceiling_xpanning: reader.read_u8()?,
            ceiling_ypanning: reader.read_u8()?,
            floor_picnum: reader.read_i16::<LE>()?,
            floor_heinum: reader.read_i16::<LE>()?,
            floor_shade: reader.read_i8()?,
            floor_pal: reader.read_u8()?,
            floor_xpanning: reader.read_u8()?,
            floor_ypanning: reader.read_u8()?,
            visibility: reader.read_u8()?,
            filler: [reader.read_u8()?],
            lotag: reader.read_i16::<LE>()?,
            hitag: reader.read_i16::<LE>()?,
            extra: reader.read_i16::<LE>()?,
        })
    }
}

#[derive(Debug)]
pub struct Sectors {
    sectors: Vec<Sector>,
    walls: Vec<Wall>,
}

impl Sectors {
    pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let sectors = Self::read_sectors(reader)?;
        let walls = Self::read_walls(reader)?;
        Ok(Self { sectors, walls })
    }

    fn read_sectors<R: Read>(reader: &mut R) -> Result<Vec<Sector>, Error> {
        let num_sectors = reader.read_u16::<LE>()? as usize;
        (0..num_sectors)
            .map(|_| Sector::from_reader(reader))
            .collect::<Result<Vec<_>, _>>()
    }

    fn read_walls<R: Read>(reader: &mut R) -> Result<Vec<Wall>, Error> {
        let num_walls = reader.read_u16::<LE>()? as usize;
        (0..num_walls)
            .map(|_| Wall::from_reader(reader))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Return a sector and an iterator over the sector's walls.
    pub fn get(&self, sector: SectorId) -> Option<(&Sector, SectorWalls<'_>)> {
        if sector < 0 {
            None
        } else {
            self.sectors
                .get(sector as usize)
                .map(|s| (s, self.sector_walls(sector)))
        }
    }

    /// Returns a slice of [`Sector`](Sector) in the same order from the source
    /// MAP file, to allow random access.
    pub fn sectors(&self) -> &[Sector] {
        self.sectors.as_slice()
    }

    /// Returns walls in the same order as in the MAP file to allow random
    /// access. To know which walls correspond to which sectors, use the
    /// [`Sectors::get`](Sectors::get) method.
    pub fn walls(&self) -> &[Wall] {
        self.walls.as_slice()
    }

    fn sector_walls(&self, sector: SectorId) -> SectorWalls<'_> {
        assert_ne!(-1, sector);
        let first = self.sectors[sector as usize].wallptr as _;
        let len = self.sectors[sector as usize].wallnum as _;
        SectorWalls {
            len,
            index: 0,
            first,
            walls: self.walls.as_slice(),
            curr: Some(first),
        }
    }
}

#[derive(Debug)]
pub struct SectorWalls<'a> {
    len: usize,
    index: usize,
    first: usize,
    walls: &'a [Wall],
    curr: Option<usize>,
}

impl<'a> Iterator for SectorWalls<'a> {
    type Item = (SectorId, &'a Wall, &'a Wall);

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.curr?;
        let left = &self.walls[curr];
        let right = &self.walls[left.point2 as usize];
        self.index += 1;
        self.curr = if left.point2 as usize == self.first {
            None
        } else {
            Some(left.point2 as _)
        };
        Some((curr as _, left, right))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len - self.index;
        (size, Some(size))
    }
}

impl ExactSizeIterator for SectorWalls<'_> {}

#[cfg(test)]
mod test {}
