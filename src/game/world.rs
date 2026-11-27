//! The Undercroft's ground: a map of 8x8 tiles, built by code for each
//! stage, and what it looks like: stone, earth, wooden platforms, ladders,
//! iron spikes, black water, crumbling ledges and the hidden platforms only
//! the visor's light shows. Also the skies behind it: the moon, the far
//! ruins, rain, lightning and fog.
use super::common::{hash, HUD_H};
use crate::fb::{Fb, H, W};
use crate::palette::*;

pub const T: i32 = 8;

pub const EMPTY: u8 = 0;
/// Solid stone or earth.
pub const SOLID: u8 = 1;
/// A platform you can jump up through and stand on.
pub const PLAT: u8 = 2;
pub const LADDER: u8 = 3;
pub const SPIKES: u8 = 4;
pub const WATER: u8 = 5;
/// A ledge that falls a moment after NIRUX stands on it.
pub const CRUMBLE: u8 = 6;
/// A platform that is there but unseen until the lamp shows it.
pub const HIDDEN: u8 = 7;
/// Solid brickwork (drawn as masonry rather than earth).
pub const BRICK: u8 = 8;
/// The top of a ladder that is also a platform.
pub const LADDER_TOP: u8 = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Graveyard,
    Marsh,
}

/// Scenery drawn behind the action; it never collides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deco {
    Grave,
    Cross,
    Tree,
    Fence,
    Chapel,
    Column,
    Lantern,
    Reeds,
    Hut,
    Stump,
    Skull,
    Tomb,
}

pub struct Map {
    pub w: i32,
    pub h: i32,
    pub t: Vec<u8>,
    pub deco: Vec<(i32, i32, Deco)>,
    /// Hidden platforms the lamp has shown (they stay shown).
    pub seen: Vec<bool>,
    /// Crumbling tiles: frames since stood on (0 = untouched).
    pub crumble: Vec<u16>,
}

impl Map {
    pub fn new(w: i32, h: i32) -> Map {
        let n = (w * h) as usize;
        Map {
            w,
            h,
            t: vec![EMPTY; n],
            deco: Vec::new(),
            seen: vec![false; n],
            crumble: vec![0; n],
        }
    }

    /// Keeps only the first w columns.
    pub fn crop(&mut self, w: i32) {
        let w = w.min(self.w);
        let old = std::mem::take(&mut self.t);
        self.t = (0..self.h)
            .flat_map(|y| old[(y * self.w) as usize..(y * self.w + w) as usize].to_vec())
            .collect();
        self.seen = vec![false; (w * self.h) as usize];
        self.crumble = vec![0; (w * self.h) as usize];
        self.w = w;
    }

