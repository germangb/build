use map::{player::Player, sector::SectorId, Map};
use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};
use render::{d2, d3, frame, frame::Frame, UpdateOpts};
use std::{env, path::PathBuf};

const MAX_SPEED: i32 = 42;

fn compute_eye_height(map: &Map) -> i32 {
    let sector = map.player.sector;
    map.player.pos_z - map.sectors.get(sector).unwrap().0.floor_z
}

fn main() {
    let path = env::args()
        .skip(1)
        .next()
        .map(PathBuf::from)
        .expect("Missing MAP argument.");

    let mut map = Map::from_file(&path).unwrap();
    let eye_height = compute_eye_height(&map);
    let mut frame = Box::new([[0; frame::WIDTH]; frame::HEIGHT]);
    let mut d3 = d3::Renderer::new();
    let mut d2 = d2::Renderer::new();
    let mut update_opts = UpdateOpts::default();

    let mut opts = WindowOptions::default();
    opts.scale = Scale::X1;
    //opts.borderless = true;
    let title = path.file_name().unwrap().to_str().unwrap();
    let mut window = Window::new(&title, frame::WIDTH, frame::HEIGHT, opts).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut d2_enabled = true;
    let mut d3_enabled = true;

    while window.is_open() {
        update_player_movement_opts(&window, &mut update_opts);
        render::update_player_movement(&mut map, &update_opts);

        // update eye level
        let floor_z = map.sectors.sectors()[map.player.sector as usize].floor_z;
        let mut target_z = floor_z + eye_height;
        if window.is_key_down(Key::C) {
            target_z -= eye_height / 2;
        }
        map.player.pos_z += (target_z - map.player.pos_z) >> 1;

        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            d2_enabled = !d2_enabled;
        }
        #[cfg(nope)]
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            d3_enabled = !d3_enabled;
        }

        // render map to frame
        #[cfg(nope)]
        {
            *frame = [[0; frame::WIDTH]; frame::HEIGHT];
        }
        #[cfg(nope)]
        if d2_enabled {
            d2.flags = render::d2::Flags::AXIS;
            d2.render(&map, &mut frame);
        }
        if d3_enabled {
            d3.render(&map, &mut frame);
        }
        if d2_enabled {
            d2.flags = d2::Flags::SECTOR | d2::Flags::PLAYER;
            d2.render(&map, &mut frame);
        }
        // black frame to hide edge artifacts :P
        for i in 0..frame::WIDTH {
            frame[0][i] = 0;
            frame[frame::HEIGHT - 1][i] = 0;
        }
        for i in 0..frame::HEIGHT {
            frame[i][0] = 0;
            frame[i][frame::WIDTH - 1] = 0;
        }

        // update window framebuffer
        update_window_buffer(&mut window, &frame);
    }
}

fn update_window_buffer(window: &mut Window, frame: &Frame) {
    let len = frame::WIDTH * frame::HEIGHT;
    let buffer = unsafe { std::slice::from_raw_parts(frame.as_ptr() as _, len) };
    window
        .update_with_buffer(buffer, frame::WIDTH, frame::HEIGHT)
        .unwrap();
}

#[rustfmt::skip]
fn update_player_movement_opts(window: &Window, opts: &mut UpdateOpts) {
    if window.is_key_down(Key::Right)
        || window.is_key_down(Key::Left)
        || window.is_key_down(Key::E)
        || window.is_key_down(Key::Q) {
        opts.rotate += 2;
        if window.is_key_down(Key::Left) || window.is_key_down(Key::Q) {
            opts.rotate -= 4;
        }
    } else {
        if opts.rotate > 0 { opts.rotate -= 1; }
        if opts.rotate < 0 { opts.rotate += 1; }
    }
    if window.is_key_down(Key::Up)
        || window.is_key_down(Key::Down)
        || window.is_key_down(Key::W)
        || window.is_key_down(Key::S)
    {
        opts.forwards += 6;
        if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
            opts.forwards -= 12;
        }
    } else {
        if opts.forwards > 0 { opts.forwards -= 1 }
        if opts.forwards < 0 { opts.forwards += 1 }
    }
    if window.is_key_down(Key::D) || window.is_key_down(Key::A) {
        opts.sideways += 6;
        if window.is_key_down(Key::A) {
            opts.sideways -= 12;
        }
    } else {
        if opts.sideways > 0 { opts.sideways -= 1; }
        if opts.sideways < 0 { opts.sideways += 1; }
    }
    opts.forwards = opts.forwards.max(-MAX_SPEED).min(MAX_SPEED);
    opts.sideways = opts.sideways.max(-MAX_SPEED).min(MAX_SPEED);
    opts.rotate = opts.rotate.max(-8).min(8);
}
