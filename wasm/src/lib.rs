use render::{d2, d3, frame::Frame, Input as RenderInput, UpdateOpts};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once()
}

#[wasm_bindgen]
pub struct Demo {
    map: map::Map,
    frame: Box<Frame>,
    d2: d2::Renderer,
    d3: d3::Renderer,
    eye_height: i32,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Input {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
    pub right: bool,
}

impl Input {
    fn to_render_input(&self) -> (RenderInput, render::UpdateOpts) {
        let mut input = RenderInput::empty();
        let mut opts = UpdateOpts::default();
        opts.forwards = 64;
        opts.sideways = 32;
        opts.rotate = 8;
        if self.right || self.left {
            input |= RenderInput::ANGULAR;
            if self.left {
                opts.rotate = -opts.rotate;
            }
        }
        if self.up || self.down || self.w || self.s {
            input |= RenderInput::FORWARDS;
            if self.down || self.s {
                opts.forwards = -opts.forwards;
            }
        }
        if self.a || self.d {
            input |= RenderInput::SIDEWAYS;
            if self.a {
                opts.sideways = -opts.sideways;
            }
        }
        (input, opts)
    }
}

#[wasm_bindgen]
impl Input {
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
impl Demo {
    pub fn new() -> Self {
        let map = map::Map::from_slice(include_bytes!("../../map/tests/maps/GERMAN.MAP")).unwrap();
        let eye_height = compute_eye_height(&map);
        Self {
            map,
            frame: Box::new([[0; render::frame::WIDTH]; render::frame::HEIGHT]),
            d2: render::d2::Renderer::new(),
            d3: render::d3::Renderer::new(),
            eye_height,
        }
    }

    pub fn render(&mut self, ctx: &web_sys::CanvasRenderingContext2d) {
        self.d3.render(&self.map, &mut self.frame);
        self.d2.flags = render::d2::Flags::SECTOR | render::d2::Flags::PLAYER;
        self.d2.render(&self.map, &mut self.frame);
        let clamped = wasm_bindgen::Clamped(unsafe {
            std::slice::from_raw_parts(
                self.frame.as_ptr() as *const u8,
                (render::frame::WIDTH * render::frame::HEIGHT * 4) as _,
            )
        });
        let image_data =
            web_sys::ImageData::new_with_u8_clamped_array(clamped, (render::frame::WIDTH) as _)
                .expect("Error creating image data");
        ctx.put_image_data(&image_data, 0.0, 0.0)
            .expect("Error writing image to canvas");
    }

    pub fn update(&mut self, input: &Input) {
        let (input, opts) = input.to_render_input();
        render::update(&mut self.map, &input, &opts);
        let floor_z = self.map.sectors.sectors()[self.map.player.sector as usize].floor_z;
        self.map.player.pos_z = floor_z;
        self.map.player.pos_z += self.eye_height;
    }
}

fn compute_eye_height(map: &map::Map) -> i32 {
    let sector = map.player.sector;
    map.player.pos_z - map.sectors.get(sector).unwrap().0.floor_z
}
