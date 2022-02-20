use crate::{frame, frame::Frame};
use algo::{Coverage, Interval};
use map::{
    player::Player,
    sector::{Sector, SectorId, Wall},
    Map,
};
use nalgebra_glm as glm;
use nalgebra_glm::{DVec4, IVec2};
use std::collections::VecDeque;

mod algo;

const EPSILON: f64 = 1e-4;

// magic scaling factors
const SCALE_X: f64 = 6_000.0;
const SCALE_Y: f64 = 8_000.0;
const SCALE_Z: f64 = 60_000.0;

// debug colors
const BLACK_COLOR: u32 = 0x000000;
const WALL_COLOR: u32 = 0x888888;
const WALL_COLORS: [u32; 4] = [0x888888, 0x777777, 0x666666, 0x555555];
const CEILING_COLOR: u32 = 0x444444;
const FLOOR_COLOR: u32 = 0x2222ff;
const TOP_FRAME_COLOR: u32 = 0x666666;
const BOTTOM_FRAME_COLOR: u32 = 0xaa33aa;

/// Represents a sector in the rendering queue.
#[derive(Debug)]
struct RenderSector {
    id: SectorId,
    interval: Interval,
}

/// Holds wall coordinates
#[derive(Clone, Debug, Default)]
struct NAWall<T> {
    tl: T,
    tr: T,
    bl: T,
    br: T,
    portal_tl: T,
    portal_tr: T,
    portal_bl: T,
    portal_br: T,
}

