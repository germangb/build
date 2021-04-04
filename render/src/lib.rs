use crate::frame::Frame;
use map::{player::Player, Map};
use nalgebra_glm as glm;

mod d2;
mod d3;
pub mod frame;

/// Render map frame from the perspective of the `map.player`.
pub fn render(map: &Map, frame: &mut Frame) {
    // transformations
    let view = compute_view_transformation(&map.player);
    let clip = compute_clip_transform();
    let viewport = compute_viewport();

    let sector = map.player.sector;
    d2::draw_axis(frame);
    d3::draw_sector(frame, &map, sector, &view, &clip, &viewport);
    d2::draw_sector(frame, &map, sector, &view, &clip, &viewport);
    d2::draw_player(frame, &map);
}

// compute camera view-to-clip-space transformation
// converts vertices from view-space into clip-space counterparts
// edges are generally clipped in this space
pub fn compute_clip_transform() -> glm::Mat3 {
    let aspect = (frame::WIDTH as f32) / (frame::HEIGHT as f32);
    let scale = 30000.0;
    glm::scaling2d(&glm::vec2(1.0 / scale, aspect / scale))
}

// compute player transformation (the inverse of the player's POV)
// this transformation is used in both the 2D and 3D renderers
// also known as VIEW transformation in most CG circles
fn compute_view_transformation(player: &Player) -> glm::Mat3 {
    #[rustfmt::skip]
    let Player { pos_x, pos_y, angle, .. } = player;
    let posx = *pos_x as f32;
    let posy = *pos_y as f32;
    let cos = angle.cos();
    let sin = angle.sin();
    let transform: glm::Mat3 = [[cos, sin, 0.0], [-sin, cos, 0.0], [posx, posy, 1.0]].into();
    glm::inverse(&transform)
}

pub fn compute_viewport() -> glm::Mat3 {
    let w2 = (frame::WIDTH / 2) as f32;
    let h2 = (frame::HEIGHT / 2) as f32;
    glm::translation2d(&glm::vec2(w2, h2)) * glm::scaling2d(&glm::vec2(w2, -h2))
}

#[rustfmt::skip]
fn clip_verts(left: &mut glm::Vec3, right: &mut glm::Vec3, eps: f32) {
    // clip y=0
    let t = (eps - left.y) / (right.y - left.y);
    if t > 0.0 && t < 1.0 {
        let clip = glm::lerp(left, right, t);
        if left.y < right.y { *left = clip; }
        else { *right = clip; }
    }
    // clip x=-1
    let t = ((eps - 1.0) - left.x) / (right.x - left.x);
    if t > 0.0 && t < 1.0 {
        let clip = glm::lerp(left, right, t);
        if left.x < right.x { *left = clip; }
        else { *right = clip; }
    }
    // clip y=1
    let t = ((1.0 - eps) - left.y) / (right.y - left.y);
    if t > 0.0 && t < 1.0 {
        let clip = glm::lerp(left, right, t);
        if left.y > right.y { *left = clip; }
        else { *right = clip; }
    }
    // clip x=1
    let t = ((1.0 - eps) - left.x) / (right.x - left.x);
    if t > 0.0 && t < 1.0 {
        let clip = glm::lerp(left, right, t);
        if left.x > right.x { *left = clip; }
        else { *right = clip; }
    }
}

// test if both left & right wall vertices are behind the player's POV
// if they are, the wall doesn't won't need to be rendered at all
fn is_outside_clip(left: &glm::Vec3, right: &glm::Vec3, eps: f32) -> bool {
    // FIXME(german): line-box intersection has false-positives
    let one_eps = 1.0 - eps;
    let eps_one = eps - 1.0;
    (left.y < eps && right.y < eps)
        || (left.y > one_eps && right.y > one_eps)
        || (left.x > one_eps && right.x > one_eps)
        || (left.x < eps_one && right.x < eps_one)
}
