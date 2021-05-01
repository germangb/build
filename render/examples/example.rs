use map::{player::Player, sector::SectorId, Map};
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use render::{d2, d3, frame, frame::Frame, Input, UpdateOpts};
use std::{env, path::PathBuf};

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
    opts.scale = Scale::X2;
    let title = path.file_name().unwrap().to_str().unwrap();
    let mut window = Window::new(&title, frame::WIDTH, frame::HEIGHT, opts).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let mut d2_enabled = true;
    let mut d3_enabled = true;

    while window.is_open() {
        let input = input_flags(&window, &mut update_opts);
        render::update(&mut map, &input, &update_opts);
        let floor_z = map.sectors.sectors()[map.player.sector as usize].floor_z;
        map.player.pos_z = floor_z;
        if window.is_key_down(Key::LeftCtrl) {
            map.player.pos_z += eye_height / 2;
        } else {
            map.player.pos_z += eye_height;
        }

        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            d2_enabled = !d2_enabled;
        }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            d3_enabled = !d3_enabled;
        }

        // render map to frame
        #[cfg(debug_assertions)]
        {
            *frame = [[0; frame::WIDTH]; frame::HEIGHT];
        }
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

fn input_flags(window: &Window, opts: &mut UpdateOpts) -> Input {
    let mut input = Input::empty();
    opts.forwards = 64;
    opts.sideways = 64;
    opts.rotate = 8;
    if window.is_key_down(Key::Right) || window.is_key_down(Key::Left) {
        input |= Input::ANGULAR;
        if window.is_key_down(Key::Left) {
            opts.rotate = -opts.rotate;
        }
    }
    if window.is_key_down(Key::Up)
        || window.is_key_down(Key::Down)
        || window.is_key_down(Key::W)
        || window.is_key_down(Key::S)
    {
        input |= Input::FORWARDS;
        if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
            opts.forwards = -opts.forwards;
        }
    }
    if window.is_key_down(Key::D) || window.is_key_down(Key::A) {
        input |= Input::SIDEWAYS;
        if window.is_key_down(Key::A) {
            opts.sideways = -opts.sideways;
        }
    }
    input
}
