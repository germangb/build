use crate::frame::Frame;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, Line},
    style::PrimitiveStyle,
};
use map::{player::Player, sector::Wall, Map};
use nalgebra_glm as glm;
use std::collections::{HashMap, HashSet};

pub mod frame;

/// Render map frame from the perspective of the `map.player`.
pub fn render(map: &Map, frame: &mut Frame) {
    let trans_inv = compute_player_transform(&map.player);
    let sector = map.player.sector as _;
    draw_player(frame, &map.player);

    // render sector walls
    let (_, walls) = map.sectors.get(sector).unwrap();
    walls.for_each(|(l, r)| {
        draw_wall(frame, l, r, &trans_inv);
        if l.next_sector != -1 {
            let (_, walls) = map.sectors.get(l.next_sector as _).unwrap();
            walls.for_each(|(l, r)| draw_wall(frame, l, r, &trans_inv));
        }
    });
}

fn compute_player_transform(player: &Player) -> glm::Mat3 {
    let w2 = (frame::WIDTH / 2) as f32;
    let h2 = (frame::HEIGHT / 2) as f32;
    let scale = 0.01; // magic scaling factor
    #[rustfmt::skip]
    let Player { pos_x, pos_y, angle, .. } = player;
    let pos_x = *pos_x as f32;
    let pos_y = *pos_y as f32;
    let cos = angle.cos();
    let sin = angle.sin();
    let trans: glm::Mat3 = [[cos, sin, 0.0], [-sin, cos, 0.0], [pos_x, pos_y, 1.0]].into();
    glm::translation2d(&glm::vec2(w2, h2))
        * glm::scaling2d(&glm::vec2(scale, scale))
        * glm::inverse(&trans)
}

// draw transformed wall
// player holds the transform of the player's POV
fn draw_wall(frame: &mut Frame, l: &Wall, r: &Wall, player: &glm::Mat3) {
    let h2 = (frame::HEIGHT / 2) as i32;
    let vec_left = player * glm::vec3(l.x as f32, l.y as f32, 1.0);
    let vec_right = player * glm::vec3(r.x as f32, r.y as f32, 1.0);
    let point_left = Point::new(vec_left.x as _, vec_left.y as _);
    let point_right = Point::new(vec_right.x as _, vec_right.y as _);
    // don't render the wall if it's behind the player.
    // this only works if sector walls are defined in CW order.
    //if point_left.x < point_right.x && (point_left.y <= h2 || point_right.y <= h2) {
        #[rustfmt::skip]
        let mut color = if l.next_sector != -1 { Rgb888::GREEN } else { Rgb888::RED };
        Line::new(point_left, point_right)
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(frame)
            .unwrap();
    //}
}

fn draw_player(frame: &mut Frame, player: &Player) {
    let w = frame::WIDTH as i32;
    let h = frame::HEIGHT as i32;
    let w2 = w / 2;
    let h2 = h / 2;
    // reference axis
    let color = Rgb888::new(0x11, 0x11, 0x11);
    Line::new(Point::new(0, h2), Point::new(w, h2))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();
    Line::new(Point::new(w2, 0), Point::new(w2, h))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();
    // player & look direction
    Circle::new(Point::new(w2, h2), 2)
        .into_styled(PrimitiveStyle::with_fill(Rgb888::GREEN))
        .draw(frame)
        .unwrap();
    let offset = 12;
    Line::new(Point::new(w2, h2), Point::new(w2, h2 - offset))
        .into_styled(PrimitiveStyle::with_stroke(Rgb888::GREEN, 1))
        .draw(frame)
        .unwrap();
}
