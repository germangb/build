use crate::{d3::algo::Coverage, frame, frame::Frame};
use algo::Interval;
use map::{
    player::Player,
    sector::{Sector, SectorId, Wall},
    Map,
};
use nalgebra_glm as glm;
use std::collections::VecDeque;

/// support data structures and algos.
mod algo;

const EPSILON: f32 = 1e-3;

#[rustfmt::skip]
cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        const BLACK_COLOR: u32        = unsafe { std::mem::transmute([0x00_u8, 0x00, 0x00, 0xff]) };
        const WALL_COLOR: u32         = unsafe { std::mem::transmute([0x88_u8, 0x88, 0x88, 0xff]) };
        const CEILING_COLOR: u32      = unsafe { std::mem::transmute([0x44_u8, 0x44, 0x44, 0xff]) };
        const FLOOR_COLOR: u32        = unsafe { std::mem::transmute([0x00_u8, 0x00, 0xff, 0xff]) };
        const PORTAL_FRAME_COLOR: u32 = unsafe { std::mem::transmute([0xaa_u8, 0x00, 0xaa, 0xff]) };
    } else {
        const BLACK_COLOR: u32        = 0x000000;
        const WALL_COLOR: u32         = 0x888888;
        const CEILING_COLOR: u32      = 0x444444;
        const FLOOR_COLOR: u32        = 0x0000ff;
        const PORTAL_FRAME_COLOR: u32 = 0xaa00aa;
    }
}

type Point = [i32; 2];
type Segment = [Point; 2];

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
struct Geometry<T, E = ()> {
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
    // extra data
    pub extra: E,
}