    #[inline]
    pub fn at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || x >= self.w {
            return SOLID;
        }
        if y < 0 || y >= self.h {
            return EMPTY;
        }
        self.t[(y * self.w + x) as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, v: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.t[(y * self.w + x) as usize] = v;
        }
    }

    /// The tile under a point in pixels.
    pub fn px(&self, x: f32, y: f32) -> u8 {
        self.at((x / T as f32).floor() as i32, (y / T as f32).floor() as i32)
    }

    pub fn solid(v: u8) -> bool {
        v == SOLID || v == BRICK
    }

    /// Tiles you can stand on from above.
    pub fn floor(v: u8) -> bool {
        Self::solid(v) || v == PLAT || v == CRUMBLE || v == HIDDEN || v == LADDER_TOP
    }

    pub fn ladder(v: u8) -> bool {
        v == LADDER || v == LADDER_TOP
    }

    // Building.

    pub fn fill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, v: u8) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.set(x, y, v);
            }
        }
    }

    /// Ground from column x0 to x1 with its top at row `top`.
    pub fn ground(&mut self, x0: i32, x1: i32, top: i32) {
        self.fill(x0, top, x1, self.h - 1, SOLID);
    }

    pub fn bricks(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        self.fill(x0, y0, x1, y1, BRICK);
    }

    pub fn plat(&mut self, x: i32, y: i32, len: i32) {
        self.fill(x, y, x + len - 1, y, PLAT);
    }

    pub fn crumbles(&mut self, x: i32, y: i32, len: i32) {
        self.fill(x, y, x + len - 1, y, CRUMBLE);
    }

    pub fn hidden(&mut self, x: i32, y: i32, len: i32) {
        self.fill(x, y, x + len - 1, y, HIDDEN);
    }

    /// A ladder from row y0 (its top, which you can stand on) down to y1.
    pub fn ladder_at(&mut self, x: i32, y0: i32, y1: i32) {
        self.set(x, y0, LADDER_TOP);
        self.fill(x, y0 + 1, x, y1, LADDER);
    }

    pub fn water(&mut self, x0: i32, x1: i32, top: i32) {
        for x in x0..=x1 {
            for y in top..self.h {
                if self.at(x, y) == EMPTY {
                    self.set(x, y, WATER);
                }
            }
        }
    }

    pub fn spikes(&mut self, x0: i32, x1: i32, y: i32) {
        self.fill(x0, y, x1, y, SPIKES);
    }

    pub fn deco(&mut self, x: i32, y: i32, d: Deco) {
        self.deco.push((x, y, d));
    }

    /// The row of the first floor at or below row y in column x.
    pub fn top_at(&self, x: i32, from: i32) -> i32 {
        (from.max(0)..self.h)
            .find(|&y| Self::floor(self.at(x, y)))
            .unwrap_or(self.h)
    }

    /// A step of the crumbling ledges; returns the tiles that fell.
    pub fn update(&mut self) -> Vec<(i32, i32)> {
        let mut fell = Vec::new();
        for i in 0..self.crumble.len() {
            let c = self.crumble[i];
            if c > 0 {
                self.crumble[i] = c + 1;
                if c == 34 && self.t[i] == CRUMBLE {
                    self.t[i] = EMPTY;
                    fell.push((i as i32 % self.w, i as i32 / self.w));
                }
                // They grow back after a while, so nobody is stuck.
                if c > 420 {
                    self.crumble[i] = 0;
                    if self.t[i] == EMPTY {
                        self.t[i] = CRUMBLE;
                    }
                }
            }
        }
        fell
    }

    pub fn touch_crumble(&mut self, x: i32, y: i32) {
        if self.at(x, y) == CRUMBLE {
            let i = (y * self.w + x) as usize;
            if self.crumble[i] == 0 {
                self.crumble[i] = 1;
            }
        }
    }
}

// Drawing.

const BAYER: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

/// True for a share `k` (0 to 16) of pixels in an ordered pattern.
#[inline]
pub fn dith(x: i32, y: i32, k: i32) -> bool {
    (BAYER[(y & 3) as usize][(x & 3) as usize] as i32) < k
}

pub struct Palette {
    pub earth: [u8; 3],
    pub top: [u8; 2],
    pub stone: [u8; 3],
    pub wood: [u8; 2],
    pub sky: [u8; 3],
    pub far: u8,
    pub mid: u8,
    pub water: [u8; 2],
}

pub fn palette(th: Theme) -> Palette {
    match th {
        Theme::Graveyard => Palette {
            earth: [BROWN, BLACK, ORANGE],
            top: [GREEN, LGREEN],
            stone: [DGREY, GREY, LGREY],
            wood: [BROWN, ORANGE],
            sky: [BLACK, BLUE, PURPLE],
            far: BLUE,
            mid: BLACK,
            water: [BLUE, LBLUE],
        },
        Theme::Marsh => Palette {
            earth: [BROWN, BLACK, GREEN],
            top: [GREEN, LGREEN],
            stone: [DGREY, GREY, LGREEN],
            wood: [BROWN, ORANGE],
            sky: [BLACK, GREEN, GREEN],
            far: DGREY,
            mid: BLACK,
            water: [BLACK, GREEN],
        },
    }
}

