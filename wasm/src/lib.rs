use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
static MAP: &[u8] = include_bytes!("../../map/tests/maps/SIMPLE0.MAP");

#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once()
}

#[wasm_bindgen]
pub struct Demo {
    map: map::Map,
    controller: render::controller::InputController,
    frame: Box<render::frame::Frame>,
    d3: render::d3::Renderer,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Input {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub c: bool,
    pub e: bool,
    pub q: bool,
    pub up: bool,
    pub left: bool,
    pub down: bool,
    pub right: bool,
    pub space: bool,
    pub left_shift: bool,
}

impl Input {
    #[rustfmt::skip]
    fn to_controller_input(&self) -> render::controller::Input {
        let mut input = render::controller::Input::empty();
        if self.w || self.up { input |= render::controller::Input::FORWARDS; }
        if self.s || self.down { input |= render::controller::Input::BACKWARDS; }
        if self.d { input |= render::controller::Input::RIGHT; }
        if self.a { input |= render::controller::Input::LEFT; }
        if self.right || self.w { input |= render::controller::Input::LOOK_RIGHT; }
        if self.left || self.q { input |= render::controller::Input::LOOK_LEFT; }
        if self.c { input |= render::controller::Input::CROUCH; }
        if self.space { input |= render::controller::Input::UP; }
        if self.left_shift { input |= render::controller::Input::DOWN; }
        input
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
        let map = map::Map::from_slice(MAP).unwrap();
        let controller = render::controller::InputController::new(&map);
        Self {
            map,
            controller,
            frame: Box::new([[0; render::frame::WIDTH]; render::frame::HEIGHT]),
            d3: render::d3::Renderer::new(),
        }
    }

    pub fn render(&mut self, ctx: &web_sys::CanvasRenderingContext2d) {
        self.d3.render(&self.map, &mut self.frame);
        // black frame to hide edge artifacts :P
        for i in 0..render::frame::WIDTH {
            self.frame[0][i] = 0;
            self.frame[render::frame::HEIGHT - 1][i] = 0;
        }
        for i in 0..render::frame::HEIGHT {
            self.frame[i][0] = 0;
            self.frame[i][render::frame::WIDTH - 1] = 0;
        }
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
        let delta = std::time::Duration::from_micros(16600);
        let input = input.to_controller_input();
        self.controller.update(&input, delta, &mut self.map);
    }
}