/// 3D MAP renderer.
#[derive(Debug)]
pub struct Renderer {
    /// Viewport transformation params.
    pub viewport: [i32; 4],
    coverage: Coverage,
    queue: VecDeque<RenderSector>,
    // camera transformation: inv(Tr * Rot)
    camera: glm::DMat4,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            viewport: [0, 0, frame::WIDTH as _, frame::HEIGHT as _],
            coverage: Coverage::new(frame::WIDTH, frame::HEIGHT),
            queue: VecDeque::new(),
            camera: glm::identity(),
        }
    }

    /// Render MAP to the given frame.
    pub fn render(&mut self, map: &Map, frame: &mut Frame) {
        self.init_render(map);
        self.render_sectors(map, frame);
    }

    fn init_render(&mut self, map: &Map) {
        self.camera = compute_camera_normalized(&map.player);
        self.coverage.clear();
        // FIXME(german): edges of the frame are being missed!
        //self.coverage.intersect(0, &Interval::EMPTY);
        self.coverage.intersect(frame::WIDTH - 1, &Interval::EMPTY);
        self.queue.clear();
        self.queue.push_back(RenderSector {
            id: map.player.sector,
            interval: Interval::new(0, frame::WIDTH as i32),
        });
    }

    pub fn render_sectors(&mut self, map: &Map, frame: &mut Frame) {
        while !self.coverage.is_full() {
            let render_sector = match self.queue.pop_back() {
                None => break,
                Some(render_sector) => render_sector,
            };
            let (sector, sector_walls) =
                map.sectors.get(render_sector.id).expect("expected sector");
            for (_, left, right) in sector_walls {
                let nawall_ivec2 = self
                    .wall_to_nawall_dvec4(map, sector, left, right)
                    .and_then(|na| self.wall_to_nawall_ivec2(left, &na));
                if let Some(na) = nawall_ivec2 {
                    if left.next_sector == -1 {
                        self.render_solid_wall(&na, frame, &render_sector.interval);
                    } else {
                        let interval = self.render_portal_wall(&na, frame, &render_sector.interval);
                        if let Some(interval) = interval {
                            self.queue.push_back(RenderSector {
                                id: left.next_sector,
                                interval: interval.intersect(&render_sector.interval),
                            });
                        }
                    }
                }
            }
        }
    }

    #[rustfmt::skip]
    fn render_solid_wall(&mut self, nawall_i2: &NAWall<glm::IVec2>, frame: &mut Frame, int: &Interval) {
        for (top, bot, _, _) in self.lines_iter(nawall_i2, int) {
            self.render_line(&glm::IVec2::new(top.x, i32::MIN), &top, frame, CEILING_COLOR);
            self.render_line(&top, &bot, frame, WALL_COLOR);
            self.render_line(&bot, &glm::IVec2::new(bot.x, i32::MAX), frame, FLOOR_COLOR);
            // mark column as totally covered so it's no longer rendered to
            self.coverage.intersect(top.x as usize, &Interval::EMPTY);
        }
    }

    #[rustfmt::skip]
    fn render_portal_wall(&mut self, nawall_i2: &NAWall<glm::IVec2>, frame: &mut Frame, int: &Interval) -> Option<Interval> {
        let mut xmin = i32::MAX;
        let mut xmax = i32::MIN;
        for (top, bot, portal_top, portal_bot) in self.lines_iter(nawall_i2, int) {
            self.render_line(&glm::IVec2::new(top.x, i32::MIN), &top, frame, CEILING_COLOR);
            // portal
            if top.y < portal_top.y {
                self.render_line(&top, &portal_top, frame, TOP_FRAME_COLOR);
            }
            if portal_bot.y < bot.y {
                self.render_line(&portal_bot, &bot, frame, BOTTOM_FRAME_COLOR);
            }
            self.render_line(&bot, &glm::IVec2::new(bot.x, i32::MAX), frame, FLOOR_COLOR);
            self.coverage.intersect(top.x as usize, &Interval::new(
                top.y.max(portal_top.y),
                bot.y.min(portal_bot.y),
            ));

            xmin = xmin.min(top.x);
            xmax = xmax.max(top.x);
        }
        if xmin < xmax {
            Some(Interval::new(xmin, xmax))
        } else {
            None
        }
    }

    // compute wall coordinates in window pixel coordinates
    #[rustfmt::skip]
    fn wall_to_nawall_ivec2(
        &self,
        wall: &Wall,
        nawall_dvec4: &NAWall<glm::DVec4>,
    ) -> Option<NAWall<glm::IVec2>> {
        let mut nawall_d4 = nawall_dvec4.clone();
        crate::util::clip_y(&mut nawall_d4.tl, &mut nawall_d4.tr, EPSILON);
        crate::util::clip_y(&mut nawall_d4.bl, &mut nawall_d4.br, EPSILON);
        crate::util::clip_y(&mut nawall_d4.portal_tl, &mut nawall_d4.portal_tr, EPSILON);
        crate::util::clip_y(&mut nawall_d4.portal_bl, &mut nawall_d4.portal_br, EPSILON);
        nawall_d4.tl /= nawall_d4.tl.y;
        if nawall_d4.tl.x > 1.0 - EPSILON { return None; } // out bounds (right)
        if nawall_d4.tl.y < EPSILON - 1.0 && nawall_d4.tr.y < EPSILON - 1.0 { return None; } // out bounds (bottom)
        nawall_d4.tr /= nawall_d4.tr.y;
        if nawall_d4.tr.x < EPSILON - 1.0 { return None; } // out bounds (left)
        nawall_d4.bl /= nawall_d4.bl.y;
        if nawall_d4.bl.z > 1.0 - EPSILON && nawall_d4.br.z > 1.0 - EPSILON { return None; } // out bounds (top)
        nawall_d4.br /= nawall_d4.br.y;
        nawall_d4.portal_tl /= nawall_d4.portal_tl.y;
        nawall_d4.portal_tr /= nawall_d4.portal_tr.y;
        nawall_d4.portal_bl /= nawall_d4.portal_bl.y;
        nawall_d4.portal_br /= nawall_d4.portal_br.y;
        crate::util::clip_x(&mut nawall_d4.tl, &mut nawall_d4.tr, EPSILON);
        crate::util::clip_x(&mut nawall_d4.bl, &mut nawall_d4.br, EPSILON);
        crate::util::clip_x(&mut nawall_d4.portal_tl, &mut nawall_d4.portal_tr, EPSILON);
        crate::util::clip_x(&mut nawall_d4.portal_bl, &mut nawall_d4.portal_br, EPSILON);
        let tl = self.compute_vp(&nawall_d4.tl);
        let tr = self.compute_vp(&nawall_d4.tr);
        if tl.x > tr.x { return None; } // ???
        let bl = self.compute_vp(&nawall_d4.bl);
        let br = self.compute_vp(&nawall_d4.br);
        if wall.next_sector == -1 {
            Some(NAWall { tl, tr, bl, br, ..Default::default() })
        } else {
            let portal_tl = self.compute_vp(&nawall_d4.portal_tl);
            let portal_tr = self.compute_vp(&nawall_d4.portal_tr);
            let portal_bl = self.compute_vp(&nawall_d4.portal_bl);
            let portal_br = self.compute_vp(&nawall_d4.portal_br);
            Some(NAWall { tl, tr, bl, br, portal_tl, portal_tr, portal_bl, portal_br })
        }
    }

    #[rustfmt::skip]
    fn wall_to_nawall_dvec4(
        &self,
        map: &Map,
        sector: &Sector,
        left: &Wall,
        right: &Wall,
    ) -> Option<NAWall<glm::DVec4>> {
        let ceiling_floor = glm::vec2(sector.ceiling_z as f64, sector.floor_z as f64);
        let tl = &self.camera * glm::vec4(left.x as f64, left.y as f64, ceiling_floor.x, 1.0);
        let tr = &self.camera * glm::vec4(right.x as f64, right.y as f64, ceiling_floor.x, 1.0);
        if tl.y < EPSILON && tr.y < EPSILON { return None; } // behind
        let bl = &self.camera * glm::vec4(left.x as f64, left.y as f64, ceiling_floor.y, 1.0);
        let br = &self.camera * glm::vec4(right.x as f64, right.y as f64, ceiling_floor.y, 1.0);
        if left.next_sector == -1 {
            Some(NAWall { tl, tr, bl, br, ..Default::default() })
        } else {
            let next_sector = &map.sectors.sectors()[left.next_sector as usize];
            let ceil_d = (next_sector.ceiling_z - sector.ceiling_z) as f64;
            let floor_d = (next_sector.floor_z - sector.floor_z) as f64;
            let portal_tl = &self.camera * glm::vec4(left.x as f64, left.y as f64, ceiling_floor.x + ceil_d, 1.0);
            let portal_tr = &self.camera * glm::vec4(right.x as f64, right.y as f64, ceiling_floor.x + ceil_d, 1.0);
            let portal_bl = &self.camera * glm::vec4(left.x as f64, left.y as f64, ceiling_floor.y + floor_d, 1.0);
            let portal_br = &self.camera * glm::vec4(right.x as f64, right.y as f64, ceiling_floor.y + floor_d, 1.0);
            Some(NAWall { tl, tr, bl, br, portal_tl, portal_tr, portal_bl, portal_br })
        }
    }

    fn lines_iter<'a>(
        &self,
        nawall_i2: &'a NAWall<glm::IVec2>,
        int: &'a Interval,
    ) -> impl Iterator<Item = (IVec2, IVec2, IVec2, IVec2)> + 'a {
        let mut nawall_i2 = nawall_i2.clone();
        let d = nawall_i2.tr.x - nawall_i2.tl.x + 1;
        ((nawall_i2.tl.x)..=(nawall_i2.tr.x))
            .enumerate()
            .filter(move |(_, x)| int.contains(*x))
            .map(move |(i, x)| {
                let mut top = glm::IVec2::new(x, 0);
                let mut bot = glm::IVec2::new(x, 0);
                let mut portal_top = glm::IVec2::new(x, 0);
                let mut portal_bot = glm::IVec2::new(x, 0);
                let n = (i as i32);
                let t = (d - n);
                top.y = ((nawall_i2.tl.y * t + (nawall_i2.tr.y * n)) / d)
                    .clamp(0, frame::HEIGHT as i32);
                bot.y = ((nawall_i2.bl.y * t + (nawall_i2.br.y * n)) / d)
                    .clamp(0, frame::HEIGHT as i32);
                portal_top.y = ((nawall_i2.portal_tl.y * t + (nawall_i2.portal_tr.y * n)) / d)
                    .clamp(0, frame::HEIGHT as i32);
                portal_bot.y = ((nawall_i2.portal_bl.y * t + (nawall_i2.portal_br.y * n)) / d)
                    .clamp(0, frame::HEIGHT as i32);
                (top, bot, portal_top, portal_bot)
            })
    }

    /// Render a vertical line.
    /// (top and bottom are assumed to be vertically aligned).
    #[rustfmt::skip]
    fn render_line(&mut self, top: &IVec2, bottom: &IVec2, frame: &mut Frame, color: u32) -> bool {
        assert_eq!(top.x, bottom.x, "vec2s must be vertically aligned!");
        let mut top = top.clone();
        let mut bot = bottom.clone();
        let interval = self.coverage.column(top.x as usize).intersect(&Interval::new(top.y, bot.y));
        for row in interval.left()..interval.right() {
            frame[row as usize][top.x as usize] = color;
        }
        !interval.is_empty()
    }

    // convert from normalized coordinates back to window pixel coordinates
    fn compute_vp(&self, v: &glm::DVec4) -> glm::IVec2 {
        let vp = self.viewport;
        let mut v = v.clone();
        v.x = (v.x + 1.0) / 2.0 * (vp[2] as f64) + (vp[0] as f64);
        v.z = (v.z + 1.0) / 2.0 * (vp[3] as f64) + (vp[1] as f64);
        glm::vec2(v.x as i32, v.z as i32)
    }
}

fn compute_camera_normalized(player: &Player) -> glm::DMat4 {
    // in Build maps, UP (z) is negative :-)
    let scale = glm::scaling(&glm::vec3(-1.0 / SCALE_X, 1.0 / SCALE_Y, 1.0 / SCALE_Z));
    let posx = player.pos_x as f64;
    let posy = player.pos_y as f64;
    let posz = player.pos_z as f64;
    let angle = player.angle.to_radians() as f64;
    let tr = glm::translation(&glm::vec3(posx, posy, posz));
    let rot = glm::rotation(angle, &glm::vec3(0.0, 0.0, 1.0));
    let camera = glm::inverse(&(tr * rot));
    scale * camera
}