/// 3D MAP renderer.
#[derive(Debug)]
pub struct Renderer {
    /// Viewport transformation params.
    pub viewport: [i32; 4],
    coverage: Coverage,
    queue: VecDeque<(SectorId, Interval)>,
    clip_view: glm::Mat4,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            viewport: [0, 0, frame::WIDTH as _, frame::HEIGHT as _],
            coverage: Coverage::new(),
            queue: VecDeque::new(),
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
        self.queue.clear();
        self.queue
            .push_back((map.player.sector, algo::interval(0, frame::WIDTH as _)));
        // do bfs traversal
        while let Some((sector_id, interval)) = self.queue.pop_back() {
            let (sector, walls) = map.sectors.get(sector_id).unwrap();
            // sort walls from closest to farthest as a workaround to handle non-convex
            // sector geometry
            let mut walls: Vec<_> = walls
                .filter_map(|(_, left, right)| {
                    self.project_wall(map, sector, left, right)
                        .map(|p| (left, p))
                })
                .collect();
            walls.sort_by_cached_key(|(_, p)| p.extra);
            for (wall, points) in walls {
                if wall.next_sector == -1 {
                    self.render_solid_wall(&points, &interval, frame);
                } else {
                    let clip_interval = algo::intersect(
                        &self.render_portal_wall(&points, &interval, frame),
                        &interval,
                    );
                    if clip_interval.is_some() {
                        self.queue.push_back((wall.next_sector, clip_interval));
                    }
                }
            }
            // exit early if all pixels in the frame have been drawn
            if self.coverage.is_full() {
                break;
            }
        }
    }

    fn render_solid_wall<E>(
        &mut self,
        geometry: &Geometry<Point, E>,
        interval: &Interval,
        frame: &mut Frame,
    ) {
        for (i, wall, _) in wall_lines_iter(geometry, interval) {
            // wall
            #[rustfmt::skip]
            let color = if i == 0 { BLACK_COLOR } else { WALL_COLOR };
            self.render_line(&wall, color, frame);
            #[rustfmt::skip] self.render_line(&[[wall[1][0], wall[1][1] - 1], wall[1]], BLACK_COLOR, frame);
            #[rustfmt::skip] self.render_line(&[wall[0], [wall[0][0], wall[0][1] + 1]], BLACK_COLOR, frame);
            // ceiling
            self.render_line(&[[wall[0][0], 0], wall[0]], CEILING_COLOR, frame);
            // floor
            #[rustfmt::skip] self.render_line(&[wall[1], [wall[1][0], frame::HEIGHT as _]], FLOOR_COLOR, frame);
            // update pixel coverage of the column to be 100%
            // no more pixels will be rendered on this column
            self.coverage.intersect_column(wall[0][0] as _, &None);
        }
    }

    fn render_portal_wall<E>(
        &mut self,
        geometry: &Geometry<Point, E>,
        interval: &Interval,
        frame: &mut Frame,
    ) -> Interval {
        // FIXME(german): Hack: has any pixel been drawn?
        let mut drawn_pixels = false;
        let mut portal_interval = [frame::WIDTH as i32, 0];

        // compute if the wall has either a top-frame, bottom-frame, or both.
        let has_top_frame = geometry.top_left[1] < geometry.inner_top_left[1];
        let has_bottom_frame = geometry.bottom_left[1] > geometry.inner_bottom_left[1];
        let wall_lines = wall_lines_iter(geometry, interval);
        for (i, wall, portal) in wall_lines {
            // portal
            #[rustfmt::skip] self.render_line(&[[portal[1][0], portal[1][1] - 1], portal[1]], BLACK_COLOR, frame);
            #[rustfmt::skip] self.render_line(&[portal[0], [portal[0][0], portal[0][1] + 1]], BLACK_COLOR, frame);
            // frames
            if has_top_frame || has_bottom_frame {
                if has_top_frame {
                    #[rustfmt::skip]
                    let color = if i == 0 { BLACK_COLOR } else { WALL_COLOR };
                    self.render_line(&[wall[0], portal[0]], color, frame);
                    #[rustfmt::skip]
                    self.render_line(&[wall[0], [wall[0][0], wall[0][1] + 1]], BLACK_COLOR, frame);
                }
                if has_bottom_frame {
                    #[rustfmt::skip]
                    let color = if i == 0 { BLACK_COLOR } else { PORTAL_FRAME_COLOR };
                    self.render_line(&[portal[1], wall[1]], color, frame);
                    #[rustfmt::skip]
                    self.render_line(&[[wall[1][0], wall[1][1] - 1], wall[1]], BLACK_COLOR, frame);
                }
            }
            // ceiling
            self.render_line(&[[wall[0][0], 0], wall[0]], CEILING_COLOR, frame);
            // floor
            #[rustfmt::skip]
            self.render_line(&[wall[1], [wall[1][0], frame::HEIGHT as _]], FLOOR_COLOR, frame);
            // update column pixel coverage
            self.coverage.intersect_column(
                wall[0][0] as _,
                &algo::interval(portal[0][1] + 1, portal[1][1] - 1),
            );
            portal_interval[0] = portal_interval[0].min(wall[0][0]);
            portal_interval[1] = portal_interval[1].max(wall[0][0] + 1);
            drawn_pixels = true;
        }
        if drawn_pixels {
            algo::interval(portal_interval[0], portal_interval[1])
        } else {
            None
        }
    }

    fn render_line(&mut self, segment: &Segment, color: u32, frame: &mut Frame) {
        let [x0, y0] = segment[0];
        let [x1, y1] = segment[1];
        assert_eq!(x0, x1);
        // only render those pixels that haven't been painted yet.
        let current_coverage = self.coverage.get_column(x0 as _);
        if let Some([y0, y1]) = algo::intersect(&current_coverage, &algo::interval(y0, y1)) {
            for y in y0..y1 {
                frame[y as usize][x0 as usize] = color;
            }
        }
    }

    // compute the coordinates of a sector wall in the viewport
    // if the wall is entirely behind the player, return None
    fn project_wall(
        &self,
        map: &Map,
        sector: &Sector,
        left: &Wall,
        right: &Wall,
    ) -> Option<Geometry<Point, i32>> {
        let Geometry {
            mut top_left,
            mut top_right,
            mut bottom_left,
            mut bottom_right,
            mut inner_top_left,
            mut inner_top_right,
            mut inner_bottom_left,
            mut inner_bottom_right,
            ..
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
        //let closest = (top_left.y.min(top_right.y) * 100000.0) as _; //
        // FIXME(german): Hack!!
        let closest = ((top_left.y * top_left.y + top_left.x * top_left.x)
            .min(top_right.y * top_right.y + top_right.x * top_right.x)
            * 100000.0) as _; // FIXME(german): Hack!!
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
            extra: closest,
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

/// Returns an iterator of vertical lines spanning the passed geometry, clipped
/// to the given horizontal interval. Each Item contain a vertical line spanning
/// the entire wall (for portal walls, this includes the portal and top and
/// bottom frames), followed by a line spanning only the region of the portal.
fn wall_lines_iter<'a, E>(
    geometry: &'a Geometry<Point, E>,
    interval: &'a Interval,
) -> impl Iterator<Item = (usize, Segment, Segment)> + 'a {
    let d = geometry.top_right[0] - geometry.top_left[0];
    let x0 = (geometry.top_left[0]).max(0).min(frame::WIDTH as i32);
    let x1 = (geometry.top_right[0]).max(0).min(frame::WIDTH as i32);
    (x0..x1)
        .enumerate()
        .filter(move |(_, x)| algo::contains(interval, *x))
        .map(move |(i, x)| {
            let n = x - geometry.top_left[0];
            let mut mid_y0 =
                geometry.top_left[1] + n * (geometry.top_right[1] - geometry.top_left[1]) / d;
            let mut mid_y1 = geometry.bottom_left[1]
                + n * (geometry.bottom_right[1] - geometry.bottom_left[1]) / d;
            let mut top_y0 = geometry.inner_top_left[1]
                + n * (geometry.inner_top_right[1] - geometry.inner_top_left[1]) / d;
            let mut top_y1 = geometry.inner_bottom_left[1]
                + n * (geometry.inner_bottom_right[1] - geometry.inner_bottom_left[1]) / d;
            mid_y0 = mid_y0.max(0).min(frame::HEIGHT as i32);
            mid_y1 = mid_y1.max(0).min(frame::HEIGHT as i32);
            top_y0 = top_y0.max(0).min(frame::HEIGHT as i32);
            top_y1 = top_y1.max(0).min(frame::HEIGHT as i32);
            let mid = [[x, mid_y0], [x, mid_y1]]; // portal
            let top = [[x, top_y0.max(mid_y0)], [x, top_y1.min(mid_y1)]];
            (i, mid, top)
        })
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
        extra: (),
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
    let scale_x = 10_000.0;
    let scale_y = 10_000.0;
    let scale_z = -150_000.0;
    glm::scaling(&glm::vec3(1.0 / scale_x, 1.0 / scale_y, 1.0 / scale_z))
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
