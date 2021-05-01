use crate::{d3::algo::Coverage, frame, frame::Frame};
use algo::Interval;
use map::{
    player::Player,
    sector::{Sector, SectorId, Wall},
    Map,
};
use nalgebra_glm as glm;
use std::collections::{HashSet, VecDeque, HashMap, BTreeMap};

/// support data structures and algos.
mod algo;

const EPSILON: f32 = 1e-4;
const BFS_MAX_DEPTH: usize = 32;

type Point = [i32; 2];
type Line = [Point; 2];

/// Struct holding the vertex of the following geometry.
///
/// ```no_rust
///           top_left *--------* top_right
///                    |        |
///     inner_top_left *--------* inner_top_right
///                    |        |
///  inner_bottom_left *--------* inner_bottom_right
///                    |        |
///        bottom_left *--------* bottom_right
/// ```
#[derive(Debug)]
struct Geometry<T> {
    // outer points
    pub top_left: T,
    pub top_right: T,
    pub bottom_left: T,
    pub bottom_right: T,
    // inner points
    pub inner_top_left: T,
    pub inner_top_right: T,
    pub inner_bottom_left: T,
    pub inner_bottom_right: T,
}

/// 3D MAP renderer.
#[derive(Debug)]
pub struct Renderer {
    /// Viewport transformation params.
    pub viewport: [i32; 4],
    coverage: Coverage,
    queue: VecDeque<SectorId>,
    visited_depth: BTreeMap<SectorId, usize>,
    clip_view: glm::Mat4,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            viewport: [0, 0, frame::WIDTH as _, frame::HEIGHT as _],
            coverage: Coverage::new(),
            queue: VecDeque::new(),
            visited_depth: BTreeMap::new(),
            clip_view: glm::identity(),
        }
    }

    /// Render MAP to the given frame.
    pub fn render(&mut self, map: &Map, frame: &mut Frame) {
        // math transforms
        self.clip_view = clip_transform() * view_transform(&map.player);
        // pixel coverage
        self.coverage.reset();
        // bfs state
        self.visited_depth.clear();
        self.queue.clear();
        self.visited_depth.insert(map.player.sector, 0);
        self.queue.push_back(map.player.sector);
        // do bfs traversal
        while let Some(sector_id) = self.queue.pop_front() {
            let (sector, walls) = map.sectors.get(sector_id).unwrap();
            for (left, right) in walls {
                if left.next_sector == -1 {
                    if let Some(points) = self.project_wall(map, sector, left, right) {
                        self.render_solid_wall(&points, frame);
                    }
                } else {
                    if let Some(points) = self.project_wall(map, sector, left, right) {
                        let depth = self.visited_depth[&sector_id] + 1;
                        if depth < BFS_MAX_DEPTH
                            && self.render_portal_wall(&points, frame)
                            && !self.visited_depth.contains_key(&left.next_sector)
                        {
                            self.visited_depth.insert(left.next_sector, depth);
                            self.queue.push_back(left.next_sector);
                        }
                    }
                }
            }
            // exit early if all pixels in the frame have been drawn
            if self.coverage.is_full() {
                break;
            }
        }
    }

    #[rustfmt::skip]
    fn render_solid_wall(&mut self, geometry: &Geometry<Point>, frame: &mut Frame) {
        let mut wall_lines = self.wall_lines_iter(&geometry);
        let wall_lines_len = wall_lines.size_hint().1.unwrap();
        for (i, (wall, _)) in wall_lines.enumerate() {
            // wall
            let mut color = if i == 0 { 0xffffff } else { 0xaaaaaa };
            if i == wall_lines_len - 1 { color = 0x333333 }
            self.render_line(&wall, color, frame);
            // ceiling
            self.render_line(&[[wall[0][0], 0], wall[0]], 0x0000ff, frame);
            // floor
            self.render_line(&[wall[1], [wall[1][0], frame::HEIGHT as _]], 0xffffff, frame);
            // update pixel coverage of the column to be 100%
            // no more pixels will be rendered on this column
            self.coverage.intersect_column(wall[0][0] as _, &None);
        }
    }

    #[rustfmt::skip]
    fn render_portal_wall(&mut self, geometry: &Geometry<Point>, frame: &mut Frame) -> bool {
        // compute if the wall has either a top-frame, bottom-frame, or both.
        let top_frame = geometry.top_left[1] < geometry.inner_top_left[1];
        let bottom_frame = geometry.bottom_left[1] > geometry.inner_bottom_left[1];
        let mut wall_lines = self.wall_lines_iter(&geometry);
        let wall_lines_len = wall_lines.size_hint().1.unwrap();
        // FIXME(german): Hack: has any pixel been drawn?
        let mut drawn_pixels = false;
        for (i, (wall, portal)) in wall_lines.enumerate() {
            drawn_pixels = true;
            // portal
            let mut color = if i == 0 { 0xffaaaa } else { 0xff0000 };
            if i == wall_lines_len - 1 { color = 0xaa0000 }
            //self.render_line(&portal, color, frame);
            self.render_line(&[[portal[1][0], portal[1][1] - 1], portal[1]], 0x333333, frame);
            self.render_line(&[portal[0], [portal[0][0], portal[0][1] + 1]], 0x000066, frame);
            // frames
            if top_frame || bottom_frame {
                let mut color = if i == 0 { 0xffffff } else { 0xaaaaaa };
                if i == wall_lines_len - 1 { color = 0x333333 }
                if top_frame {
                    self.render_line(&[wall[0], portal[0]], color, frame);
                }
                let mut color = if i == 0 { 0xffaaff } else { 0xff00ff };
                if i == wall_lines_len - 1 { color = 0xaa00aa }
                if bottom_frame {
                    self.render_line(&[portal[1], wall[1]], color, frame);
                }
            }
            // ceiling
            self.render_line(&[[wall[0][0], 0], wall[0]], 0x0000ff, frame);
            // floor
            self.render_line(&[wall[1], [wall[1][0], frame::HEIGHT as _]], 0xffffff, frame);
            // update column pixel coverage
            self.coverage.intersect_column(wall[0][0] as _, &Some([portal[0][1] + 1, portal[1][1] - 1]));
        }
        drawn_pixels
    }

    fn render_line(&mut self, line: &Line, color: u32, frame: &mut Frame) {
        let [x0, y0] = line[0];
        let [x1, y1] = line[1];
        assert_eq!(x0, x1);
        // only render those pixels that haven't been painted yet.
        let current_coverage = self.coverage.get_column(x0 as _);
        if let Some([y0, y1]) = algo::intersect(&current_coverage, &Some([y0, y1])) {
            for y in y0..y1 {
                frame[y as usize][x0 as usize] = color;
            }
        }
    }

    fn wall_lines_iter<'a>(
        &self,
        points: &'a Geometry<Point>,
    ) -> impl Iterator<Item = (Line, Line)> + 'a {
        let d = points.top_right[0] - points.top_left[0];
        let x0 = (points.top_left[0]).max(0).min(frame::WIDTH as i32);
        let x1 = (points.top_right[0]).max(0).min(frame::WIDTH as i32);
        (x0..x1).filter_map(move |x| {
            let n = x - points.top_left[0];
            let mut mid_y0 =
                points.top_left[1] + n * (points.top_right[1] - points.top_left[1]) / d;
            let mut mid_y1 =
                points.bottom_left[1] + n * (points.bottom_right[1] - points.bottom_left[1]) / d;
            let mut top_y0 = points.inner_top_left[1]
                + n * (points.inner_top_right[1] - points.inner_top_left[1]) / d;
            let mut top_y1 = points.inner_bottom_left[1]
                + n * (points.inner_bottom_right[1] - points.inner_bottom_left[1]) / d;
            mid_y0 = mid_y0.max(0).min(frame::HEIGHT as i32);
            mid_y1 = mid_y1.max(0).min(frame::HEIGHT as i32);
            top_y0 = top_y0.max(0).min(frame::HEIGHT as i32);
            top_y1 = top_y1.max(0).min(frame::HEIGHT as i32);
            let mid = [[x, mid_y0], [x, mid_y1]]; // portal
            let top = [[x, top_y0.max(mid_y0)], [x, top_y1.min(mid_y1)]];
            Some((mid, top))
        })
    }

    // compute the coordinates of a sector wall in the viewport
    // if the wall is entirely behind the player, return None
    fn project_wall(
        &self,
        map: &Map,
        sector: &Sector,
        left: &Wall,
        right: &Wall,
    ) -> Option<Geometry<Point>> {
        let Geometry {
            top_left: mut top_left,
            top_right: mut top_right,
            bottom_left: mut bottom_left,
            bottom_right: mut bottom_right,
            inner_top_left: mut inner_top_left,
            inner_top_right: mut inner_top_right,
            inner_bottom_left: mut inner_bottom_left,
            inner_bottom_right: mut inner_bottom_right,
        } = wall_to_glm(map, sector, left, right);
        top_left = self.clip_view * top_left;
        top_right = self.clip_view * top_right;
        // clip wall entirely (don't render it at all)
        if top_left.y < EPSILON && top_right.y < EPSILON {
            return None;
        }
        bottom_left = self.clip_view * bottom_left;
        bottom_right = self.clip_view * bottom_right;
        inner_top_left = self.clip_view * inner_top_left;
        inner_top_right = self.clip_view * inner_top_right;
        inner_bottom_left = self.clip_view * inner_bottom_left;
        inner_bottom_right = self.clip_view * inner_bottom_right;
        clip_y(&mut top_left, &mut top_right, EPSILON);
        clip_y(&mut bottom_left, &mut bottom_right, EPSILON);
        clip_y(&mut inner_top_left, &mut inner_top_right, EPSILON);
        clip_y(&mut inner_bottom_left, &mut inner_bottom_right, EPSILON);
        top_left /= top_left.y;
        top_right /= top_right.y;
        bottom_left /= bottom_left.y;
        bottom_right /= bottom_right.y;
        inner_top_left /= inner_top_left.y;
        inner_top_right /= inner_top_right.y;
        inner_bottom_left /= inner_bottom_left.y;
        inner_bottom_right /= inner_bottom_right.y;
        clip_x(&mut top_left, &mut top_right, EPSILON);
        clip_x(&mut bottom_left, &mut bottom_right, EPSILON);
        clip_x(&mut inner_top_left, &mut inner_top_right, EPSILON);
        clip_x(&mut inner_bottom_left, &mut inner_bottom_right, EPSILON);
        Some(Geometry {
            top_left: self.apply_viewport_transform(&top_left),
            top_right: self.apply_viewport_transform(&top_right),
            bottom_left: self.apply_viewport_transform(&bottom_left),
            bottom_right: self.apply_viewport_transform(&bottom_right),
            inner_top_left: self.apply_viewport_transform(&inner_top_left),
            inner_top_right: self.apply_viewport_transform(&inner_top_right),
            inner_bottom_left: self.apply_viewport_transform(&inner_bottom_left),
            inner_bottom_right: self.apply_viewport_transform(&inner_bottom_right),
        })
    }

    fn apply_viewport_transform(&self, v: &glm::Vec4) -> Point {
        let viewport = self.viewport;
        let mut v = *v;
        v.x = (0.5 - v.x) * (viewport[2] as f32) + (viewport[0] as f32);
        v.z = (0.5 - v.z) * (viewport[3] as f32) + (viewport[1] as f32);
        [v.x as i32, v.z as i32]
    }
}

