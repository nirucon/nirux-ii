//! What every stage shares: the random numbers, explosions and particles,
//! screen shake, floating score texts, the player's stats, the HUD and the
//! stars.
// Shared with the first game; not every helper is used here.
#![allow(dead_code)]
use crate::fb::{Fb, H, W};
use crate::palette::*;
use crate::sfx::Sfx;

pub const HUD_H: i32 = 12;

#[derive(Clone, Debug)]
pub struct Rng(pub u32);

impl Rng {
    pub fn new(seed: u32) -> Rng {
        Rng(seed.max(1))
    }
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// 0.0 to 1.0.
    pub fn f(&mut self) -> f32 {
        (self.next() >> 8) as f32 / (1u32 << 24) as f32
    }
    /// -1.0 to 1.0.
    pub fn s(&mut self) -> f32 {
        self.f() * 2.0 - 1.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
    pub fn chance(&mut self, p: f32) -> bool {
        self.f() < p
    }
}

/// Stable per-position noise for drawing details.
pub fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9e37_79b1) ^ (y as u32).wrapping_mul(0x85eb_ca77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pk {
    Spark,
    Debris,
    Smoke,
    Ring,
    Flash,
}

#[derive(Clone, Debug)]
pub struct Part {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: u16,
    pub max: u16,
    pub kind: Pk,
    pub c: u8,
    pub r: f32,
}

#[derive(Clone, Debug)]
pub struct FloatText {
    pub x: f32,
    pub y: f32,
    pub t: u16,
    pub s: String,
    pub c: u8,
}

/// Particles, shake, the white flash and floating texts.
pub struct Fx {
    pub p: Vec<Part>,
    pub shake: f32,
    pub flash: u8,
    pub texts: Vec<FloatText>,
    pub rng: Rng,
    /// Gravity on debris (side views have none, top-down views a little).
    pub gravity: f32,
}

const MAX_PARTS: usize = 900;

impl Fx {
    pub fn new(seed: u32) -> Fx {
        Fx {
            p: Vec::with_capacity(MAX_PARTS),
            shake: 0.0,
            flash: 0,
            texts: Vec::new(),
            rng: Rng::new(seed),
            gravity: 0.0,
        }
    }

    fn push(&mut self, p: Part) {
        if self.p.len() < MAX_PARTS {
            self.p.push(p);
        }
    }

    /// An explosion: a flash, a shock ring, hot sparks, debris and smoke.
    /// `size` 1 is a small fighter, 3 a big target, 6 a boss.
    pub fn explode(&mut self, x: f32, y: f32, size: f32) {
        let n = (10.0 * size) as usize + 6;
        self.push(Part {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            life: 0,
            max: (6.0 + size * 2.0) as u16,
            kind: Pk::Flash,
            c: WHITE,
            r: 3.0 + size * 2.5,
        });
        self.push(Part {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            life: 0,
            max: (10.0 + size * 3.0) as u16,
            kind: Pk::Ring,
            c: YELLOW,
            r: 2.0 + size * 1.5,
        });
        for _ in 0..n {
            let a = self.rng.f() * std::f32::consts::TAU;
            let sp = self.rng.range(0.4, 1.6 + size * 0.5);
            let life = self.rng.range(14.0, 26.0 + size * 6.0) as u16;
            self.push(Part {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                life: 0,
                max: life,
                kind: Pk::Spark,
                c: WHITE,
                r: 0.0,
            });
        }
        for _ in 0..(n / 3) {
            let a = self.rng.f() * std::f32::consts::TAU;
            let sp = self.rng.range(0.6, 1.2 + size * 0.4);
            let c = [GREY, LGREY, DGREY, BROWN][self.rng.below(4) as usize];
            let life = self.rng.range(30.0, 60.0 + size * 8.0) as u16;
            self.push(Part {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp - 0.4,
                life: 0,
                max: life,
                kind: Pk::Debris,
                c,
                r: 0.0,
            });
        }
        for _ in 0..(2 + n / 5) {
            let life = self.rng.range(30.0, 60.0) as u16;
            let (dx, dy) = (self.rng.s() * size * 2.0, self.rng.s() * size * 2.0);
            let (vx, vy) = (self.rng.s() * 0.2, -self.rng.range(0.05, 0.3));
            self.push(Part {
                x: x + dx,
                y: y + dy,
                vx,
                vy,
                life: 0,
                max: life,
                kind: Pk::Smoke,
                c: DGREY,
                r: 1.0 + size * 0.6,
            });
        }
        self.shake = self.shake.max(size * 1.2);
    }

    /// A few sparks where a shot hit something that did not die.
    pub fn sparks(&mut self, x: f32, y: f32, n: usize, dir: f32) {
        for _ in 0..n {
            let a = dir + self.rng.s() * 1.2;
            let sp = self.rng.range(0.8, 2.2);
            let max = self.rng.range(6.0, 14.0) as u16;
            self.push(Part {
                x,
                y,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                life: 0,
                max,
                kind: Pk::Spark,
                c: WHITE,
                r: 0.0,
            });
        }
    }

    pub fn smoke(&mut self, x: f32, y: f32, r: f32) {
        let life = self.rng.range(25.0, 45.0) as u16;
        let (vx, vy) = (self.rng.s() * 0.15, -self.rng.range(0.1, 0.35));
        self.push(Part {
            x,
            y,
            vx,
            vy,
            life: 0,
            max: life,
            kind: Pk::Smoke,
            c: DGREY,
            r,
        });
    }

    pub fn text(&mut self, x: f32, y: f32, s: String, c: u8) {
        if self.texts.len() < 24 {
            self.texts.push(FloatText { x, y, t: 0, s, c });
        }
    }

    pub fn update(&mut self) {
        let g = self.gravity;
        self.p.retain_mut(|p| {
            p.life += 1;
            p.x += p.vx;
            p.y += p.vy;
            match p.kind {
                Pk::Spark => {
                    p.vx *= 0.94;
                    p.vy *= 0.94;
                }
                Pk::Debris => {
                    p.vy += g;
                    p.vx *= 0.985;
                }
                Pk::Smoke => {
                    p.r += 0.05;
                }
                _ => {}
            }
            p.life < p.max
        });
        self.texts.retain_mut(|t| {
            t.t += 1;
            t.y -= 0.35;
            t.t < 60
        });
        self.shake *= 0.88;
        if self.shake < 0.2 {
            self.shake = 0.0;
        }
        self.flash = self.flash.saturating_sub(1);
    }

    /// The shake offset for this frame.
    pub fn offset(&self, t: u32) -> (i32, i32) {
        if self.shake <= 0.0 {
            return (0, 0);
        }
        let h = hash(t as i32, 7);
        let a = self.shake.min(8.0);
        let dx = ((h & 0xff) as f32 / 127.5 - 1.0) * a;
        let dy = (((h >> 8) & 0xff) as f32 / 127.5 - 1.0) * a;
        (dx.round() as i32, dy.round() as i32)
    }

    pub fn draw(&self, fb: &mut Fb, ox: i32, oy: i32) {
        for p in &self.p {
            let k = p.life as f32 / p.max.max(1) as f32;
            let (x, y) = (p.x as i32 + ox, p.y as i32 + oy);
            if p.kind == Pk::Smoke {
                let c = if k < 0.4 { GREY } else { DGREY };
                let r = p.r as i32;
                // Dithered puffs read as smoke without blending.
                for dy in -r..=r {
                    for dx in -r..=r {
                        if dx * dx + dy * dy <= r * r && (x + dx + y + dy) & 1 == 0 {
                            fb.set(x + dx, y + dy, c);
                        }
                    }
                }
            }
        }
        for p in &self.p {
            let k = p.life as f32 / p.max.max(1) as f32;
            let (x, y) = (p.x as i32 + ox, p.y as i32 + oy);
            match p.kind {
                Pk::Spark => {
                    let c = HEAT[((k * 6.0) as usize).min(6)];
                    fb.set(x, y, c);
                    if k < 0.35 {
                        fb.set(x - p.vx.signum() as i32, y - p.vy.signum() as i32, YELLOW);
                    }
                }
                Pk::Debris => {
                    fb.set(x, y, p.c);
                    if k < 0.5 {
                        fb.set(x + 1, y, p.c);
                    }
                }
                Pk::Ring => {
                    let r = (p.r + k * p.r * 3.0) as i32;
                    let c = HEAT[((k * 5.0) as usize + 1).min(6)];
                    fb.circle(x, y, r, c);
                    if k < 0.5 {
                        fb.circle(x, y, r - 1, HEAT[((k * 5.0) as usize).min(6)]);
                    }
                }
                Pk::Flash => {
                    let r = (p.r * (1.0 - k * 0.5)) as i32;
                    let c = HEAT[((k * 4.0) as usize).min(4)];
                    fb.disc(x, y, r, c);
                    if k < 0.5 {
                        fb.disc(x, y, r / 2, WHITE);
                    }
                }
                Pk::Smoke => {}
            }
        }
        for t in &self.texts {
            let c = if t.t % 8 < 4 { t.c } else { WHITE };
            let w = crate::fb::text_w(&t.s);
            fb.text(t.x as i32 - w / 2 + ox, t.y as i32 + oy, &t.s, c);
        }
    }
}

/// The weapons NIRUX finds in the Undercroft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weapon {
    /// A bolt of light, straight ahead.
    Bolt,
    /// A hammer of light thrown in an arc; it passes through enemies.
    Hammer,
    /// A disc that bounces off walls twice.
    Disc,
    /// A short chain whip: an arc in front, strong and quick.
    Whip,
}

