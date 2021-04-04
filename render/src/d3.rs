use crate::{frame, frame::Frame};
use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Line, style::PrimitiveStyle};
use map::{sector::Wall, Map};
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
    const EPS: f32 = 0.01;
    #[rustfmt::skip]
    if crate::is_outside_clip(&le, &ri, EPS) { return };
    crate::clip_verts(&mut le, &mut ri, EPS);
    ri.x /= ri.y;
    le.x /= le.y;
    let le_h = (10.0 / le.y) as isize;
    let ri_h = (10.0 / ri.y) as isize;

    le = viewport * le;
    ri = viewport * ri;
    let h2 = (frame::HEIGHT / 2) as f32;
    let top_le = (le.x as isize, h2 as isize - le_h);
    let top_ri = (ri.x as isize, h2 as isize - ri_h);
    let bot_le = (le.x as isize, h2 as isize + le_h);
    let bot_ri = (ri.x as isize, h2 as isize + ri_h);
    {
        let top = line_drawing::Bresenham::new(top_le, top_ri);
        let bot = line_drawing::Bresenham::new(bot_le, bot_ri);
        for ((tx, mut ty), (bx, mut by)) in top.zip(bot) {
            #[rustfmt::skip] if tx <    0 || bx <    0 { continue };
            #[rustfmt::skip] if tx >= 320 || bx >= 320 { continue };
            ty = ty.max(0).min(200);
            by = by.max(0).min(200);
            for y in ty..by {
                frame[y as usize][tx as usize] = 0xaaaaaa;
            }
        }
    }
}