/// The sky, the moon and two layers of far scenery, scrolled by the camera.
pub fn draw_sky(fb: &mut Fb, th: Theme, cx: f32, cy: f32, t: u32, flash: u8) {
    let p = palette(th);
    // A dithered gradient from black at the top.
    for y in HUD_H..H {
        let k = ((y - HUD_H) * 28 / (H - HUD_H) - 12).clamp(0, 10);
        for x in 0..W {
            let c = if dith(x, y, k) { p.sky[1] } else { p.sky[0] };
            fb.set(x, y, c);
        }
    }
    if flash > 0 {
        return;
    }
    // Stars in the dark part.
    for i in 0..40 {
        let h = hash(i, 7);
        let x = ((h % 997) as f32 - cx * 0.02).rem_euclid(W as f32) as i32;
        let y = HUD_H + (h >> 10) as i32 % 70;
        if (t / 7 + h) % 23 != 0 {
            fb.set(x, y, if h % 3 == 0 { WHITE } else { GREY });
        }
    }
    // The moon.
    let (mx, my) = (250 - (cx * 0.03) as i32 % 40, 44 - (cy * 0.02) as i32);
    let mr = 18;
    for y in -mr..=mr {
        for x in -mr..=mr {
            let d = x * x + y * y;
            if d <= mr * mr {
                let shade = (x + y) * 8 / mr;
                let c = if d > (mr - 2) * (mr - 2) || dith(x, y, 6 - shade) {
                    LGREY
                } else {
                    WHITE
                };
                fb.set(mx + x, my + y, c);
            }
        }
    }
    for (ox, oy, r) in [(-6, -4, 4), (5, 6, 3), (7, -7, 2), (-4, 9, 2)] {
        fb.disc(mx + ox, my + oy, r, GREY);
        fb.set(mx + ox - 1, my + oy - 1, LGREY);
    }
    // Far ruins: towers and broken walls.
    let base = H - 70 - (cy * 0.1) as i32;
    for x in 0..W {
        let wx = x as f32 + cx * 0.15;
        let k = (wx / 4.0) as i32;
        let h = hash(k / 6, 3);
        let tower = h % 5 == 0;
        let top = if tower {
            base - 26 - (h >> 8) as i32 % 18
        } else {
            base - (h >> 8) as i32 % 10
        };
        let crenel = !tower && (k % 2 == 0);
        let y0 = if crenel { top - 3 } else { top };
        let misty = th == Theme::Marsh;
        for y in y0.max(HUD_H)..H {
            if !misty || dith(x, y, 9 + (y - y0) / 8) {
                fb.set(x, y, p.far);
            }
        }
        // A lit window now and then.
        if tower && (wx as i32 % 24) == 3 && (h >> 4) % 3 == 0 {
            fb.set(x, top + 8, if t % 90 < 80 { YELLOW } else { ORANGE });
            fb.set(x, top + 9, ORANGE);
        }
    }
    // Middle: hills and dead trees.
    let base = H - 40 - (cy * 0.25) as i32;
    for x in 0..W {
        let wx = x as f32 + cx * 0.4;
        let hy = base + ((wx * 0.02).sin() * 8.0 + (wx * 0.051).sin() * 4.0) as i32;
        for y in hy.max(HUD_H)..H {
            fb.set(x, y, p.mid);
        }
        let k = (wx / 70.0).floor() as i32;
        let tx = wx - k as f32 * 70.0;
        let hh = hash(k, 11);
        if hh % 2 == 0 && (20.0..22.0).contains(&tx) {
            for y in (hy - 26)..hy {
                fb.set(x, y, p.mid);
            }
        }
        if hh % 2 == 0 && (14.0..28.0).contains(&tx) {
            // Bare branches.
            let d = (tx - 21.0).abs() as i32;
            fb.set(x, hy - 18 - d / 2, p.mid);
            fb.set(x, hy - 24 + d / 3, p.mid);
        }
    }
}