impl Weapon {
    pub fn name(self) -> &'static str {
        match self {
            Weapon::Bolt => "BOLT",
            Weapon::Hammer => "HAMMER",
            Weapon::Disc => "DISC",
            Weapon::Whip => "WHIP",
        }
    }
    pub fn color(self) -> u8 {
        match self {
            Weapon::Bolt => CYAN,
            Weapon::Hammer => YELLOW,
            Weapon::Disc => LGREEN,
            Weapon::Whip => LRED,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Stats {
    pub score: u32,
    pub lives: i32,
    /// The scarf: three pieces. A hit with none left costs a life.
    pub scarf: u8,
    pub weapon: Weapon,
    /// Weapon level 1 to 3 (more shots, more damage).
    pub power: u8,
    /// The visor lamp's charge, 0 to 100.
    pub lamp: f32,
    pub kills: u32,
    pub gems: u32,
    pub next_life: u32,
    pub deaths_this_level: u32,
}

pub const LIFE_EVERY: u32 = 50000;
pub const SCARF_MAX: u8 = 3;

impl Stats {
    pub fn new(lives: i32) -> Stats {
        Stats {
            score: 0,
            lives,
            scarf: SCARF_MAX,
            weapon: Weapon::Bolt,
            power: 1,
            lamp: 100.0,
            kills: 0,
            gems: 0,
            next_life: LIFE_EVERY,
            deaths_this_level: 0,
        }
    }

    /// Add points; returns true when that earned an extra life.
    pub fn add(&mut self, pts: u32) -> bool {
        self.score = self.score.saturating_add(pts);
        if self.score >= self.next_life {
            self.next_life += LIFE_EVERY;
            self.lives = (self.lives + 1).min(9);
            return true;
        }
        false
    }
}

/// What a level asks of the game this frame.
#[derive(Default)]
pub struct Out {
    pub sfx: Vec<Sfx>,
    pub music: Option<crate::songs::SongId>,
}

impl Out {
    pub fn play(&mut self, s: Sfx) {
        if !self.sfx.contains(&s) {
            self.sfx.push(s);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Cleared,
    Over,
}

/// Parallax stars: three layers, `speed` pixels per frame for the nearest.
pub struct Stars {
    s: Vec<(f32, f32, u8)>,
}

impl Stars {
    pub fn new(n: usize, seed: u32) -> Stars {
        let mut r = Rng::new(seed);
        let s = (0..n)
            .map(|i| {
                (
                    r.f() * W as f32,
                    HUD_H as f32 + r.f() * (H - HUD_H) as f32,
                    (i % 3) as u8,
                )
            })
            .collect();
        Stars { s }
    }

    pub fn update(&mut self, dx: f32, dy: f32) {
        for (x, y, l) in self.s.iter_mut() {
            let k = [0.2, 0.5, 1.0][*l as usize];
            *x -= dx * k;
            *y -= dy * k;
            if *x < 0.0 {
                *x += W as f32;
            }
            if *x >= W as f32 {
                *x -= W as f32;
            }
            if *y < HUD_H as f32 {
                *y += (H - HUD_H) as f32;
            }
            if *y >= H as f32 {
                *y -= (H - HUD_H) as f32;
            }
        }
    }

    pub fn draw(&self, fb: &mut Fb, t: u32) {
        for (i, (x, y, l)) in self.s.iter().enumerate() {
            let c = match l {
                0 => DGREY,
                1 => {
                    if (i as u32 + t / 9) % 11 == 0 {
                        LGREY
                    } else {
                        GREY
                    }
                }
                _ => {
                    if (i as u32 + t / 5) % 7 == 0 {
                        CYAN
                    } else {
                        WHITE
                    }
                }
            };
            fb.set(*x as i32, *y as i32, c);
            if *l == 2 && i % 4 == 0 {
                fb.set(*x as i32 + 1, *y as i32, LGREY);
            }
        }
    }
}

fn digits(n: u32, w: usize) -> String {
    format!("{n:0w$}")
}

/// The status line: score, high score, lives, the scarf, the weapon and the
/// lamp's charge.
pub fn draw_hud(fb: &mut Fb, st: &Stats, hi: u32, line: u8, t: u32) {
    fb.rect(0, 0, W, HUD_H, BLACK);
    fb.hline(0, W - 1, HUD_H - 1, line);
    fb.text(2, 2, &format!("{:07}", st.score), WHITE);
    fb.text(62, 2, "HI", LBLUE);
    fb.text(80, 2, &format!("{:07}", hi.max(st.score)), LGREY);
    // Lives: little helmets.
    for i in 0..st.lives.clamp(0, 5) {
        let x = 142 + i * 8;
        fb.disc(x + 2, 5, 2, LGREY);
        fb.hline(x + 1, x + 3, 5, LGREEN);
    }
    // The scarf: three red pieces, a burnt one dark.
    let sx = 186;
    for i in 0..SCARF_MAX as i32 {
        let on = (i as u8) < st.scarf;
        let c = if on {
            if st.scarf == 1 && t % 20 < 10 {
                WHITE
            } else {
                LRED
            }
        } else {
            DGREY
        };
        fb.rect(sx + i * 9, 3, 7, 4, c);
        if on {
            fb.hline(sx + i * 9, sx + i * 9 + 6, 3, RED);
        }
    }
    // The weapon and its level.
    let wx = 218;
    fb.text(wx, 2, &st.weapon.name()[..1], st.weapon.color());
    for i in 0..3 {
        let c = if (i as u8) < st.power {
            st.weapon.color()
        } else {
            DGREY
        };
        fb.rect(wx + 9 + i * 3, 6 - i, 2, 2 + i, c);
    }
    // The lamp: a green bar.
    let lx = 240;
    fb.text(lx, 2, "*", LGREEN);
    fb.frame(lx + 9, 2, 32, 7, DGREY);
    let f = (st.lamp.clamp(0.0, 100.0) * 0.3) as i32;
    fb.rect(
        lx + 10,
        3,
        f,
        5,
        if st.lamp < 20.0 { GREEN } else { LGREEN },
    );
    if st.gems > 0 {
        fb.text(W - 36, 2, &format!("{:03}", st.gems.min(999)), YELLOW);
        fb.set(W - 40, 4, YELLOW);
        fb.set(W - 41, 5, YELLOW);
        fb.set(W - 39, 5, YELLOW);
        fb.set(W - 40, 6, YELLOW);
    }
}

/// A boss health bar along the bottom of the screen.
pub fn draw_boss_bar(fb: &mut Fb, name: &str, hp: i32, max: i32, t: u32) {
    let y = H - 9;
    fb.rect(0, y - 1, W, 10, BLACK);
    fb.text(4, y, name, if t % 16 < 8 { LRED } else { RED });
    let x0 = 8 + crate::fb::text_w(name) + 4;
    let w = W - x0 - 6;
    fb.frame(x0, y + 1, w, 6, GREY);
    let f = (w - 2) * hp.max(0) / max.max(1);
    fb.rect(x0 + 1, y + 2, f, 4, if hp * 4 < max { RED } else { LRED });
    fb.hline(x0 + 1, x0 + f, y + 2, YELLOW);
}

/// Distance squared.
pub fn d2(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    (ax - bx) * (ax - bx) + (ay - by) * (ay - by)
}

/// A round enemy bullet, flickering between two colours.
pub fn draw_bullet(fb: &mut Fb, x: i32, y: i32, t: u32, big: bool) {
    let c = if t % 6 < 3 { LRED } else { PURPLE };
    if big {
        fb.disc(x, y, 3, c);
        fb.disc(x, y, 1, YELLOW);
        fb.set(x, y, WHITE);
    } else {
        fb.hline(x - 1, x + 1, y - 1, c);
        fb.hline(x - 2, x + 2, y, c);
        fb.hline(x - 1, x + 1, y + 1, c);
        fb.set(x, y, WHITE);
    }
}
