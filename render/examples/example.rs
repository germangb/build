use map::Map;
use minifb::{Key, Scale, Window, WindowOptions};
use render::frame::Frame;
use std::{env, path::PathBuf, time::Duration};

fn main() {
    let path = env::args().skip(1).next().expect("Missing MAP argument.");
    let path = PathBuf::from(path);
    let mut map = Map::from_file(&path).unwrap();
    let mut frame = Frame::new();

    let mut opts = WindowOptions::default();
    opts.scale = Scale::X2;
    let filename = path.file_name().unwrap().to_str().unwrap();
    let mut window = Window::new(filename, 320, 200, opts).unwrap();

    while window.is_open() {
        if window.is_key_down(Key::Left) {
            map.player.angle.0 -= 2;
        }
        if window.is_key_down(Key::Right) {
            map.player.angle.0 += 2;
        }
        if window.is_key_down(Key::Up) {
            let x = map.player.angle.sin() * 32.0;
            let y = -map.player.angle.cos() * 32.0;
            map.player.pos_x += x as i32;
            map.player.pos_y += y as i32;
        }
        if window.is_key_down(Key::Down) {
            let x = map.player.angle.sin() * 32.0;
            let y = -map.player.angle.cos() * 32.0;
            map.player.pos_x -= x as i32;
            map.player.pos_y -= y as i32;
        }

        frame.clear();
        render::render(&map, &mut frame);
        let buffer = unsafe { std::slice::from_raw_parts(frame.as_ptr() as _, 320 * 200) };
        window.update_with_buffer(buffer, 320, 200).unwrap();
    }
}
