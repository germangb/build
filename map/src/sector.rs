use crate::wall::{SectorWalls, Wall};
use byteorder::{ReadBytesExt, LE};
use std::{io, io::Read};

bitflags::bitflags! {
    pub struct SectorStat: u16 {
        const PARALLAXING                 = 0b0000_0000_0000_0001;
        const SLOPPED                     = 0b0000_0000_0000_0010;
        const SWAP_X_Y                    = 0b0000_0000_0000_0100;
        const DOUBLE_SMOOSHINESS          = 0b0000_0000_0000_1000;
        const X_FLIP                      = 0b0000_0000_0001_0000;
        const Y_FLIP                      = 0b0000_0000_0010_0000;
        const ALIGN_TEXTURE_TO_FIRST_WALL = 0b0000_0000_0100_0000;
        const RESERVED                    = 0b1111_1111_1000_0000;
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

impl Sector {
    pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Self, io::Error> {
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
    pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Self, io::Error> {
        let sectors = Self::read_sectors(reader)?;
        let walls = Self::read_walls(reader)?;
        Ok(Self { sectors, walls })
    }

    fn read_sectors<R: Read>(reader: &mut R) -> Result<Vec<Sector>, io::Error> {
        let num_sectors = reader.read_u16::<LE>()? as usize;
        (0..num_sectors)
            .map(|_| Sector::from_reader(reader))
            .collect::<Result<Vec<_>, _>>()
    }

    #[rustfmt::skip]
    fn read_walls<R: Read>(reader: &mut R) -> Result<Vec<Wall>, io::Error> {
        let num_walls = reader.read_u16::<LE>()? as usize;
        (0..num_walls).map(|_| Wall::from_reader(reader))
            .collect::<Result<Vec<_>, _>>()
    }

    /// Return a sector and an iterator over the sector's walls.
    pub fn get(&self, sector: usize) -> Option<(&Sector, SectorWalls<'_>)> {
        self.sectors
            .get(sector)
            .map(|s| (s, self.sector_walls(sector)))
    }

    /// Returns a slice of [`Sector`](Sector) in the same order from the source
    /// MAP file, to allow random access.
    pub fn as_slice(&self) -> &[Sector] {
        self.sectors.as_slice()
    }

    /// Returns walls in the same order as in the MAP file to allow random
    /// access. To know which walls correspond to which sectors, use the
    /// [`Sectors::get`](Sectors::get) method.
    pub fn walls_as_slice(&self) -> &[Wall] {
        self.walls.as_slice()
    }

    fn sector_walls(&self, sector: usize) -> SectorWalls<'_> {
        let first = self.sectors[sector].wallptr as _;
        let len = self.sectors[sector].wallnum as _;
        SectorWalls {
            len,
            index: 0,
            first,
            walls: self.walls.as_slice(),
            curr: Some(first),
        }
    }
}

#[cfg(test)]
mod test {}
