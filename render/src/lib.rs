//#![deny(unused)]

#[cfg(feature = "d2")]
pub mod d2;
#[cfg(feature = "d3")]
pub mod d3;
pub mod frame;

// TODO(german): delete me
bitflags::bitflags! {
    pub struct Input: u8 {
        const FORWARDS = 0b0000_0001;
        const SIDEWAYS = 0b0000_0010;
        const ANGULAR  = 0b0000_0100;
    }
}

/// Player update parameters.
#[derive(Debug)]
pub struct UpdateOpts {
    /// Linear forwards velocity.
    pub forwards: i32,

    /// Linear sideways velocity.
    pub sideways: i32,

    /// Rotation velocity
    pub rotate: i16,
}

impl Default for UpdateOpts {
    fn default() -> Self {
        Self {
            forwards: 32,
            sideways: 32,
            rotate: 8,
        }
    }
}

/// Update player's sector.
pub fn update(map: &mut map::Map, input: &Input, opts: &UpdateOpts) {
    if input.contains(Input::ANGULAR) {
        map.player.angle.0 += opts.rotate;
    }
    let mut x = 0;
    let mut y = 0;
    let forwards = opts.forwards as f32;
    let sideways = opts.sideways as f32;
    let sin = map.player.angle.to_radians().sin();
    let cos = map.player.angle.to_radians().cos();
    if input.contains(Input::FORWARDS) {
        let dx = -sin * forwards;
        let dy = cos * forwards;
        x += dx as i32;
        y += dy as i32;
    }
    if input.contains(Input::SIDEWAYS) {
        let dx = cos * sideways;
        let dy = sin * sideways;
        x -= dx as i32;
        y -= dy as i32;
    }
    // update player sector
    let (_, walls) = map.sectors.get(map.player.sector).unwrap();
    let px = map.player.pos_x;
    let py = map.player.pos_y;
    let tx = px + x;
    let ty = py + y;
    for (left, right) in walls {
        if left.next_sector != -1 && intrsect_movement_with_wall(left, right, [px, py], [tx, ty]) {
            map.player.sector = left.next_sector;
            break;
        }
    }
    map.player.pos_x += x;
    map.player.pos_y += y;
}

fn intrsect_movement_with_wall(
    left: &map::sector::Wall,
    right: &map::sector::Wall,
    [px, py]: [i32; 2],
    [tx, ty]: [i32; 2],
) -> bool {
    let lx = left.x;
    let ly = left.y;
    let rx = right.x;
    let ry = right.y;
    let num0 = (px - lx) * (ty - py) - (tx - px) * (py - ly);
    let num1 = (rx - lx) * (py - ly) - (px - lx) * (ry - ly);
    let den = (rx - lx) * (ty - py) - (tx - px) * (ry - ly);
    num0.abs() <= den.abs()
        && num1.abs() <= den.abs()
        && num0.signum() == den.signum()
        && num1.signum() == den.signum()
}
