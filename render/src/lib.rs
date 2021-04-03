use crate::frame::Frame;
use embedded_graphics::{
    fonts::{Font6x6, Text},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Circle, Line},
    style::{PrimitiveStyle, TextStyle},
};
use map::{player::Player, sector::Wall, Map};
use nalgebra_glm as glm;
use std::collections::{HashMap, HashSet};

pub mod frame;

/// Render map frame from the perspective of the `map.player`.
pub fn render(map: &Map, frame: &mut Frame) {
    draw_axis(frame);
    let player = compute_player_transform(&map.player);
    let viewport = compute_viewport_transform();
    let sector = map.player.sector;
    let mut bfs = vec![sector].into_iter().collect();
    draw_sector_wire(frame, &map, sector, &player, &viewport, Some((0, &mut bfs)));
    //draw_sector_wire(frame, &map, sector, &player, &viewport, None);
    draw_player(frame, &map);
}

fn compute_viewport_transform() -> glm::Mat3 {
    let w2 = (frame::WIDTH / 2) as f32;
    let h2 = (2 * frame::HEIGHT / 3) as f32;
    let scale = 0.025; // magic scaling factor
    glm::translation2d(&glm::vec2(w2, h2)) * glm::scaling2d(&glm::vec2(scale, -scale))
}

fn compute_player_transform(player: &Player) -> glm::Mat3 {
    #[rustfmt::skip]
    let Player { pos_x, pos_y, angle, .. } = player;
    let pos_x = *pos_x as f32;
    let pos_y = *pos_y as f32;
    let cos = angle.cos();
    let sin = angle.sin();
    let trans: glm::Mat3 = [[cos, sin, 0.0], [-sin, cos, 0.0], [pos_x, pos_y, 1.0]].into();
    glm::inverse(&trans)
}

fn draw_sector_wire(
    frame: &mut Frame,
    map: &Map,
    sector: i16,
    player: &glm::Mat3,
    viewport: &glm::Mat3,
    mut bfs: Option<(usize, &mut HashSet<i16>)>,
) {
    let (_, walls) = map.sectors.get(sector as _).unwrap();
    walls.for_each(|(l, r)| {
        draw_wall_wire(frame, l, r, player, viewport);
        if let Some((depth, visit)) = &mut bfs {
            if l.next_sector != -1 && *depth < 1 && !visit.contains(&l.next_sector) {
                visit.insert(l.next_sector);
                draw_sector_wire(
                    frame,
                    map,
                    l.next_sector,
                    player,
                    viewport,
                    Some((*depth + 1, visit)),
                );
            }
        }
    });
}

#[rustfmt::skip]
fn clip_verts(left: &mut glm::Vec3, right: &mut glm::Vec3) {
    const E: f32 = 1.0;
    let t = (E - left.y) / (right.y - left.y);
    if t > 0.0 && t < 1.0 {
        let clipped = glm::lerp(left, right, t);
        if left.y < right.y { *left = clipped; }
        else { *right = clipped; }
    }
}

fn is_behind_player(left: &glm::Vec3, right: &glm::Vec3) -> bool {
    const E: f32 = 1.0;
    left.y < E && right.y < E
}

// draw transformed wall
// player holds the transform of the player's POV
fn draw_wall_wire(frame: &mut Frame, l: &Wall, r: &Wall, player: &glm::Mat3, viewport: &glm::Mat3) {
    let mut left = player * glm::vec3(l.x as f32, l.y as f32, 1.0);
    let mut right = player * glm::vec3(r.x as f32, r.y as f32, 1.0);
    // clip vertices to POV
    #[rustfmt::skip]
    if is_behind_player(&left, &right) { return };
    clip_verts(&mut left, &mut right);
    // adjust to viewport
    left = viewport * left;
    right = viewport * right;
    let mut point_left = Point::new(left.x as _, left.y as _);
    let mut point_right = Point::new(right.x as _, right.y as _);
    // don't render the wall if POV is facing away from it.
    // this only works if sector walls are defined in CW order.
    //if point_left.x < point_right.x {
    #[rustfmt::skip]
    let mut color = if l.next_sector != -1 { Rgb888::GREEN } else { Rgb888::RED };
    Line::new(point_left, point_right)
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();
    //}
}

fn draw_axis(frame: &mut Frame) {
    let w = frame::WIDTH as i32;
    let h = frame::HEIGHT as i32;
    let w2 = w / 2;
    let h2 = 2 * h / 3;
    let color = Rgb888::new(0x11, 0x11, 0x11);
    Line::new(Point::new(0, h2), Point::new(w, h2))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();
    Line::new(Point::new(w2, 0), Point::new(w2, h))
        .into_styled(PrimitiveStyle::with_stroke(color, 1))
        .draw(frame)
        .unwrap();
}

fn draw_player(frame: &mut Frame, map: &Map) {
    let player = &map.player;
    let w = frame::WIDTH as i32;
    let h = frame::HEIGHT as i32;
    let w2 = w / 2;
    let h2 = 2 * h / 3;
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
        .into_styled(TextStyle::new(Font6x6, Rgb888::WHITE))
        .draw(frame)
        .unwrap();
}