/// Rain over everything: short diagonal streaks.
pub fn draw_rain(fb: &mut Fb, t: u32, amount: u32, cx: f32) {
    for i in 0..amount as i32 {
        let h = hash(i, 99);
        let speed = 5 + (h % 3) as i32;
        let x = ((h % 640) as i32 - (t as i32 * 2) - cx as i32 / 2).rem_euclid(W + 40) - 20;
        let y = ((h >> 12) as i32 % 260 + t as i32 * speed).rem_euclid(H + 20) - 10;
        let c = if h % 4 == 0 { LGREY } else { LBLUE };
        for k in 0..4 {
            fb.set(x - k / 2, y + k, c);
        }
    }
}

/// Fog: dithered bands drifting along the bottom.
pub fn draw_fog(fb: &mut Fb, t: u32, cx: f32, from: i32) {
    for y in from..H {
        let k = ((y - from) * 8 / (H - from).max(1)).min(5);
        for x in 0..W {
            let wx = x + (cx * 1.2) as i32 + (t / 3) as i32;
            let wave = ((wx as f32 * 0.03).sin() * 3.0) as i32;
            if dith(wx, y, k + wave) && (x + y) % 2 == 0 {
                fb.set(x, y, GREY);
            }
        }
    }
}

/// The tiles in view; `lamp` is the lit cone (x, y, dir) if the lamp shines.
pub fn draw_tiles(fb: &mut Fb, m: &Map, th: Theme, ox: i32, oy: i32, t: u32) {
    let p = palette(th);
    let tx0 = (ox / T).max(0);
    let ty0 = ((oy + HUD_H) / T).max(0);
    let tx1 = ((ox + W) / T + 1).min(m.w - 1);
    let ty1 = ((oy + H) / T + 1).min(m.h - 1);
    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            let v = m.at(tx, ty);
            if v == EMPTY {
                continue;
            }
            let sx = tx * T - ox;
            let sy = ty * T - oy;
            draw_tile(fb, m, &p, tx, ty, v, sx, sy, t);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_tile(fb: &mut Fb, m: &Map, p: &Palette, tx: i32, ty: i32, v: u8, sx: i32, sy: i32, t: u32) {
    let above = m.at(tx, ty - 1);
    match v {
        SOLID => {
            let open = !Map::solid(above);
            for y in 0..T {
                for x in 0..T {
                    let (wx, wy) = (tx * T + x, ty * T + y);
                    let h = hash(wx / 2, wy / 2);
                    let mut c = if h % 7 == 0 {
                        p.earth[1]
                    } else if h % 11 == 0 {
                        p.earth[2]
                    } else {
                        p.earth[0]
                    };
                    // Darker deeper down.
                    let depth = ty - m.top_at(tx, 0);
                    if depth > 2 && dith(wx, wy, (depth - 2).min(4) * 3) {
                        c = BLACK;
                    }
                    if open && y < 3 {
                        let tuft = hash(wx, 5) % 3 == 0;
                        c = if y == 0 && !tuft {
                            CLEAR
                        } else if y == 0 || (y == 1 && tuft) {
                            p.top[1]
                        } else {
                            p.top[0]
                        };
                        if y == 0 && tuft {
                            fb.set(sx + x, sy + y - 1, p.top[0]);
                        }
                    }
                    if c != CLEAR {
                        fb.set(sx + x, sy + y, c);
                    }
                }
            }
        }
        BRICK => {
            for y in 0..T {
                for x in 0..T {
                    let (wx, wy) = (tx * T + x, ty * T + y);
                    let row = wy / 4;
                    let mortar = wy % 4 == 3 || (wx + if row % 2 == 0 { 0 } else { 4 }) % 8 == 0;
                    let h = hash(wx / 4 + row * 31, row);
                    let c = if mortar {
                        BLACK
                    } else if wy % 4 == 0 {
                        p.stone[2]
                    } else if h % 5 == 0 {
                        p.stone[0]
                    } else {
                        p.stone[1]
                    };
                    fb.set(sx + x, sy + y, c);
                }
            }
            if !Map::solid(above) {
                fb.hline(sx, sx + T - 1, sy, p.stone[2]);
                if hash(tx, ty) % 3 == 0 {
                    fb.set(sx + 3, sy - 1, p.top[0]);
                }
            }
        }
        PLAT | CRUMBLE => {
            let i = (ty * m.w + tx) as usize;
            let c = m.crumble.get(i).copied().unwrap_or(0);
            let shake = if c > 0 && c < 34 {
                ((t / 2) % 2) as i32
            } else {
                0
            };
            let (a, b) = if v == CRUMBLE {
                (p.stone[0], p.stone[1])
            } else {
                (p.wood[0], p.wood[1])
            };
            fb.hline(sx, sx + T - 1, sy + shake, b);
            fb.rect(sx, sy + 1 + shake, T, 3, a);
            fb.set(sx + (hash(tx, 1) % 6) as i32 + 1, sy + 2 + shake, BLACK);
            if v == CRUMBLE {
                fb.set(sx + 2, sy + 3 + shake, BLACK);
                fb.set(sx + 5, sy + 1 + shake, BLACK);
            } else if m.at(tx - 1, ty) != PLAT || m.at(tx + 1, ty) != PLAT {
                // Supports at the ends.
                fb.line(sx + 3, sy + 4, sx + 3, sy + 7, BLACK);
            }
        }
        HIDDEN => {
            let i = (ty * m.w + tx) as usize;
            if m.seen[i] {
                fb.hline(sx, sx + T - 1, sy, LGREEN);
                for x in 0..T {
                    if (x + t as i32 / 4) % 3 == 0 {
                        fb.set(sx + x, sy + 1, GREEN);
                    }
                }
                fb.hline(sx, sx + T - 1, sy + 2, GREEN);
            } else if (t / 4 + hash(tx, ty)) % 37 == 0 {
                // A faint shimmer, so a sharp eye knows something is there.
                fb.set(sx + (hash(tx, t as i32 / 40) % 8) as i32, sy + 1, DGREY);
            }
        }
        LADDER | LADDER_TOP => {
            fb.line(sx + 1, sy, sx + 1, sy + T - 1, p.wood[0]);
            fb.line(sx + 6, sy, sx + 6, sy + T - 1, p.wood[0]);
            fb.hline(sx + 1, sx + 6, sy + 2, p.wood[1]);
            fb.hline(sx + 1, sx + 6, sy + 6, p.wood[1]);
        }
        SPIKES => {
            fb.hline(sx, sx + T - 1, sy + T - 2, DGREY);
            fb.hline(sx, sx + T - 1, sy + T - 1, BLACK);
            for k in [1, 4, 7] {
                fb.line(sx + k, sy + 1, sx + k, sy + T - 2, GREY);
                fb.set(sx + k, sy, LGREY);
            }
        }
        WATER => {
            let surface = above != WATER;
            for y in 0..T {
                for x in 0..T {
                    let (wx, wy) = (tx * T + x, ty * T + y);
                    let mut c = p.water[0];
                    if surface && y < 2 {
                        let wave = ((wx as f32 * 0.4 + t as f32 * 0.12).sin() * 1.2) as i32;
                        if y == (1 + wave).clamp(0, 1) {
                            c = p.water[1];
                        }
                    } else if dith(wx + (t / 6) as i32, wy, 2) {
                        c = p.water[1];
                    } else if dith(wx, wy, 8) {
                        c = BLACK;
                    }
                    fb.set(sx + x, sy + y, c);
                }
            }
        }
        _ => {}
    }
}

/// Scenery behind the action.
pub fn draw_deco(fb: &mut Fb, m: &Map, th: Theme, ox: i32, oy: i32, t: u32) {
    let p = palette(th);
    for &(x, y, d) in &m.deco {
        let sx = x - ox;
        let sy = y - oy;
        if !(-60..=W + 60).contains(&sx) {
            continue;
        }
        match d {
            Deco::Grave => {
                fb.rect(sx - 5, sy - 12, 11, 12, p.stone[1]);
                fb.disc(sx, sy - 12, 5, p.stone[1]);
                fb.rect(sx - 5, sy - 13, 11, 1, p.stone[1]);
                fb.line(sx - 5, sy - 13, sx - 5, sy - 1, p.stone[2]);
                fb.set(sx - 3, sy - 15, p.stone[2]);
                fb.set(sx - 2, sy - 16, p.stone[2]);
                fb.hline(sx - 2, sx + 2, sy - 10, p.stone[0]);
                fb.hline(sx - 2, sx + 1, sy - 7, p.stone[0]);
                fb.hline(sx - 6, sx + 6, sy - 1, BLACK);
            }
            Deco::Cross => {
                fb.rect(sx - 1, sy - 18, 3, 18, p.stone[1]);
                fb.rect(sx - 6, sy - 14, 13, 3, p.stone[1]);
                fb.line(sx - 1, sy - 18, sx - 1, sy - 1, p.stone[2]);
                fb.hline(sx - 6, sx + 6, sy - 14, p.stone[2]);
            }
            Deco::Tomb => {
                fb.rect(sx - 12, sy - 10, 24, 10, p.stone[1]);
                fb.hline(sx - 13, sx + 12, sy - 11, p.stone[2]);
                fb.hline(sx - 13, sx + 12, sy - 10, p.stone[0]);
                fb.rect(sx - 4, sy - 8, 8, 6, p.stone[0]);
                fb.hline(sx - 12, sx + 12, sy - 1, BLACK);
            }
            Deco::Tree => {
                let c = BLACK;
                fb.rect(sx - 2, sy - 40, 5, 40, c);
                fb.line(sx, sy - 30, sx - 14, sy - 44, c);
                fb.line(sx + 1, sy - 30, sx - 13, sy - 44, c);
                fb.line(sx, sy - 24, sx + 16, sy - 38, c);
                fb.line(sx + 1, sy - 24, sx + 17, sy - 38, c);
                fb.line(sx - 8, sy - 38, sx - 10, sy - 50, c);
                fb.line(sx + 10, sy - 32, sx + 18, sy - 30, c);
                fb.line(sx, sy - 40, sx + 4, sy - 54, c);
                fb.line(sx - 3, sy - 2, sx - 6, sy, c);
                fb.line(sx + 3, sy - 2, sx + 6, sy, c);
            }
            Deco::Fence => {
                for k in 0..5 {
                    let x = sx - 16 + k * 8;
                    fb.line(x, sy - 14, x, sy - 1, DGREY);
                    fb.set(x, sy - 15, GREY);
                }
                fb.hline(sx - 16, sx + 16, sy - 11, DGREY);
                fb.hline(sx - 16, sx + 16, sy - 5, DGREY);
            }
            Deco::Chapel => {
                // A ruined chapel with a bell in its tower.
                let c = p.far;
                fb.rect(sx - 24, sy - 34, 48, 34, BLACK);
                fb.rect(sx - 23, sy - 33, 46, 33, c);
                fb.poly(
                    &[
                        ((sx - 28) as f32, (sy - 34) as f32),
                        (sx as f32, (sy - 52) as f32),
                        ((sx + 28) as f32, (sy - 34) as f32),
                    ],
                    c,
                );
                fb.rect(sx - 6, sy - 70, 12, 22, c);
                fb.poly(
                    &[
                        ((sx - 8) as f32, (sy - 70) as f32),
                        (sx as f32, (sy - 84) as f32),
                        ((sx + 8) as f32, (sy - 70) as f32),
                    ],
                    c,
                );
                fb.rect(sx - 3, sy - 66, 6, 7, BLACK);
                let swing = ((t as f32 * 0.05).sin() * 1.5) as i32;
                fb.disc(sx + swing, sy - 62, 2, YELLOW);
                fb.rect(sx - 4, sy - 24, 8, 24, BLACK);
                fb.disc(sx, sy - 24, 4, BLACK);
                for wx in [-16, 12] {
                    fb.rect(
                        sx + wx,
                        sy - 26,
                        4,
                        9,
                        if t % 120 < 110 { PURPLE } else { LRED },
                    );
                    fb.disc(sx + wx + 1, sy - 26, 2, PURPLE);
                }
            }
            Deco::Column => {
                fb.rect(sx - 4, sy - 30, 9, 30, p.stone[1]);
                fb.line(sx - 4, sy - 30, sx - 4, sy - 1, p.stone[2]);
                fb.line(sx + 4, sy - 30, sx + 4, sy - 1, p.stone[0]);
                fb.rect(sx - 6, sy - 32, 13, 3, p.stone[2]);
                // Broken top.
                fb.set(sx + 3, sy - 33, p.stone[1]);
                fb.set(sx - 2, sy - 34, p.stone[1]);
            }
            Deco::Lantern => {
                fb.line(sx, sy - 24, sx, sy - 1, BLACK);
                fb.rect(sx - 2, sy - 30, 5, 6, BLACK);
                let flick = (t / 5 + x as u32) % 7 < 5;
                fb.rect(sx - 1, sy - 29, 3, 4, if flick { YELLOW } else { ORANGE });
                fb.set(sx, sy - 31, DGREY);
            }
            Deco::Reeds => {
                for k in 0..6 {
                    let x = sx - 8 + k * 3;
                    let h = 8 + (hash(x, 3) % 8) as i32;
                    let sway = ((t as f32 * 0.04 + k as f32).sin() * 1.5) as i32;
                    fb.line(x, sy - 1, x + sway, sy - h, p.top[0]);
                    fb.set(x + sway, sy - h - 1, BROWN);
                    fb.set(x + sway, sy - h - 2, BROWN);
                }
            }
            Deco::Hut => {
                fb.rect(sx - 16, sy - 22, 32, 22, p.wood[0]);
                for k in 0..4 {
                    fb.hline(sx - 16, sx + 15, sy - 20 + k * 6, BLACK);
                }
                fb.poly(
                    &[
                        ((sx - 20) as f32, (sy - 22) as f32),
                        (sx as f32, (sy - 36) as f32),
                        ((sx + 20) as f32, (sy - 22) as f32),
                    ],
                    DGREY,
                );
                fb.rect(sx - 3, sy - 14, 6, 14, BLACK);
                fb.rect(
                    sx + 7,
                    sy - 16,
                    4,
                    4,
                    if t % 140 < 120 { YELLOW } else { ORANGE },
                );
            }
            Deco::Stump => {
                fb.rect(sx - 4, sy - 7, 9, 7, p.wood[0]);
                fb.hline(sx - 4, sx + 4, sy - 8, p.wood[1]);
                fb.set(sx, sy - 6, BLACK);
            }
            Deco::Skull => {
                fb.disc(sx, sy - 3, 2, LGREY);
                fb.set(sx - 1, sy - 3, BLACK);
                fb.set(sx + 1, sy - 3, BLACK);
                fb.hline(sx - 1, sx + 1, sy - 1, GREY);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_ladders_and_crumbling() {
        let mut m = Map::new(20, 10);
        m.ground(0, 19, 8);
        m.ladder_at(5, 4, 7);
        m.crumbles(10, 5, 2);
        assert!(Map::floor(m.at(3, 8)));
        assert_eq!(m.top_at(5, 0), 4);
        assert!(Map::ladder(m.at(5, 6)));
        assert_eq!(m.at(-1, 3), SOLID);
        m.touch_crumble(10, 5);
        let mut fell = Vec::new();
        for _ in 0..40 {
            fell.extend(m.update());
        }
        assert_eq!(fell, vec![(10, 5)]);
        assert_eq!(m.at(10, 5), EMPTY);
        for _ in 0..400 {
            m.update();
        }
        assert_eq!(m.at(10, 5), CRUMBLE);
    }
}
