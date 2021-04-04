use crate::{frame, frame::Frame};
use embedded_graphics::{
    fonts::{Font6x6, Text},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, Line},
    style::{PrimitiveStyle, TextStyle},
};
use map::{player::Player, sector::Wall, Map};
use nalgebra_glm as glm;

pub fn draw_sector(
    frame: &mut Frame,
    map: &Map,
    sector: i16,
    view: &glm::Mat3,
    clip: &glm::Mat3,
    viewport: &glm::Mat3,
) {
    let (_, walls) = map.sectors.get(sector as _).unwrap();
    walls.for_each(|(l, r)| draw_wall(frame, l, r, view, clip, viewport));
}

// draw transformed wall
// player holds the transform of the player's POV
pub fn draw_wall(
    frame: &mut Frame,
    l: &Wall,
    r: &Wall,
    view: &glm::Mat3,
    clip: &glm::Mat3,
    viewport: &glm::Mat3,
) {
    let mut le = clip * view * glm::vec3(l.x as f32, l.y as f32, 1.0);
    let mut ri = clip * view * glm::vec3(r.x as f32, r.y as f32, 1.0);
    // clip vertices to POV
    const EPS: f32 = 0.001;
    #[rustfmt::skip]
    if crate::is_outside_clip(&le, &ri, EPS) { return };
    crate::clip_verts(&mut le, &mut ri, EPS);
    // adjust to viewport
    le = viewport * le;
    ri = viewport * ri;
    let mut point_left = Point::new(le.x as _, le.y as _);
    let mut point_right = Point::new(ri.x as _, ri.y as _);
    #[rustfmt::skip]
    let mut color = if l.next_sector != -1 { Rgb888::GREEN } else { Rgb888::RED };
    Line::new(point_left, point_right)
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();
}

macro_rules! draw_axis_label {
    ($frame:expr, $text:expr, ($x:expr, $y:expr), $color:expr) => {
        Text::new($text, Point::new($x, $y))
            .into_styled(TextStyle::new(Font6x6, $color))
            .draw($frame)
            .unwrap();
    };
}

pub fn draw_axis(frame: &mut Frame) {
    let w = frame::WIDTH as i32;
    let h = frame::HEIGHT as i32;
    let w2 = w / 2;
    let h2 = h / 2;
    let color = Rgb888::new(0x11, 0x11, 0x11);
    Line::new(Point::new(0, h2), Point::new(w, h2))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();
    Line::new(Point::new(w2, 0), Point::new(w2, h))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();

    draw_axis_label!(frame, "-1, 0", (0, h2 + 2), color);
    draw_axis_label!(frame, "1, 0", (w - 12, h2 + 2), color);
    draw_axis_label!(frame, "0, -1", (w2 + 2, h - 6), color);
    draw_axis_label!(frame, "0, 1", (w2 + 2, 0), color);
}

pub fn draw_player(frame: &mut Frame, map: &Map) {
    let player = &map.player;
    let w = frame::WIDTH as i32;
    let h = frame::HEIGHT as i32;
    let w2 = w / 2;
    let h2 = h / 2;
    // reference axis
    // player & look direction
    let color = Rgb888::CYAN;
    Circle::new(Point::new(w2, h2), 2)
        .into_styled(PrimitiveStyle::with_fill(color))
        .draw(frame)
        .unwrap();
    let offset = 12;
    Line::new(Point::new(w2, h2), Point::new(w2, h2 - offset))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();
    // help text
    let text = format!("x={}\ny={}\nz={}", player.pos_x, player.pos_y, player.pos_z);
    Text::new(&text, Point::new(w2 + 6, h2 + 6))
        .into_styled(TextStyle::new(Font6x6, Rgb888::CYAN))
        .draw(frame)
        .unwrap();
}
