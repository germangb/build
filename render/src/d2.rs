use crate::{
    frame,
    frame::{EGFrame, Frame},
};
use embedded_graphics::{
    fonts::{Font6x6, Text},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, Line, Rectangle},
    style::{PrimitiveStyle, TextStyle},
};
use map::{
    player::Player,
    sector::{SectorId, Wall},
    Map,
};
use nalgebra_glm as glm;
use std::collections::BTreeMap;

const MAX_SECTOR_RENDER_DEPTH: usize = 32;
const EPSILON: f32 = 1e-5;

bitflags::bitflags! {
    pub struct Flags: u8 {
        const PLAYER = 0b0000_0001;
        const AXIS   = 0b0000_0010;
        const SECTOR = 0b0000_0100;

        /// Clip sector geometry (hide everything behind the player).
        const CLIP   = 0b0000_1000;
    }
}

/// 2D MAP renderer.
#[derive(Debug)]
pub struct Renderer {
    /// Renderer bitflags.
    pub flags: Flags,

    visited_depth: BTreeMap<SectorId, usize>,
    view: glm::Mat3,
    clip: glm::Mat3,
}

macro_rules! draw_axis_label {
    ($frame:expr, $text:expr, ($x:expr, $y:expr), $color:expr) => {
        Text::new($text, Point::new($x, $y))
            .into_styled(TextStyle::new(Font6x6, $color))
            .draw(&mut EGFrame($frame))
            .unwrap();
    };
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            flags: Flags::all(),
            visited_depth: BTreeMap::new(),
            view: glm::identity(),
            clip: glm::identity(),
        }
    }

    /// Render MAP to the given frame.
    pub fn render(&mut self, map: &Map, frame: &mut Frame) {
        if self.flags.contains(Flags::AXIS) {
            Self::render_axis(frame);
        }
        if self.flags.contains(Flags::SECTOR) {
            self.view = compute_view(&map.player);
            self.clip = compute_clip(20000.0);
            self.visited_depth.clear();
            self.visited_depth.insert(map.player.sector, 0);
            self.render_sector(map, map.player.sector, frame);
        }
        if self.flags.contains(Flags::PLAYER) {
            Self::render_player(&map.player, frame);
        }
    }

    fn render_sector(&mut self, map: &Map, sector: SectorId, frame: &mut Frame) {
        let (_, walls) = map.sectors.get(sector).unwrap();
        walls.for_each(|(_, l, r)| {
            let child_depth = self.visited_depth[&sector] + 1;
            if l.next_sector != -1
                && !self.visited_depth.contains_key(&l.next_sector)
                && child_depth < MAX_SECTOR_RENDER_DEPTH
            {
                self.visited_depth.insert(l.next_sector, child_depth);
                self.render_sector(map, l.next_sector, frame);
            }
            self.render_wall(frame, map, sector, l, r);
        });
    }

    fn render_wall(&self, frame: &mut Frame, map: &Map, sector: i16, left: &Wall, right: &Wall) {
        let clip_view = &self.clip * &self.view;
        let mut left_clip = clip_view * glm::vec3(left.x as f32, left.y as f32, 1.0);
        let mut right_clip = clip_view * glm::vec3(right.x as f32, right.y as f32, 1.0);
        // clip vertices to POV
        if self.flags.contains(Flags::CLIP) {
            #[rustfmt::skip]
            if is_outside_clip(&left_clip, &right_clip, EPSILON) { return; };
            crate::util::clip_xy(&mut left_clip, &mut right_clip, EPSILON);
        }
        #[rustfmt::skip]
        let color = if left.next_sector == -1 { Rgb888::GREEN } else { Rgb888::RED };
        let stroke = if map.player.sector == sector { 3 } else { 1 };
        let left = self.apply_viewport(left_clip);
        let right = self.apply_viewport(right_clip);
        let point_left = Point::new(left.x as _, left.y as _);
        let point_right = Point::new(right.x as _, right.y as _);
        Line::new(point_left, point_right)
            .into_styled(PrimitiveStyle::with_stroke(color, stroke))
            .draw(&mut EGFrame(frame))
            .unwrap();
        let mut r0 = point_left.clone();
        r0.y -= 1;
        r0.x -= 1;
        let mut r1 = point_left.clone();
        r1.y += 1;
        r1.x += 1;
        Rectangle::new(r0, r1)
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::BLACK, 1))
            .draw(&mut EGFrame(frame))
            .unwrap();
    }

    fn apply_viewport(&self, mut v: glm::Vec3) -> glm::I32Vec2 {
        let viewport = [0, 0, frame::WIDTH as _, frame::HEIGHT as _];
        v.x += 0.5;
        v.y += 0.5;
        v.x = (1.0 - v.x) * (viewport[2] as f32) + (viewport[0] as f32);
        v.y = (1.0 - v.y) * (viewport[3] as f32) + (viewport[1] as f32);
        glm::vec2(v.x as i32, v.y as i32)
    }

    fn render_player(player: &Player, frame: &mut Frame) {
        let w = frame::WIDTH as i32;
        let h = frame::HEIGHT as i32;
        let w2 = w / 2;
        let h2 = h / 2;
        // reference axis
        // player & look direction
        let color = Rgb888::CYAN;
        Circle::new(Point::new(w2, h2), 2)
            .into_styled(PrimitiveStyle::with_fill(color))
            .draw(&mut EGFrame(frame))
            .unwrap();
        let offset = 12;
        Line::new(Point::new(w2, h2), Point::new(w2, h2 - offset))
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(&mut EGFrame(frame))
            .unwrap();
        // help text
        let text = format!("x={}\ny={}\nz={}", player.pos_x, player.pos_y, player.pos_z);
        Text::new(&text, Point::new(w2 + 6, h2 + 6))
            .into_styled(TextStyle::new(Font6x6, Rgb888::CYAN))
            .draw(&mut EGFrame(frame))
            .unwrap();
    }

    fn render_axis(frame: &mut Frame) {
        let w = frame::WIDTH as i32;
        let h = frame::HEIGHT as i32;
        let w2 = w / 2;
        let h2 = h / 2;
        let color = Rgb888::new(0x11, 0x11, 0x11);

        Line::new(Point::new(0, h2), Point::new(w, h2))
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(&mut EGFrame(frame))
            .unwrap();
        Line::new(Point::new(w2, 0), Point::new(w2, h))
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(&mut EGFrame(frame))
            .unwrap();

        draw_axis_label!(frame, "-1, 0", (0, h2 + 2), color);
        draw_axis_label!(frame, "1, 0", (w - 12, h2 + 2), color);
        draw_axis_label!(frame, "0, -1", (w2 + 2, h - 6), color);
        draw_axis_label!(frame, "0, 1", (w2 + 2, 0), color);
    }
}

