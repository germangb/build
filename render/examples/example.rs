use map::Map;
use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
use render::{controller::Input, d2, d3, frame, frame::Frame};
use std::{env, path::PathBuf};

const MAX_SPEED: i32 = 32;

fn main() {
    let path = env::args()
        .skip(1)
        .next()
        .map(PathBuf::from)
        .expect("Missing MAP argument.");

    let mut map = Map::from_file(&path).unwrap();
    let mut frame = Box::new([[0; frame::WIDTH]; frame::HEIGHT]);
    let mut d3 = d3::Renderer::new();
    let mut d2 = d2::Renderer::new();
    let mut controller = render::controller::InputController::new(&mut map);
    controller.max_speed = MAX_SPEED;

    let mut opts = WindowOptions::default();
    opts.scale = Scale::X2;
    //opts.borderless = true;
    let title = path.file_name().unwrap().to_str().unwrap();
    let mut window = Window::new(&title, frame::WIDTH, frame::HEIGHT, opts).unwrap();
    let delta = std::time::Duration::from_micros(16600);
    window.limit_update_rate(Some(delta));
    let mut d2_enabled = true;
    let mut d3_enabled = true;

    while window.is_open() {
        unsafe {
            //*frame = std::mem::zeroed();
        }
        let input = resolve_input(&window);
        controller.update(&input, delta, &mut map);

        if window.is_key_pressed(Key::F, KeyRepeat::No) {
            controller.fly = !controller.fly;
            println!("fly = {}", controller.fly);
        }
        if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            d2_enabled = !d2_enabled;
        }
        if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            d3_enabled = !d3_enabled;
        }

        // render map to frame
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
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
        /*
        for i in 0..frame::WIDTH {
            frame[0][i] = 0;
            frame[frame::HEIGHT - 1][i] = 0;
        }
        for i in 0..frame::HEIGHT {
            frame[i][0] = 0;
            frame[i][frame::WIDTH - 1] = 0;
        }
        */

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
fn resolve_input(window: &Window) -> Input {
    let mut input = Input::empty();
    if window.is_key_down(Key::W) || window.is_key_down(Key::Up) { input |= Input::FORWARDS; }
    if window.is_key_down(Key::S) || window.is_key_down(Key::Down) { input |= Input::BACKWARDS; }
    if window.is_key_down(Key::D) { input |= Input::RIGHT; }
    if window.is_key_down(Key::A) { input |= Input::LEFT; }
    if window.is_key_down(Key::Right) || window.is_key_down(Key::E) { input |= Input::LOOK_RIGHT; }
    if window.is_key_down(Key::Left) || window.is_key_down(Key::Q) { input |= Input::LOOK_LEFT; }
    if window.is_key_down(Key::C) { input |= Input::CROUCH; }
    if window.is_key_down(Key::Space) { input |= Input::UP; }
    if window.is_key_down(Key::LeftShift) { input |= Input::DOWN; }
    input
}