// convert sectors to glm types for easier handling. All vector calculations are
// performed with f32, but I want to change that at some point
fn wall_to_glm(map: &Map, sector: &Sector, left: &Wall, right: &Wall) -> Geometry<glm::Vec4> {
    let ceil_floor_z = glm::vec2(sector.ceiling_z as f32, sector.floor_z as f32);
    let ceil_floor_diff = map
        .sectors
        .get(left.next_sector)
        .map(|s| s.0)
        .map(|s| glm::vec2(s.ceiling_z as f32, s.floor_z as f32) - ceil_floor_z)
        .unwrap_or(glm::vec2(0.0, 0.0));
    let lx = left.x as f32;
    let ly = left.y as f32;
    let rx = right.x as f32;
    let ry = right.y as f32;
    Geometry {
        top_left: glm::vec4(lx, ly, ceil_floor_z.x, 1.0),
        top_right: glm::vec4(rx, ry, ceil_floor_z.x, 1.0),
        bottom_left: glm::vec4(lx, ly, ceil_floor_z.y, 1.0),
        bottom_right: glm::vec4(rx, ry, ceil_floor_z.y, 1.0),
        inner_top_left: glm::vec4(lx, ly, ceil_floor_z.x + ceil_floor_diff.x, 1.0),
        inner_top_right: glm::vec4(rx, ry, ceil_floor_z.x + ceil_floor_diff.x, 1.0),
        inner_bottom_left: glm::vec4(lx, ly, ceil_floor_z.y + ceil_floor_diff.y, 1.0),
        inner_bottom_right: glm::vec4(rx, ry, ceil_floor_z.y + ceil_floor_diff.y, 1.0),
    }
}

