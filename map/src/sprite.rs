use crate::Error;
use byteorder::{ReadBytesExt, LE};
use std::io::Read;

bitflags::bitflags! {
    pub struct SpriteStat: u16 {
        /// Blocking sprite (used with clipmove, getzrange).
        const BLOCKING_SPRITE                  = 0b0000_0000_0000_0001;
        const TRANSLUCENCE                     = 0b0000_0000_0000_0010;
        const X_FLIPPED                        = 0b0000_0000_0000_0100;
        const Y_FLIPPED                        = 0b0000_0000_0000_1000;
        #[doc(hidden)]
        const RESERVED_SPRITE_TYPE             = 0b0000_0000_0011_0000;
        const ONE_SIDED                        = 0b0000_0000_0100_0000;
        const REAL_CENTERED_CENTERING          = 0b0000_0000_1000_0000;

        /// Blocking sprite (used with hitscan / cliptype 1).
        const BLOCKING_SPRITE_HITSCAN_CLIPTYPE = 0b0000_0001_0000_0000;
        const TRANSLUCENCE_REVERSING           = 0b0000_0010_0000_0000;
        #[doc(hidden)]
        const RESERVED                         = 0b0111_1100_0000_0000;
        const INVISIBLE                        = 0b1000_0000_0000_0000;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u16)]
pub enum SpriteType {
    /// FACE sprite.
    Face = 0b00 << 4,

    /// WALL sprite.
    Wall = 0b01 << 4,

    /// FLOOR sprite.
    Floor = 0b10 << 4,
}

#[derive(Debug)]
#[repr(C)]
pub struct Sprite {
    // position
    pub x: i32,
    pub y: i32,
    pub z: i32,

    pub sprite_stat: SpriteStat,

    /// Texture index into ART file.
    pub picnum: i16,

    /// Shade offset of wall.
    pub shade: i8,

    /// Palette lookup table number (0 = standard colours).
    pub pal: u8,

    /// Size of the movement clipping square (face sprites only).
    pub clip_dist: u8,

    filler: [u8; 1],

    pub x_repeat: u8,
    pub y_repeat: u8,
    pub x_offset: u8,
    pub y_offset: u8,

    /// Current sector of sprite's position.
    pub sectnum: i16,

    /// Current status of sprite (inactive, monster, bullet, etc.).
    pub statnum: i16,

    pub angle: i16,

    // undocumented
    pub owner: i16,
    pub x_vel: i16,
    pub y_vel: i16,
    pub z_vel: i16,

    // game-specific data
    pub lotag: i16,
    pub hitag: i16,
    pub extra: i16,
}

impl Sprite {
    fn from_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            x: reader.read_i32::<LE>()?,
            y: reader.read_i32::<LE>()?,
            z: reader.read_i32::<LE>()?,

            /// TODO(german): validate RESERVED_SPRITE_TYPE cannot be '0b11'
            sprite_stat: SpriteStat::from_bits(reader.read_u16::<LE>()?)
                .expect("Error parsing sprite stat bits."),
            picnum: reader.read_i16::<LE>()?,
            shade: reader.read_i8()?,
            pal: reader.read_u8()?,
            clip_dist: reader.read_u8()?,
            filler: [reader.read_u8()?],
            x_repeat: reader.read_u8()?,
            y_repeat: reader.read_u8()?,
            x_offset: reader.read_u8()?,
            y_offset: reader.read_u8()?,
            sectnum: reader.read_i16::<LE>()?,
            statnum: reader.read_i16::<LE>()?,
            angle: reader.read_i16::<LE>()?,
            owner: reader.read_i16::<LE>()?,
            x_vel: reader.read_i16::<LE>()?,
            y_vel: reader.read_i16::<LE>()?,
            z_vel: reader.read_i16::<LE>()?,
            lotag: reader.read_i16::<LE>()?,
            hitag: reader.read_i16::<LE>()?,
            extra: reader.read_i16::<LE>()?,
        })
    }

    /// Return the sprite type.
    pub fn sprite_type(&self) -> SpriteType {
        let stat = (self.sprite_stat.bits >> 4) & 0b11;
        match stat {
            0b00 | 0b01 | 0b10 => unsafe { std::mem::transmute(stat) },
            0b11 => panic!(),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn from_reader<R: Read>(reader: &mut R) -> Result<Vec<Sprite>, Error> {
    let num_sprites = reader.read_u16::<LE>()? as usize;
    (0..num_sprites)
        .map(|_| Sprite::from_reader(reader))
        .collect::<Result<Vec<_>, _>>()
}