fn compute_view(player: &Player) -> glm::Mat3 {
    #[rustfmt::skip]
    let Player { pos_x, pos_y, angle, .. } = player;
    let posx = *pos_x as f32;
    let posy = *pos_y as f32;
    let cos = angle.to_radians().cos();
    let sin = angle.to_radians().sin();
    let transform: glm::Mat3 = [[cos, sin, 0.0], [-sin, cos, 0.0], [posx, posy, 1.0]].into();
    glm::inverse(&transform)
}

fn compute_clip(scale: f32) -> glm::Mat3 {
    let aspect = (frame::WIDTH as f32) / (frame::HEIGHT as f32);
    glm::scaling2d(&glm::vec2(1.0 / scale, aspect / scale))
}

// test if both left & right wall vertices are behind the player's POV
// if they are, the wall doesn't need to be rendered at all
pub fn is_outside_clip(left: &glm::Vec3, right: &glm::Vec3, eps: f32) -> bool {
    // FIXME(german): line-box intersection has false-positives
    let one_eps = 1.0 - eps;
    let eps_one = eps - 1.0;
    (left.y < eps && right.y < eps)
        || (left.y > one_eps && right.y > one_eps)
        || (left.x > one_eps && right.x > one_eps)
        || (left.x < eps_one && right.x < eps_one)
}
