use std::time::Duration;

/// Player update parameters.
#[derive(Debug, Default)]
pub struct UpdateOpts {
    /// Linear forwards velocity.
    pub forwards: i32,

    /// Linear sideways velocity.
    pub sideways: i32,

    /// Rotation velocity
    pub rotate: i16,
}

bitflags::bitflags! {
    pub struct Input: u16 {
        const FORWARDS   = 0b0000_0000_0001;
        const BACKWARDS  = 0b0000_0000_0010;
        const RIGHT      = 0b0000_0000_0100;
        const LEFT       = 0b0000_0000_1000;
        const UP         = 0b0000_0001_0000;
        const DOWN       = 0b0000_0010_0000;
        const LOOK_RIGHT = 0b0000_0100_0000;
        const LOOK_LEFT  = 0b0000_1000_0000;
        const CROUCH     = 0b0001_0000_0000;
    }
}

/// Very basic player controller
#[derive(Debug)]
pub struct InputController {
    pub max_speed: i32,
    pub fly: bool,
    eye_height: i32,
    opts: UpdateOpts,
}

impl InputController {
    pub fn new(map: &map::Map) -> Self {
        let player_sector = map.player.sector;
        let eye_height = map.player.pos_z - map.sectors.get(player_sector).unwrap().0.floor_z;
        Self {
            max_speed: 32,
            fly: false,
            eye_height,
            opts: UpdateOpts::default(),
        }
    }

    /// Update controller
    #[rustfmt::skip]
    pub fn update(&mut self, input: &Input, delta: Duration, map: &mut map::Map) {
        self.update_opts(input, delta);
        update_player(map, &self.opts);
        self.update_eye_height(input, delta, map);
    }

    #[rustfmt::skip]
    fn update_eye_height(&mut self, input: &Input, duration: Duration, map: &mut map::Map) {
        let sector = &map.sectors.sectors()[map.player.sector as usize];
        if self.fly {
            if input.contains(Input::UP) { map.player.pos_z -= 500; }
            if input.contains(Input::DOWN) { map.player.pos_z += 500; }
            map.player.pos_z = map.player.pos_z.min(sector.floor_z).max(sector.ceiling_z);
        } else {
            let mut target_z = sector.floor_z + self.eye_height;
            if input.contains(Input::CROUCH) {
                target_z -= self.eye_height / 2;
            }
            map.player.pos_z += (target_z - map.player.pos_z) >> 1;
        }
    }

    #[rustfmt::skip]
    fn update_opts(&mut self, input: &Input, duration: Duration) {
        let opts = &mut self.opts;
        if input.contains(Input::LOOK_RIGHT) || input.contains(Input::LOOK_LEFT) {
            opts.rotate += 2;
            if input.contains(Input::LOOK_LEFT) {
                opts.rotate -= 4;
            }
        } else {
            if opts.rotate > 0 { opts.rotate -= 1; }
            if opts.rotate < 0 { opts.rotate += 1; }
        }
        if input.contains(Input::FORWARDS) || input.contains(Input::BACKWARDS) {
            opts.forwards += 6;
            if input.contains(Input::BACKWARDS) {
                opts.forwards -= 12;
            }
        } else {
            if opts.forwards > 0 { opts.forwards -= 1 }
            if opts.forwards < 0 { opts.forwards += 1 }
        }
        if input.contains(Input::RIGHT) || input.contains(Input::LEFT) {
            opts.sideways += 6;
            if input.contains(Input::LEFT) {
                opts.sideways -= 12;
            }
        } else {
            if opts.sideways > 0 { opts.sideways -= 1; }
            if opts.sideways < 0 { opts.sideways += 1; }
        }
        let max_speed = self.max_speed;
        opts.forwards = opts.forwards.max(-max_speed).min(max_speed);
        opts.sideways = opts.sideways.max(-max_speed).min(max_speed);
        opts.rotate = opts.rotate.max(-8).min(8);
    }
}

/// Update player's sector.
pub fn update_player(map: &mut map::Map, opts: &UpdateOpts) {
    if opts.rotate != 0 {
        map.player.angle.0 += opts.rotate;
    }
    let mut x = 0;
    let mut y = 0;
    let sin = map.player.angle.to_radians().sin();
    let cos = map.player.angle.to_radians().cos();
    if opts.forwards != 0 {
        let forwards = opts.forwards as f32;
        let dx = -sin * forwards;
        let dy = cos * forwards;
        x += dx as i32;
        y += dy as i32;
    }
    if opts.sideways != 0 {
        let sideways = opts.sideways as f32;
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
    for (_, left, right) in walls.filter(|(_, l, _)| l.next_sector != -1) {
        if intrsect_movement_with_wall(left, right, [px, py], [tx, ty]) {
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