// compute the inverse of the player's transformation
// also known as VIEW transformation/matrix
fn view_transform(player: &Player) -> glm::Mat4 {
    #[rustfmt::skip]
    let Player { pos_x, pos_y, pos_z, angle, .. } = player;
    let posx = *pos_x as f32;
    let posy = *pos_y as f32;
    let posz = *pos_z as f32;
    let transform = glm::translation(&glm::vec3(posx, posy, posz))
        * glm::rotation(angle.to_radians(), &glm::vec3(0.0, 0.0, 1.0));
    glm::inverse(&transform)
}

// compute camera view-to-clip-space transformation
// converts vertices from view-space into their clip-space counterparts
// "clip-space" naming is misleading, as no vertex clipping happens at this
// stage but I can't think of a better name
fn clip_transform() -> glm::Mat4 {
    let scale = 10000.0;
    let scale_z = -150000.0;
    glm::scaling(&glm::vec3(1.0 / scale, 1.0 / scale, 1.0 / scale_z))
}

#[rustfmt::skip]
fn clip_y(left: &mut glm::Vec4, right: &mut glm::Vec4, eps: f32) {
    let t = (eps - left.y) / (right.y - left.y);
    if t > 0.0 && t < 1.0 {
        let clip = glm::lerp(left, right, t);
        if left.y < right.y { *left = clip } else { *right = clip }
    }
}

#[rustfmt::skip]
fn clip_x(left: &mut glm::Vec4, right: &mut glm::Vec4, eps: f32) {
    let t = ((eps - 1.0) - left.x) / (right.x - left.x);
    if t > 0.0 && t < 1.0 {
        let clip = glm::lerp(left, right, t);
        if left.x < right.x { *left = clip } else { *right = clip }
    }
    let t = ((1.0 - eps) - left.x) / (right.x - left.x);
    if t > 0.0 && t < 1.0 {
        let clip = glm::lerp(left, right, t);
        if left.x > right.x { *left = clip } else { *right = clip }
    }
}
