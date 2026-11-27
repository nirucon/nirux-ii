//! A stage of the Undercroft: NIRUX running, jumping, climbing and
//! fighting through a map built in `stages.rs`, the dead rising around,
//! the cron bells, the visor lamp, shrines that mend the scarf, and the
//! boss at the end.
use super::boss::{Boss, BossKind};
use super::common::*;
use super::foes::{Drop, Foe, Kind};
use super::world::{self, Map, Theme, T};
use super::Level;
use crate::config::Difficulty;
use crate::fb::{text_w, Fb, H, W};
use crate::hero::{self, Pose};
use crate::input::Input;
use crate::palette::*;
use crate::sfx::Sfx;
use crate::songs::SongId;

/// Who speaks a line: ECHO in green, NULL in red, or the world in white.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    Echo,
    Null,
    World,
    Reaper,
}

#[derive(Clone, Debug)]
pub enum Ev {
    Say(Who, &'static str),
    /// The cron bells: three tolls, then a wave of wraiths.
    Bells,
    /// The Reaper looks on from a height, and reaps.
    Reaper,
    /// The Reaper walks with NIRUX for a while, reaping the risen dead.
    ReaperAlly,
    /// More or less rain from here.
    Rain(u32),
    Thunder,
    /// The boss arena starts here: the camera stops and the boss comes.
    Boss(BossKind),
}

pub struct Trigger {
    pub x: f32,
    pub ev: Ev,
    pub done: bool,
}

/// An enemy or urn waiting in the map until NIRUX comes near.
pub struct Placed {
    pub x: f32,
    pub y: f32,
    pub kind: Kind,
    pub drop: Drop,
    pub n: u8,
    pub done: bool,
}

/// A shrine: touching it makes it the place to come back to, and mends
/// one piece of the scarf.
pub struct Shrine {
    pub x: f32,
    pub y: f32,
    pub lit: bool,
}

pub struct Spec {
    pub theme: Theme,
    pub rain: u32,
    pub fog: bool,
    pub border: u8,
    pub hud: u8,
}

pub struct Built {
    pub map: Map,
    pub triggers: Vec<Trigger>,
    pub placed: Vec<Placed>,
    pub shrines: Vec<Shrine>,
    pub start: (f32, f32),
    /// Where the arena begins (pixels) and its floor (row).
    pub arena: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Shot {
    Bolt,
    Hammer,
    Disc,
}

struct PShot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    kind: Shot,
    dmg: i32,
    life: u32,
    bounces: u8,
    hit: Vec<usize>,
    big: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EKind {
    Fire,
    Clod,
    Spit,
    Note,
    Gear,
    Shard,
    Static,
    Wave,
}

pub struct EShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub g: f32,
    pub kind: EKind,
    pub life: u32,
}

struct Pick {
    x: f32,
    y: f32,
    vy: f32,
    drop: Drop,
    t: u32,
}

struct Reaper {
    x: f32,
    y: f32,
    t: u32,
    /// Walking with NIRUX (stage 6 on), not only watching.
    ally: bool,
}

/// How long the Reaper walks with NIRUX.
const ALLY_FRAMES: u32 = 1500;

pub struct Stage {
    pub spec: Spec,
    pub map: Map,
    triggers: Vec<Trigger>,
    placed: Vec<Placed>,
    shrines: Vec<Shrine>,
    pub foes: Vec<Foe>,
    shots: Vec<PShot>,
    pub eshots: Vec<EShot>,
    picks: Vec<Pick>,
    pub fx: Fx,
    pub rng: Rng,
    pub d: Difficulty,
    pub t: u32,
    // NIRUX.
    pub px: f32,
    pub py: f32,
    pub vx: f32,
    pub vy: f32,
    pub f: f32,
    grounded: bool,
    coyote: u32,
    jump_buf: u32,
    crouch: bool,
    climbing: bool,
    climb_ph: f32,
    drop_t: u32,
    run_ph: f32,
    fire_cd: u32,
    whip_t: u32,
    muzzle: u32,
    pub inv: u32,
    hurt: u32,
    pub dead: u32,
    pub lamp_on: bool,
    safe: (f32, f32),
    respawn: (f32, f32),
    aim_up: bool,
    // The camera.
    pub cx: f32,
    pub cy: f32,
    pub arena: f32,
    pub locked: bool,
    // The world.
    rain: u32,
    bolt: u32,
    bolt_x: i32,
    bells: Option<u32>,
    reaper: Option<Reaper>,
    say: Option<(Who, &'static str, u32)>,
    pub boss: Option<Boss>,
    boss_kind: Option<BossKind>,
    clear_t: Option<u32>,
    over_t: Option<u32>,
    length: f32,
    pub stage_no: usize,
}

const RUN: f32 = 1.4;
const GRAV: f32 = 0.22;
const JUMP: f32 = 4.15;
const MAX_FALL: f32 = 4.6;
const PW: f32 = 4.0;

impl Stage {
    pub fn new(spec: Spec, b: Built, d: Difficulty, seed: u32, stage_no: usize) -> Stage {
        let start = b.start;
        let length = b.map.w as f32 * T as f32;
        let rain = spec.rain;
        let mut fx = Fx::new(seed);
        fx.gravity = 0.12;
        Stage {
            spec,
            map: b.map,
            triggers: b.triggers,
            placed: b.placed,
            shrines: b.shrines,
            foes: Vec::new(),
            shots: Vec::new(),
            eshots: Vec::new(),
            picks: Vec::new(),
            fx,
            rng: Rng::new(seed ^ 0x5eed),
            d,
            t: 0,
            px: start.0,
            py: start.1,
            vx: 0.0,
            vy: 0.0,
            f: 1.0,
            grounded: false,
            coyote: 0,
            jump_buf: 0,
            crouch: false,
            climbing: false,
            climb_ph: 0.0,
            drop_t: 0,
            run_ph: 0.0,
            fire_cd: 0,
            whip_t: 0,
            muzzle: 0,
            inv: 60,
            hurt: 0,
            dead: 0,
            lamp_on: false,
            safe: start,
            respawn: start,
            aim_up: false,
            cx: 0.0,
            cy: 0.0,
            arena: b.arena,
            locked: false,
            rain,
            bolt: 0,
            bolt_x: 0,
            bells: None,
            reaper: None,
            say: None,
            boss: None,
            boss_kind: None,
            clear_t: None,
            over_t: None,
            length,
            stage_no,
        }
    }

    fn agg(&self) -> f32 {
        self.d.aggression()
    }

    fn ph(&self) -> f32 {
        if self.crouch {
            14.0
        } else {
            22.0
        }
    }

    // Collision against the map.

    fn solid_px(&self, x: f32, y: f32) -> bool {
        Map::solid(self.map.px(x, y))
    }

    fn hits_wall(&self, x: f32, y: f32, h: f32) -> bool {
        let mut yy = y - h + 1.0;
        while yy < y {
            if self.solid_px(x - PW, yy) || self.solid_px(x + PW, yy) {
                return true;
            }
            yy += 6.0;
        }
        self.solid_px(x - PW, y - 1.0) || self.solid_px(x + PW, y - 1.0)
    }

    /// Is there a floor right under the feet at x?
    fn floor_under(&self, x: f32, y: f32, oneway: bool) -> Option<f32> {
        for &dx in &[-PW + 1.0, 0.0, PW - 1.0] {
            let v = self.map.px(x + dx, y);
            let ok = if oneway { Map::floor(v) } else { Map::solid(v) };
            if ok {
                return Some(((y / T as f32).floor()) * T as f32);
            }
        }
        None
    }

    fn move_player(&mut self) {
        let h = self.ph();
        // Across.
        let nx = self.px + self.vx;
        if self.hits_wall(nx, self.py, h) {
            self.vx = 0.0;
        } else {
            self.px = nx;
        }
        if self.locked {
            let (a, b) = (self.arena + 6.0, self.arena + W as f32 - 6.0);
            self.px = self.px.clamp(a, b);
        }
        self.px = self.px.clamp(6.0, self.length - 6.0);
        // Down or up.
        let ny = self.py + self.vy;
        self.grounded = false;
        if self.vy >= 0.0 {
            // One-way floors count only when the feet come from above.
            let oneway = self.drop_t == 0;
            let was = self.py;
            let row_was = (was / T as f32).floor();
            let row_now = (ny / T as f32).floor();
            let mut landed = None;
            let mut r = row_was;
            while r <= row_now {
                let y = r * T as f32;
                if y >= was - 0.01 || Map::solid(self.map.px(self.px, y)) {
                    if let Some(top) = self.floor_under(self.px, y + 0.5, oneway) {
                        if top >= was - 0.01 || self.floor_under(self.px, y + 0.5, false).is_some()
                        {
                            landed = Some(top);
                            break;
                        }
                    }
                }
                r += 1.0;
            }
            if let Some(top) = landed {
                self.py = top;
                if self.vy > 2.5 {
                    self.fx.sparks(self.px, self.py, 3, -1.57);
                }
                self.vy = 0.0;
                self.grounded = true;
                let tx = (self.px / T as f32).floor() as i32;
                let ty = (top / T as f32) as i32;
                for dx in [-1, 0, 1] {
                    let x = ((self.px + dx as f32 * (PW - 1.0)) / T as f32).floor() as i32;
                    self.map.touch_crumble(x, ty);
                }
                // Remember solid ground to come back to after a fall.
                let v = self.map.at(tx, ty);
                if Map::solid(v)
                    && Map::solid(self.map.at(((self.px - 8.0) / T as f32) as i32, ty))
                    && Map::solid(self.map.at(((self.px + 8.0) / T as f32) as i32, ty))
                {
                    self.safe = (self.px, self.py);
                }
            } else {
                self.py = ny;
            }
        } else {
            let head = ny - h;
            if self.solid_px(self.px - PW + 1.0, head) || self.solid_px(self.px + PW - 1.0, head) {
                self.vy = 0.0;
                self.py = ((head / T as f32).floor() + 1.0) * T as f32 + h;
            } else {
                self.py = ny;
            }
        }
    }

    fn on_ladder(&self) -> bool {
        Map::ladder(self.map.px(self.px, self.py - 4.0))
            || Map::ladder(self.map.px(self.px, self.py - 14.0))
    }

    // Damage.

    fn hit_player(&mut self, st: &mut Stats, out: &mut Out, from: f32) {
        if self.inv > 0 || self.dead > 0 || self.clear_t.is_some() {
            return;
        }
        if st.scarf == 0 {
            self.kill(st, out);
            return;
        }
        st.scarf -= 1;
        out.play(Sfx::ScarfTear);
        self.fx.sparks(
            self.px,
            self.py - 16.0,
            8,
            if from < self.px {
                0.0
            } else {
                std::f32::consts::PI
            },
        );
        for k in 0..5 {
            let a = self.rng.range(-2.5, -0.6);
            self.fx.p.push(Part {
                x: self.px - self.f * 4.0,
                y: self.py - 18.0,
                vx: a.cos() * 1.2 - self.f * 0.6,
                vy: a.sin() * 1.2,
                life: 40 + k * 5,
                max: 60,
                kind: Pk::Debris,
                c: if k % 2 == 0 { RED } else { LRED },
                r: 1.0,
            });
        }
        self.hurt = 24;
        self.inv = 110;
        self.climbing = false;
        self.vx = if from < self.px { 1.6 } else { -1.6 };
        self.vy = -2.6;
        if st.scarf == 0 {
            self.fx
                .text(self.px, self.py - 30.0, "NO SCARF LEFT".into(), LRED);
        }
    }

    fn kill(&mut self, st: &mut Stats, out: &mut Out) {
        if self.dead > 0 {
            return;
        }
        self.dead = 1;
        st.lives -= 1;
        st.deaths_this_level += 1;
        out.play(Sfx::PlayerDie);
        self.fx.explode(self.px, self.py - 12.0, 1.5);
        self.climbing = false;
    }

    /// Fell into a pit or the water: a piece of the scarf, then back to
    /// the last safe ground.
    fn fell(&mut self, st: &mut Stats, out: &mut Out, water: bool) {
        if water {
            out.play(Sfx::Splash);
            for _ in 0..12 {
                let a = self.rng.range(-2.6, -0.5);
                self.fx.p.push(Part {
                    x: self.px,
                    y: self.py - 4.0,
                    vx: a.cos() * 1.5,
                    vy: a.sin() * 2.0,
                    life: 30,
                    max: 30,
                    kind: Pk::Debris,
                    c: if Map::liquid(self.map.px(self.px, self.py + 2.0))
                        && self.map.px(self.px, self.py + 2.0) == world::LAVA
                    {
                        ORANGE
                    } else if self.spec.theme == Theme::Marsh {
                        LGREEN
                    } else {
                        LBLUE
                    },
                    r: 1.0,
                });
            }
        }
        if st.scarf == 0 {
            self.kill(st, out);
            return;
        }
        st.scarf -= 1;
        out.play(Sfx::ScarfTear);
        self.px = self.safe.0;
        self.py = self.safe.1;
        self.vx = 0.0;
        self.vy = 0.0;
        self.inv = 100;
        self.climbing = false;
    }

    // Spawning.

    fn spawn_near(&mut self, out: &mut Out) {
        let view_r = self.cx + W as f32 + 24.0;
        let view_l = self.cx - 40.0;
        let agg = self.agg();
        for i in 0..self.placed.len() {
            let p = &self.placed[i];
            if p.done || p.x > view_r || p.x < view_l {
                continue;
            }
            // Zombies wait until NIRUX is close, then rise.
            if p.kind == Kind::Zombie && (p.x - self.px).abs() > 110.0 {
                continue;
            }
            let (x, y, kind, drop, n) = (p.x, p.y, p.kind, p.drop, p.n);
            self.placed[i].done = true;
            for k in 0..n.max(1) {
                let mut e = Foe::new(kind, x + k as f32 * 14.0, y);
                e.drop = drop;
                e.f = if self.px < x { -1.0 } else { 1.0 };
                e.cd = 40 + (k as i32 * 17) + self.rng.below(40) as i32;
                if kind == Kind::Zombie {
                    out.play(Sfx::Rise);
                    e.rise = (50.0 / agg.max(0.6)) as u32;
                }
                if kind == Kind::Ghost {
                    e.t = self.rng.below(200);
                }
                if kind == Kind::Pendulum {
                    e.ay = y - 64.0;
                    e.t = (x as u32) % 180;
                }
                self.foes.push(e);
            }
        }
    }

    fn drop_at(&mut self, x: f32, y: f32, d: Drop) {
        match d {
            Drop::Nothing => {}
            Drop::Gems(n) => {
                for k in 0..n {
                    self.picks.push(Pick {
                        x: x + (k as f32 - n as f32 / 2.0) * 5.0,
                        y,
                        vy: -2.0 - k as f32 * 0.2,
                        drop: Drop::Gems(1),
                        t: 0,
                    });
                }
            }
            _ => self.picks.push(Pick {
                x,
                y,
                vy: -2.2,
                drop: d,
                t: 0,
            }),
        }
    }

    // Weapons.

    fn fire(&mut self, st: &Stats, out: &mut Out) {
        let (x, y) = (
            self.px + self.f * 6.0,
            self.py - if self.crouch { 7.0 } else { 14.0 },
        );
        let up = self.aim_up;
        let n_out = self.shots.len();
        let p = st.power;
        match st.weapon {
            Weapon::Bolt => {
                let max = [2, 3, 4][(p as usize - 1).min(2)];
                if n_out >= max {
                    return;
                }
                let (vx, vy) = if up {
                    (self.f * 3.3, -3.3)
                } else {
                    (self.f * 4.8, 0.0)
                };
                self.shots.push(PShot {
                    x,
                    y,
                    vx,
                    vy,
                    kind: Shot::Bolt,
                    dmg: if p >= 3 { 2 } else { 1 },
                    life: 70,
                    bounces: 0,
                    hit: Vec::new(),
                    big: p >= 2,
                });
                if p >= 3 {
                    self.shots.push(PShot {
                        x,
                        y: y - 2.0,
                        vx: vx * 0.95,
                        vy: vy - 0.8,
                        kind: Shot::Bolt,
                        dmg: 1,
                        life: 60,
                        bounces: 0,
                        hit: Vec::new(),
                        big: false,
                    });
                }
                self.fire_cd = 9;
                out.play(Sfx::Bolt);
            }
            Weapon::Hammer => {
                let max = [1, 2, 2][(p as usize - 1).min(2)];
                if n_out >= max {
                    return;
                }
                let (vx, vy) = if up {
                    (self.f * 1.2, -5.2)
                } else {
                    (self.f * 2.6, -3.6)
                };
                self.shots.push(PShot {
                    x,
                    y,
                    vx,
                    vy,
                    kind: Shot::Hammer,
                    dmg: if p >= 2 { 3 } else { 2 },
                    life: 120,
                    bounces: 0,
                    hit: Vec::new(),
                    big: p >= 3,
                });
                self.fire_cd = 16;
                out.play(Sfx::Hammer);
            }
            Weapon::Disc => {
                let max = [2, 2, 3][(p as usize - 1).min(2)];
                if n_out >= max {
                    return;
                }
                let (vx, vy) = if up {
                    (self.f * 2.8, -2.8)
                } else {
                    (self.f * 3.8, 0.0)
                };
                self.shots.push(PShot {
                    x,
                    y,
                    vx,
                    vy,
                    kind: Shot::Disc,
                    dmg: if p >= 2 { 2 } else { 1 },
                    life: 150,
                    bounces: if p >= 3 { 3 } else { 2 },
                    hit: Vec::new(),
                    big: p >= 3,
                });
                self.fire_cd = 12;
                out.play(Sfx::Disc);
            }
            Weapon::Whip => {
                self.whip_t = 12;
                self.fire_cd = 16;
                out.play(Sfx::Whip);
            }
        }
        self.muzzle = 6;
    }

    /// The whip's reach this frame, as a box (x0, y0, x1, y1).
    fn whip_box(&self, st: &Stats) -> Option<(f32, f32, f32, f32)> {
        if self.whip_t == 0 || self.whip_t > 9 {
            return None;
        }
        let reach = [26.0, 32.0, 38.0][(st.power as usize - 1).min(2)];
        let y = self.py - if self.crouch { 7.0 } else { 14.0 };
        let (x0, x1) = if self.f > 0.0 {
            (self.px, self.px + reach)
        } else {
            (self.px - reach, self.px)
        };
        if self.aim_up {
            Some((
                x0.min(x1) + self.f * 4.0,
                y - reach * 0.8,
                x1.max(x0),
                y + 2.0,
            ))
        } else {
            Some((x0, y - 6.0, x1, y + 6.0))
        }
    }

    fn damage_foe(
        &mut self,
        i: usize,
        dmg: i32,
        from_x: f32,
        st: &mut Stats,
        out: &mut Out,
        pierce: bool,
    ) -> bool {
        let e = &mut self.foes[i];
        if !e.tangible() || e.hp <= 0 {
            return false;
        }
        if e.kind == Kind::Pendulum {
            out.play(Sfx::Clang);
            return true;
        }
        // The knight's shield stops what comes at it from the front, except
        // hammers falling from above and the whip.
        if e.kind == Kind::Knight && !pierce && (from_x - e.x) * e.f > 0.0 && e.state != 2 {
            e.flash = 2;
            out.play(Sfx::Clang);
            self.fx.sparks(
                e.x + e.f * 6.0,
                e.y - 13.0,
                4,
                if e.f > 0.0 { 0.0 } else { std::f32::consts::PI },
            );
            return true;
        }
        let dmg = if e.kind == Kind::Ghost && e.lit {
            dmg * 2
        } else {
            dmg
        };
        e.hp -= dmg;
        e.flash = 6;
        if e.rise > 0 {
            e.rise = 0;
        }
        out.play(Sfx::Hit);
        if e.hp <= 0 {
            let (x, y, kind, drop) = (e.x, e.y, e.kind, e.drop);
            st.kills += 1;
            if st.add(kind.score()) {
                out.play(Sfx::ExtraLife);
            }
            match kind {
                Kind::Urn => {
                    out.play(Sfx::Crumble);
                    self.fx.sparks(x, y - 5.0, 6, -1.57);
                }
                Kind::Ghost => {
                    for k in 0..10 {
                        let a = k as f32 * 0.628;
                        self.fx.p.push(Part {
                            x,
                            y: y - 8.0,
                            vx: a.cos(),
                            vy: a.sin() - 0.5,
                            life: 30,
                            max: 30,
                            kind: Pk::Spark,
                            c: if k % 2 == 0 { WHITE } else { LGREY },
                            r: 1.0,
                        });
                    }
                    out.play(Sfx::ExplodeSmall);
                }
                _ => {
                    self.fx.explode(
                        x,
                        y - kind.size().1 / 2.0,
                        if kind == Kind::Knight { 2.0 } else { 1.0 },
                    );
                    out.play(Sfx::ExplodeSmall);
                }
            }
            self.fx.text(x, y - 20.0, kind.score().to_string(), YELLOW);
            if kind == Kind::ForkBomb {
                for s in [-1.0, 1.0] {
                    let mut b = Foe::new(Kind::ForkBit, x + s * 4.0, y - 4.0);
                    b.vx = s * 1.4;
                    b.vy = -2.5;
                    self.foes.push(b);
                }
            }
            let d = match drop {
                Drop::Nothing if kind == Kind::Urn => Drop::Gems(1 + self.rng.below(3) as u8),
                Drop::Nothing if self.rng.chance(0.08) => Drop::Gems(1),
                d => d,
            };
            self.drop_at(x, y - 6.0, d);
        }
        true
    }

    // The frame.

    fn update_player(&mut self, inp: &Input, st: &mut Stats, out: &mut Out) {
        if self.dead > 0 {
            self.dead += 1;
            if self.dead > 110 {
                if st.lives <= 0 {
                    self.over_t.get_or_insert(self.t);
                } else {
                    self.dead = 0;
                    self.px = self.respawn.0;
                    self.py = self.respawn.1;
                    self.safe = self.respawn;
                    self.vx = 0.0;
                    self.vy = 0.0;
                    self.inv = 150;
                    st.scarf = SCARF_MAX;
                    st.lamp = 100.0;
                    if self.boss.is_some() {
                        // Back to the arena's door.
                        self.px = self.arena + 24.0;
                        self.py = (self.map.top_at(((self.arena + 24.0) / T as f32) as i32, 0) * T)
                            as f32;
                    }
                }
            }
            return;
        }
        if self.clear_t.is_some() {
            self.vx *= 0.8;
            self.vy = (self.vy + GRAV).min(MAX_FALL);
            self.move_player();
            return;
        }
        self.inv = self.inv.saturating_sub(1);
        self.drop_t = self.drop_t.saturating_sub(1);
        self.fire_cd = self.fire_cd.saturating_sub(1);
        self.whip_t = self.whip_t.saturating_sub(1);
        self.muzzle = self.muzzle.saturating_sub(1);
        if inp.p_jump {
            self.jump_buf = 7;
        } else {
            self.jump_buf = self.jump_buf.saturating_sub(1);
        }
        // The lamp.
        let was = self.lamp_on;
        self.lamp_on = inp.special && st.lamp > 0.5 && self.hurt == 0;
        if self.lamp_on {
            st.lamp = (st.lamp - 0.55).max(0.0);
            if !was {
                out.play(Sfx::Lamp);
            }
        } else {
            st.lamp = (st.lamp + 0.22).min(100.0);
        }
        if self.hurt > 0 {
            self.hurt -= 1;
            self.vy = (self.vy + GRAV).min(MAX_FALL);
            self.vx *= 0.96;
            self.move_player();
            return;
        }
        self.aim_up = inp.up && !self.climbing && !(self.on_ladder() && !self.grounded);
        // Climbing.
        if !self.climbing
            && (inp.up || (inp.down && !self.grounded) || (inp.down && self.map_ladder_below()))
            && self.ladder_here(inp)
        {
            self.climbing = true;
            self.px = ((self.px / T as f32).floor() + 0.5) * T as f32;
            self.vx = 0.0;
            self.vy = 0.0;
            if inp.down && self.grounded {
                self.py += 2.0;
            }
        }
        if self.climbing {
            self.aim_up = false;
            self.crouch = false;
            let dy = inp.dy() * 1.1;
            self.climb_ph += dy.abs() * 0.06;
            let ny = self.py + dy;
            let tx = (self.px / T as f32).floor() as i32;
            // Off the top: stand on the ladder's top.
            let row_feet = ((ny - 0.5) / T as f32).floor() as i32;
            if !Map::ladder(self.map.at(tx, row_feet))
                && !Map::ladder(self.map.at(tx, ((ny - 12.0) / T as f32).floor() as i32))
            {
                if dy < 0.0 {
                    self.climbing = false;
                    self.py = ((ny / T as f32).floor() + 1.0) * T as f32;
                    self.py =
                        self.map.top_at(tx, (self.py / T as f32) as i32 - 2) as f32 * T as f32;
                    self.grounded = true;
                } else {
                    self.climbing = false;
                }
            } else if dy > 0.0 && Map::solid(self.map.px(self.px, ny + 0.5)) {
                self.climbing = false;
                self.py = (ny / T as f32).floor() * T as f32 + T as f32;
                self.py = ((self.py / T as f32).floor()) * T as f32;
            } else {
                self.py = ny;
            }
            if inp.p_jump || inp.dx() != 0.0 && self.jump_buf > 0 {
                self.climbing = false;
                self.vy = -2.2;
                self.jump_buf = 0;
                self.vx = inp.dx() * RUN;
            }
            if inp.fire && self.fire_cd == 0 {
                if inp.dx() != 0.0 {
                    self.f = inp.dx();
                }
                self.fire(st, out);
            }
            return;
        }
        // Running and crouching.
        self.crouch = inp.down && self.grounded;
        let want = if self.crouch { 0.0 } else { inp.dx() * RUN };
        let on_ice = self.grounded && self.map.px(self.px, self.py + 1.0) == world::ICE;
        let acc = if on_ice {
            0.035
        } else if self.grounded {
            0.3
        } else {
            0.07
        };
        if self.vx < want {
            self.vx = (self.vx + acc).min(want);
        } else if self.vx > want {
            self.vx = (self.vx - acc).max(want);
        }
        if inp.dx() != 0.0
            && (self.grounded || inp.dx() == self.f || self.vx.abs() < 0.3)
            && !(inp.fire && self.grounded && self.fire_cd > 4)
        {
            self.f = inp.dx();
        }
        // Jumping: the arc is firm, a little control in the air, and a
        // short tap makes a short hop.
        if self.grounded {
            self.coyote = 6;
        } else {
            self.coyote = self.coyote.saturating_sub(1);
        }
        if self.jump_buf > 0 && self.coyote > 0 {
            if inp.down && self.grounded && self.oneway_below() {
                self.drop_t = 12;
                self.py += 1.0;
            } else {
                self.vy = -JUMP;
                out.play(Sfx::Jump);
            }
            self.jump_buf = 0;
            self.coyote = 0;
            self.grounded = false;
            self.crouch = false;
        }
        if !inp.jump && self.vy < -1.6 {
            self.vy = -1.6;
        }
        self.vy = (self.vy + GRAV).min(MAX_FALL);
        let was_air = !self.grounded;
        self.move_player();
        if self.grounded && was_air && self.vy == 0.0 {
            out.play(Sfx::Land);
        }
        if self.grounded && self.vx.abs() > 0.2 {
            self.run_ph = (self.run_ph + self.vx.abs() * 0.045) % 1.0;
        }
        if inp.fire && self.fire_cd == 0 {
            self.fire(st, out);
        }
        // Hazards: spikes, water, the pit.
        let feet = self.map.px(self.px, self.py - 1.0);
        let below = self.map.px(self.px, self.py + 1.0);
        if feet == world::SPIKES || (self.grounded && below == world::SPIKES) {
            self.hit_player(st, out, self.px - self.f);
        }
        if Map::liquid(feet) || Map::liquid(self.map.px(self.px, self.py - 6.0)) {
            self.fell(st, out, true);
        } else if self.py > (self.map.h * T) as f32 + 24.0 {
            self.fell(st, out, false);
        }
    }

    fn ladder_here(&self, inp: &Input) -> bool {
        let tx = self.px;
        if inp.up {
            Map::ladder(self.map.px(tx, self.py - 4.0))
                || Map::ladder(self.map.px(tx, self.py - 16.0))
        } else {
            Map::ladder(self.map.px(tx, self.py + 2.0))
                || Map::ladder(self.map.px(tx, self.py - 4.0))
        }
    }

    fn map_ladder_below(&self) -> bool {
        Map::ladder(self.map.px(self.px, self.py + 2.0))
    }

    fn oneway_below(&self) -> bool {
        let v = self.map.px(self.px, self.py + 1.0);
        matches!(
            v,
            world::PLAT | world::HIDDEN | world::LADDER_TOP | world::CRUMBLE
        )
    }

    fn update_lamp(&mut self) {
        if !self.lamp_on {
            for e in self.foes.iter_mut() {
                e.lit = false;
            }
            return;
        }
        let (ox, oy) = (self.px + self.f * 3.0, self.py - 20.0);
        let inside = |x: f32, y: f32| {
            let dx = (x - ox) * self.f;
            let dy = y - oy;
            dx > 0.0 && dx < 100.0 && dy.abs() < dx * 0.45 + 6.0
        };
        for e in self.foes.iter_mut() {
            e.lit = inside(e.x, e.y - e.kind.size().1 / 2.0);
        }
        // Hidden platforms in the beam show, and stay shown.
        let tx0 = ((ox.min(ox + self.f * 100.0)) / T as f32) as i32;
        let tx1 = ((ox.max(ox + self.f * 100.0)) / T as f32) as i32;
        for ty in ((oy - 50.0) / T as f32) as i32..=((oy + 50.0) / T as f32) as i32 {
            for tx in tx0..=tx1 {
                if self.map.at(tx, ty) == world::HIDDEN
                    && inside(tx as f32 * T as f32 + 4.0, ty as f32 * T as f32 + 2.0)
                {
                    let i = (ty * self.map.w + tx) as usize;
                    self.map.seen[i] = true;
                }
            }
        }
    }

    fn update_foes(&mut self, st: &mut Stats, out: &mut Out) {
        let agg = self.agg();
        let (px, py) = (self.px, self.py);
        let mut new_shots = Vec::new();
        let mut hurt_from = None;
        let n = self.foes.len();
        for i in 0..n {
            let mut e = self.foes[i].clone();
            e.t += 1;
            e.flash = e.flash.saturating_sub(1);
            if e.hp <= 0 {
                self.foes[i] = e;
                continue;
            }
            let dx = px - e.x;
            match e.kind {
                Kind::Zombie => {
                    if e.rise > 0 {
                        e.rise -= 1;
                        if e.t % 4 == 0 {
                            self.fx
                                .sparks(e.x + self.rng.s() * 5.0, e.y - 1.0, 1, -1.57);
                        }
                    } else {
                        let speed = 0.32 * agg;
                        if e.t % 50 == 0 {
                            e.f = dx.signum();
                        }
                        e.vx = e.f * speed;
                        let ahead = e.x + e.f * 6.0;
                        let floor_ahead = Map::floor(self.map.px(ahead, e.y + 1.0));
                        let wall = Map::solid(self.map.px(ahead, e.y - 6.0));
                        if !floor_ahead || wall {
                            e.f = -e.f;
                        } else {
                            e.x += e.vx;
                        }
                    }
                }
                Kind::Orphan => {
                    e.vy = (e.vy + GRAV).min(MAX_FALL);
                    if e.on_ground {
                        e.vx *= 0.7;
                        e.cd -= 1;
                        if e.cd <= 0 {
                            e.f = dx.signum();
                            e.vx = e.f * (1.0 + 0.5 * agg);
                            e.vy = -2.2 - self.rng.f() * 1.2;
                            e.cd = (40.0 / agg) as i32 + self.rng.below(30) as i32;
                            e.on_ground = false;
                        }
                    }
                    self.walk_body(&mut e);
                }
                Kind::Daemon => {
                    e.cd -= 1;
                    match e.state {
                        0 => {
                            // Hover above and ahead of NIRUX.
                            let tx = px + (e.t as f32 * 0.02).sin() * 60.0;
                            let ty = py - 70.0 + (e.t as f32 * 0.05).sin() * 10.0;
                            e.x += (tx - e.x) * 0.02;
                            e.y += (ty - e.y) * 0.03;
                            e.f = dx.signum();
                            if e.cd <= 0 {
                                if self.rng.chance(0.5) {
                                    e.state = 1;
                                    e.cd = 60;
                                    e.vx = dx.clamp(-80.0, 80.0) / 30.0;
                                    e.vy = 2.6;
                                } else {
                                    let (sx, sy) = (e.x, e.y - 8.0);
                                    let a = (py - 14.0 - sy).atan2(px - sx);
                                    let sp = 1.3 * agg;
                                    new_shots.push(EShot {
                                        x: sx,
                                        y: sy,
                                        vx: a.cos() * sp,
                                        vy: a.sin() * sp,
                                        g: 0.0,
                                        kind: EKind::Fire,
                                        life: 240,
                                    });
                                    out.play(Sfx::EnemyShot);
                                    e.cd = (150.0 / agg) as i32;
                                }
                            }
                        }
                        _ => {
                            e.x += e.vx;
                            e.y += e.vy;
                            e.vy -= 0.09;
                            if e.cd <= 0 || e.vy < -2.0 {
                                e.state = 0;
                                e.cd = (130.0 / agg) as i32;
                            }
                        }
                    }
                }
                Kind::Ghost => {
                    // Seen, then gone, then seen; drifting at NIRUX through
                    // the walls. The lamp holds it.
                    let cycle = e.t % 220;
                    e.shown = cycle < 110;
                    let slow = if e.lit { 0.2 } else { 1.0 };
                    let a = (py - 12.0 - e.y).atan2(dx);
                    e.x += a.cos() * 0.45 * agg * slow;
                    e.y += a.sin() * 0.35 * agg * slow + (e.t as f32 * 0.07).sin() * 0.3;
                    e.f = dx.signum();
                }
                Kind::Knight => {
                    e.vy = (e.vy + GRAV).min(MAX_FALL);
                    e.cd -= 1;
                    match e.state {
                        0 | 1 => {
                            e.f = dx.signum();
                            e.vx = if dx.abs() > 30.0 {
                                e.f * 0.35 * agg
                            } else {
                                0.0
                            };
                            if dx.abs() < 60.0 && e.cd <= 0 {
                                e.state = 2;
                                e.cd = 40;
                            }
                        }
                        _ => {
                            e.vx = e.f * if e.cd > 20 { 1.8 } else { 0.0 };
                            if e.cd <= 0 {
                                e.state = 0;
                                e.cd = (110.0 / agg) as i32;
                            }
                        }
                    }
                    let ahead = e.x + e.f * 7.0;
                    if !Map::floor(self.map.px(ahead, e.y + 1.0)) {
                        e.vx = 0.0;
                    }
                    self.walk_body(&mut e);
                }
                Kind::ForkBomb | Kind::ForkBit => {
                    e.vy = (e.vy + GRAV).min(MAX_FALL);
                    if e.on_ground {
                        e.f = dx.signum();
                        e.vx = e.f
                            * (0.8 + 0.3 * agg)
                            * if e.kind == Kind::ForkBit { 1.3 } else { 1.0 };
                        e.vy = if e.kind == Kind::ForkBomb { -3.0 } else { -2.4 };
                        e.on_ground = false;
                    }
                    self.walk_body(&mut e);
                }
                Kind::Wraith => {
                    e.x += e.vx;
                    e.y = e.base_y + (e.t as f32 * 0.15).sin() * 1.5;
                    if (e.x - self.cx).abs() > W as f32 + 120.0 {
                        e.hp = 0;
                    }
                }
                Kind::Pendulum => {
                    let a = (e.t as f32 * 0.035).sin() * 1.1;
                    e.x = e.ax + a.sin() * 64.0;
                    e.y = e.ay + a.cos() * 64.0;
                }
                Kind::Icicle => match e.state {
                    0 => {
                        if (dx).abs() < 18.0 && py > e.y {
                            e.state = 1;
                            e.cd = 22;
                        }
                    }
                    1 => {
                        e.cd -= 1;
                        if e.cd <= 0 {
                            e.state = 2;
                        }
                    }
                    _ => {
                        e.vy = (e.vy + 0.3).min(6.0);
                        e.y += e.vy;
                        if Map::solid(self.map.px(e.x, e.y))
                            || Map::floor(self.map.px(e.x, e.y)) && e.vy > 1.0
                        {
                            e.hp = 0;
                            self.fx.sparks(e.x, e.y, 6, -1.57);
                            out.play(Sfx::Clang);
                        }
                    }
                },
                Kind::Urn => {
                    e.vy = (e.vy + GRAV).min(MAX_FALL);
                    self.walk_body(&mut e);
                }
            }
            // Touching NIRUX.
            let harmless = e.kind == Kind::Urn || (e.kind == Kind::Icicle && e.state < 2);
            if !harmless && e.tangible() && (e.kind != Kind::Zombie || e.rise == 0) {
                let (hw, hh) = e.kind.size();
                let ph = self.ph();
                if (e.x - px).abs() < hw + PW - 1.0 && e.y > py - ph + 2.0 && e.y - hh < py - 1.0 {
                    hurt_from = Some(e.x);
                }
                // The knight's spear.
                if e.kind == Kind::Knight && e.state == 2 && e.cd > 18 {
                    let tip = e.x + e.f * 16.0;
                    if (tip - px).abs() < 6.0
                        && (e.y - 12.0 - (py - ph / 2.0)).abs() < ph / 2.0 + 2.0
                    {
                        hurt_from = Some(e.x);
                    }
                }
            }
            // Far behind and forgotten.
            if e.x < self.cx - 200.0 && e.kind != Kind::Wraith && e.kind != Kind::Pendulum
                || e.kind == Kind::Pendulum && e.ax < self.cx - 400.0
            {
                e.hp = 0;
            }
            self.foes[i] = e;
        }
        self.eshots.extend(new_shots);
        if let Some(x) = hurt_from {
            self.hit_player(st, out, x);
        }
        self.foes.retain(|e| e.hp > 0);
    }

    /// Gravity and the floor for things that walk.
    fn walk_body(&self, e: &mut Foe) {
        let nx = e.x + e.vx;
        let (hw, hh) = e.kind.size();
        if Map::solid(self.map.px(nx + hw * e.vx.signum(), e.y - hh / 2.0)) {
            e.vx = -e.vx * 0.5;
        } else {
            e.x = nx;
        }
        let ny = e.y + e.vy;
        e.on_ground = false;
        if e.vy >= 0.0 {
            let row = (ny / T as f32).floor();
            let top = row * T as f32;
            if Map::floor(self.map.px(e.x, ny)) && e.y <= top + 0.01 {
                e.y = top;
                e.vy = 0.0;
                e.on_ground = true;
                return;
            }
        }
        e.y = ny;
        if e.y > (self.map.h * T) as f32 + 30.0 {
            e.hp = 0;
        }
    }

    fn update_shots(&mut self, st: &mut Stats, out: &mut Out) {
        let mut i = 0;
        while i < self.shots.len() {
            let mut s = std::mem::replace(
                &mut self.shots[i],
                PShot {
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    kind: Shot::Bolt,
                    dmg: 0,
                    life: 0,
                    bounces: 0,
                    hit: Vec::new(),
                    big: false,
                },
            );
            s.life = s.life.saturating_sub(1);
            if s.kind == Shot::Hammer {
                s.vy += 0.17;
            }
            let nx = s.x + s.vx;
            let ny = s.y + s.vy;
            let mut alive = s.life > 0;
            if Map::solid(self.map.px(nx, ny)) {
                match s.kind {
                    Shot::Disc if s.bounces > 0 => {
                        s.bounces -= 1;
                        if Map::solid(self.map.px(nx, s.y)) {
                            s.vx = -s.vx;
                        } else {
                            s.vy = -s.vy;
                        }
                        out.play(Sfx::Clang);
                    }
                    Shot::Hammer => {
                        self.fx.sparks(s.x, s.y, 6, -1.57);
                        alive = false;
                    }
                    _ => {
                        self.fx.sparks(
                            s.x,
                            s.y,
                            3,
                            if s.vx > 0.0 {
                                std::f32::consts::PI
                            } else {
                                0.0
                            },
                        );
                        alive = false;
                    }
                }
            } else {
                s.x = nx;
                s.y = ny;
            }
            if (s.x - self.cx) < -30.0
                || (s.x - self.cx) > W as f32 + 30.0
                || s.y > (self.map.h * T) as f32 + 20.0
            {
                alive = false;
            }
            // Enemies.
            if alive {
                let r = if s.big { 5.0 } else { 3.0 };
                for j in 0..self.foes.len() {
                    if s.hit.contains(&j) {
                        continue;
                    }
                    let e = &self.foes[j];
                    let (hw, hh) = e.kind.size();
                    if (e.x - s.x).abs() < hw + r
                        && s.y > e.y - hh - r
                        && s.y < e.y + r
                        && e.hp > 0
                        && e.tangible()
                    {
                        let from = if s.kind == Shot::Hammer && s.vy > 0.0 {
                            e.x - e.f * 10.0
                        } else {
                            s.x
                        };
                        let pierce = s.kind == Shot::Hammer;
                        if self.damage_foe(j, s.dmg, from, st, out, pierce) {
                            s.hit.push(j);
                            if !pierce {
                                alive = false;
                                break;
                            }
                        }
                    }
                }
            }
            if alive {
                if let Some(b) = self.boss.as_mut() {
                    if b.hurtbox_hit(s.x, s.y, 4.0) && !s.hit.contains(&usize::MAX) {
                        b.damage(s.dmg, &mut self.fx, out);
                        s.hit.push(usize::MAX);
                        if s.kind != Shot::Hammer {
                            alive = false;
                        }
                    }
                }
            }
            if alive {
                self.shots[i] = s;
                i += 1;
            } else {
                self.shots.swap_remove(i);
            }
        }
        // The whip.
        if let Some((x0, y0, x1, y1)) = self.whip_box(st) {
            let dmg = 3;
            for j in 0..self.foes.len() {
                let e = &self.foes[j];
                let (hw, hh) = e.kind.size();
                if e.x + hw > x0 && e.x - hw < x1 && e.y > y0 && e.y - hh < y1 && e.flash == 0 {
                    let x = self.px;
                    self.damage_foe(j, dmg, x, st, out, true);
                }
            }
            if self.whip_t == 6 {
                if let Some(b) = self.boss.as_mut() {
                    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                    if b.hurtbox_hit(cx, cy, (x1 - x0) / 2.0) {
                        b.damage(dmg, &mut self.fx, out);
                    }
                }
            }
        }
        // Enemy shots.
        let ph = self.ph();
        let mut hit = None;
        for s in self.eshots.iter_mut() {
            s.x += s.vx;
            s.y += s.vy;
            s.vy += s.g;
            s.life = s.life.saturating_sub(1);
            if (s.x - self.px).abs() < PW + 2.0 && s.y > self.py - ph && s.y < self.py {
                hit = Some(s.x);
                s.life = 0;
            }
            if s.g > 0.0 && Map::solid(self.map.px(s.x, s.y)) {
                s.life = 0;
                self.fx.sparks(s.x, s.y, 3, -1.57);
            }
        }
        self.eshots
            .retain(|s| s.life > 0 && s.y < (self.map.h * T) as f32 + 10.0);
        if let Some(x) = hit {
            self.hit_player(st, out, x);
        }
    }

    fn update_picks(&mut self, st: &mut Stats, out: &mut Out) {
        let (px, py) = (self.px, self.py);
        let mut taken = Vec::new();
        for (i, p) in self.picks.iter_mut().enumerate() {
            p.t += 1;
            p.vy = (p.vy + 0.15).min(3.0);
            let ny = p.y + p.vy;
            if p.vy > 0.0 && Map::floor(self.map.px(p.x, ny)) {
                p.y = (ny / T as f32).floor() * T as f32;
                p.vy = 0.0;
            } else {
                p.y = ny;
            }
            if (p.x - px).abs() < 9.0 && p.y > py - 24.0 && p.y < py + 4.0 && self.dead == 0 {
                taken.push(i);
            }
        }
        for &i in taken.iter().rev() {
            let p = self.picks.swap_remove(i);
            match p.drop {
                Drop::Gems(_) => {
                    st.gems += 1;
                    st.add(100);
                    out.play(Sfx::Gem);
                    if st.gems % 100 == 0 {
                        st.lives = (st.lives + 1).min(9);
                        out.play(Sfx::ExtraLife);
                        self.fx.text(p.x, p.y - 16.0, "1UP".into(), LGREEN);
                    }
                }
                Drop::Weapon(w) => {
                    if st.weapon == w {
                        st.power = (st.power + 1).min(3);
                    } else {
                        st.weapon = w;
                    }
                    out.play(Sfx::PowerUp);
                    let s = if st.power > 1 && st.weapon == w {
                        format!("{} {}", w.name(), ["", "", "II", "III"][st.power as usize])
                    } else {
                        w.name().to_string()
                    };
                    self.fx.text(p.x, p.y - 16.0, s, w.color());
                }
                Drop::Thread => {
                    if st.scarf < SCARF_MAX {
                        st.scarf += 1;
                    } else {
                        st.add(1000);
                    }
                    out.play(Sfx::Mend);
                    self.fx.text(p.x, p.y - 16.0, "SCARF MENDED".into(), LRED);
                }
                Drop::Life => {
                    st.lives = (st.lives + 1).min(9);
                    out.play(Sfx::ExtraLife);
                    self.fx.text(p.x, p.y - 16.0, "1UP".into(), LGREEN);
                }
                Drop::Nothing => {}
            }
        }
        self.picks.retain(|p| p.t < 900);
    }

    fn update_world(&mut self, st: &mut Stats, out: &mut Out) {
        for (x, y) in self.map.update() {
            out.play(Sfx::Crumble);
            for k in 0..4 {
                self.fx.p.push(Part {
                    x: (x * T) as f32 + k as f32 * 2.0,
                    y: (y * T) as f32 + 2.0,
                    vx: self.rng.s() * 0.4,
                    vy: 0.5,
                    life: 40,
                    max: 40,
                    kind: Pk::Debris,
                    c: GREY,
                    r: 1.0,
                });
            }
        }
        // Lightning now and then while it rains.
        self.bolt = self.bolt.saturating_sub(1);
        if self.rain > 30 && self.rng.chance(0.0025) && self.bolt == 0 {
            self.thunder(out);
        }
        // Shrines.
        for s in self.shrines.iter_mut() {
            if !s.lit
                && (s.x - self.px).abs() < 10.0
                && (s.y - self.py).abs() < 24.0
                && self.dead == 0
            {
                s.lit = true;
                self.respawn = (s.x, s.y);
                if st.scarf < SCARF_MAX {
                    st.scarf += 1;
                }
                st.lamp = 100.0;
                out.play(Sfx::Mend);
                self.fx.text(s.x, s.y - 34.0, "SHRINE".into(), LGREEN);
                self.say = Some((Who::World, "THE SHRINE MENDS YOUR SCARF", self.t));
            }
        }
        // The bells.
        if let Some(b) = self.bells {
            let k = self.t - b;
            if k == 1 || k == 70 || k == 140 {
                out.play(Sfx::Bell);
                self.fx.shake = 2.0;
            }
            if k == 200 {
                let gy = self.py;
                let from_right = self.rng.chance(0.5);
                let agg = self.agg();
                for n in 0..6 {
                    let high = n < 3;
                    let x = if from_right {
                        self.cx + W as f32 + 20.0 + n as f32 * 46.0
                    } else {
                        self.cx - 20.0 - n as f32 * 46.0
                    };
                    let mut w = Foe::new(Kind::Wraith, x, if high { gy - 15.0 } else { gy - 1.0 });
                    w.base_y = w.y;
                    w.vx = if from_right { -1.6 } else { 1.6 } * (0.8 + 0.25 * agg);
                    w.f = w.vx.signum();
                    self.foes.push(w);
                }
                self.bells = None;
            }
        }
        // The Reaper.
        if let Some(r) = self.reaper.as_mut().filter(|r| r.ally) {
            r.t += 1;
            // A step behind NIRUX, on the ground.
            let want = self.px - self.f * 34.0;
            r.x += (want - r.x) * 0.04;
            let tx = (r.x / T as f32) as i32;
            r.y = (self.map.top_at(tx, ((self.py - 40.0) / T as f32) as i32) * T) as f32;
            if r.t % 80 == 40 {
                let (rx, ry) = (r.x, r.y);
                let mut reaped = false;
                for e in self.foes.iter_mut() {
                    let near = (e.x - rx).abs() < 120.0 && (e.y - ry).abs() < 80.0;
                    if near
                        && matches!(
                            e.kind,
                            Kind::Zombie
                                | Kind::Orphan
                                | Kind::Daemon
                                | Kind::Wraith
                                | Kind::Knight
                        )
                    {
                        e.hp = 0;
                        reaped = true;
                        self.fx.explode(e.x, e.y - 8.0, 0.8);
                    }
                }
                if reaped {
                    out.play(Sfx::Whip);
                }
            }
            if r.t > ALLY_FRAMES {
                self.reaper = None;
            }
        } else if let Some(r) = self.reaper.as_mut() {
            r.t += 1;
            if r.t == 60 {
                self.bolt = 6;
                self.bolt_x = (r.x - self.cx) as i32;
                out.play(Sfx::Thunder);
                self.fx.flash = 4;
            }
            if r.t == 110 {
                // It reaps: every risen dead in sight falls.
                out.play(Sfx::Whip);
                for e in self.foes.iter_mut() {
                    if e.kind == Kind::Zombie
                        && (e.x - self.cx) > -10.0
                        && (e.x - self.cx) < W as f32 + 10.0
                    {
                        e.hp = 0;
                        self.fx.explode(e.x, e.y - 10.0, 0.8);
                    }
                }
            }
            if r.t > 260 {
                self.reaper = None;
            }
        }
        // Triggers.
        for i in 0..self.triggers.len() {
            if self.triggers[i].done || self.px < self.triggers[i].x {
                continue;
            }
            self.triggers[i].done = true;
            match self.triggers[i].ev.clone() {
                Ev::Say(w, s) => self.say = Some((w, s, self.t)),
                Ev::Bells => {
                    self.bells = Some(self.t);
                    self.say = Some((Who::World, "THE CRON BELLS RING", self.t));
                }
                Ev::Reaper => {
                    let x = self.px + 150.0;
                    let tx = (x / T as f32) as i32;
                    let y = (self.map.top_at(tx, 0) * T) as f32;
                    self.reaper = Some(Reaper {
                        x,
                        y,
                        t: 0,
                        ally: false,
                    });
                    self.say = Some((Who::World, "THE REAPER WATCHES", self.t));
                }
                Ev::ReaperAlly => {
                    let x = self.px - 40.0;
                    self.reaper = Some(Reaper {
                        x,
                        y: self.py,
                        t: 0,
                        ally: true,
                    });
                }
                Ev::Rain(n) => self.rain = n,
                Ev::Thunder => self.thunder(out),
                Ev::Boss(k) => {
                    self.locked = true;
                    self.boss_kind = Some(k);
                    let floor =
                        (self.map.top_at(((self.arena + 200.0) / T as f32) as i32, 0) * T) as f32;
                    self.boss = Some(Boss::new(k, self.arena, floor, self.d));
                    out.music = Some(if k == BossKind::Null {
                        SongId::FinalBoss
                    } else {
                        SongId::Boss
                    });
                    out.play(Sfx::Alarm);
                    // The stragglers leave.
                    self.foes.retain(|e| e.x > self.arena);
                }
            }
        }
    }

    fn thunder(&mut self, out: &mut Out) {
        self.bolt = 8;
        self.bolt_x = self.rng.below(W as u32) as i32;
        self.fx.flash = 3;
        out.play(Sfx::Thunder);
    }

    fn update_camera(&mut self) {
        let max_x = self.length - W as f32;
        let tx = if self.locked {
            self.arena
        } else {
            (self.px - W as f32 / 2.0 + self.f * 30.0).clamp(0.0, max_x.max(0.0))
        };
        self.cx += (tx - self.cx) * if self.locked { 0.08 } else { 0.12 };
        if self.locked && (self.cx - self.arena).abs() < 0.5 {
            self.cx = self.arena;
        }
        let max_y = (self.map.h * T) as f32 - H as f32;
        let ty = (self.py - 120.0).clamp(0.0, max_y.max(0.0));
        self.cy += (ty - self.cy) * 0.1;
    }

    /// The autopilot: run right, jump at walls, gaps and enemies, keep
    /// firing, light the lamp over water, climb when stuck below a ladder.
    fn bot_input(&mut self, st: &Stats) -> Input {
        let mut i = Input::default();
        if self.dead > 0 || self.hurt > 0 {
            return i;
        }
        let x = self.px;
        let y = self.py;
        // In the arena: keep a distance on solid footing, face the boss,
        // fire, and jump what comes.
        if let Some(b) = &self.boss {
            let (bx, by) = b.center();
            let side = if x < bx { -1.0 } else { 1.0 };
            // A boss up high: stand off as far as it is high, and aim up.
            let high = by < y - 36.0;
            let dist = if high {
                (y - 14.0 - by).clamp(40.0, 110.0)
            } else {
                76.0
            };
            let side = if (bx - self.arena) < 70.0 {
                1.0
            } else if (self.arena + W as f32 - bx) < 70.0 {
                -1.0
            } else {
                side
            };
            let want = bx + side * dist;
            let mut best = x;
            let mut bd = f32::MAX;
            let mut c = self.arena + 12.0;
            while c < self.arena + W as f32 - 12.0 {
                let v = self.map.px(c, y + 1.0);
                let solid_foot = (0..3).all(|k| {
                    let v = self.map.px(c - 4.0 + k as f32 * 4.0, y + 1.0);
                    Map::floor(v)
                });
                if Map::floor(v) && solid_foot && (c - want).abs() < bd {
                    bd = (c - want).abs();
                    best = c;
                }
                c += 4.0;
            }
            let d = best - x;
            if d.abs() > 4.0 && self.grounded {
                if d > 0.0 {
                    i.right = true;
                } else {
                    i.left = true;
                }
                let dir = d.signum();
                let edge = x + dir * (PW - 1.0);
                if !Map::floor(self.map.px(edge, y + 1.0)) {
                    i.jump = true;
                    i.p_jump = true;
                }
            } else if self.grounded {
                self.f = (bx - x).signum();
            }
            if !self.grounded {
                // Steer the landing onto the chosen footing.
                if (best - x).abs() > 3.0 {
                    if best > x {
                        i.right = true;
                    } else {
                        i.left = true;
                    }
                }
                if self.vy < 0.0 {
                    i.jump = true;
                }
            }
            i.fire = self.t % 2 == 0;
            i.up = high && self.grounded && (best - x).abs() <= 4.0;
            let danger = self
                .eshots
                .iter()
                .any(|s| (s.x - x).abs() < 34.0 && (s.y - (y - 10.0)).abs() < 30.0)
                || b.threat(x, y);
            if danger && self.grounded {
                i.jump = true;
                i.p_jump = true;
            }
            return i;
        }
        let f = 1.0;
        i.right = true;
        i.fire = self.t % 2 == 0 && !self.foes.is_empty();
        let ahead = x + f * 10.0;
        let wall =
            Map::solid(self.map.px(ahead, y - 4.0)) || Map::solid(self.map.px(ahead, y - 14.0));
        // Jump at the very edge, for the longest leap.
        let edge = x + f * (PW - 1.0);
        let under = self.map.px(edge, y + 1.0);
        let gap =
            !Map::floor(under) || Map::liquid(self.map.px(edge, y + 9.0)) && !Map::floor(under);
        let foe_near = self.foes.iter().any(|e| {
            e.tangible()
                && e.kind != Kind::Urn
                && (e.x - x) * f > 0.0
                && (e.x - x).abs() < 30.0
                && (e.y - y).abs() < 20.0
        }) || self
            .eshots
            .iter()
            .any(|s| (s.x - x).abs() < 30.0 && (s.y - y).abs() < 24.0);
        let wraith_high = self
            .foes
            .iter()
            .any(|e| e.kind == Kind::Wraith && (e.x - x).abs() < 50.0 && e.y < y - 8.0);
        let small = self.foes.iter().any(|e| {
            matches!(e.kind, Kind::Orphan | Kind::ForkBit)
                && (e.x - x) * f > 0.0
                && (e.x - x).abs() < 70.0
                && (e.y - y).abs() < 6.0
        });
        let firm = Map::solid(self.map.px(x, y + 1.0));
        if small && self.grounded && firm && !gap {
            i.fire = self.t % 2 == 0;
            i.down = self.t % 60 < 40;
            i.right = self.t % 60 >= 40;
            return i;
        }
        if wraith_high && self.grounded {
            i.right = false;
            i.down = true;
        } else if self.grounded && (wall || gap || foe_near) {
            i.jump = true;
            i.p_jump = true;
        } else if !self.grounded && self.vy < 0.0 {
            i.jump = true;
        }
        // Coming down: brake over a ledge rather than sail past it.
        if !self.grounded && self.vy > 0.0 {
            let floor_at = |px: f32| {
                (0..12).any(|k| {
                    let v = self.map.px(px, y + 2.0 + k as f32 * 6.0);
                    Map::floor(v) && !Map::liquid(v)
                })
            };
            if floor_at(x) && !floor_at(x + f * 18.0) {
                i.right = false;
                i.left = self.vx > 0.6;
            }
        }
        // A ladder up when the way ahead is a high wall.
        if wall && Map::ladder(self.map.px(x, y - 4.0)) {
            i.up = true;
            i.jump = false;
            i.p_jump = false;
        }
        if self.climbing {
            i.up = true;
            i.right = false;
        }
        // Light the way where hidden platforms are.
        let tx = (x / T as f32) as i32;
        let hidden_ahead =
            (tx..tx + 12).any(|c| (0..self.map.h).any(|r| self.map.at(c, r) == world::HIDDEN));
        if hidden_ahead && st.lamp > 10.0 {
            i.special = true;
        }
        if self.t % 120 < 40
            && self
                .foes
                .iter()
                .any(|e| e.kind == Kind::Ghost && (e.x - x).abs() < 90.0)
            && st.lamp > 30.0
        {
            i.special = true;
        }
        i
    }

    // Drawing.

    fn draw_lamp(&self, fb: &mut Fb, ox: i32, oy: i32) {
        if !self.lamp_on {
            return;
        }
        let (lx, ly) = (
            (self.px + self.f * 3.0) as i32 + ox,
            (self.py - 20.0) as i32 + oy,
        );
        // The beam brightens what it falls on, in a dithered cone.
        let bright: [u8; 16] = [11, 1, 10, 14, 10, 13, 14, 1, 15, 8, 7, 12, 15, 1, 3, 1];
        for dx in 4..100 {
            let half = (dx as f32 * 0.45 + 6.0) as i32;
            let x = lx + dx * self.f as i32;
            if !(0..W).contains(&x) {
                continue;
            }
            for dy in -half..=half {
                let y = ly + dy;
                if !(HUD_H..H).contains(&y) {
                    continue;
                }
                let edge = dy.abs() > half - 3 || dx > 85;
                if (x + y) % 2 == 0 || (!edge && (x + y) % 4 == 1) {
                    let i = (y * W + x) as usize;
                    let c = fb.px[i];
                    if (c as usize) < 16 {
                        fb.px[i] = bright[c as usize];
                    }
                }
            }
        }
        fb.set(lx + self.f as i32, ly, WHITE);
    }

    fn draw_shrine(&self, fb: &mut Fb, s: &Shrine, ox: i32, oy: i32) {
        let (x, y) = (s.x as i32 + ox, s.y as i32 + oy);
        fb.rect(x - 6, y - 8, 13, 8, GREY);
        fb.hline(x - 7, x + 7, y - 9, LGREY);
        fb.rect(x - 4, y - 16, 9, 7, DGREY);
        fb.hline(x - 5, x + 5, y - 17, GREY);
        // The Core's mark, a small diamond.
        fb.set(x, y - 5, if s.lit { LGREEN } else { DGREY });
        fb.set(x - 1, y - 4, if s.lit { GREEN } else { BLACK });
        fb.set(x + 1, y - 4, if s.lit { GREEN } else { BLACK });
        fb.set(x, y - 3, if s.lit { LGREEN } else { DGREY });
        if s.lit {
            for k in 0..6 {
                let fy = y - 18 - k - ((self.t / 3 + k as u32) % 3) as i32;
                let w = (6 - k) / 2;
                fb.hline(
                    x - w,
                    x + w,
                    fy,
                    if k < 2 {
                        LGREEN
                    } else if k < 4 {
                        GREEN
                    } else {
                        WHITE
                    },
                );
            }
        }
    }

    fn draw_reaper(&self, fb: &mut Fb, r: &Reaper, ox: i32, oy: i32) {
        let (x, y) = (r.x as i32 + ox, r.y as i32 + oy);
        let end = if r.ally { ALLY_FRAMES } else { 260 };
        let fade = if r.t < 40 {
            r.t as i32 / 3
        } else if r.t + 40 > end {
            (end as i32 - r.t as i32).max(0) / 3
        } else {
            16
        };
        let put = |fb: &mut Fb, px: i32, py: i32, c: u8| {
            if world::dith(px, py, fade) {
                fb.set(px, py, c);
            }
        };
        // A tall hooded figure in a ragged cloak, the scythe raised.
        for yy in 0..34 {
            let w = 3 + yy / 5;
            for xx in -w..=w {
                let rag = yy > 28 && (xx + (self.t / 8) as i32) % 3 == 0;
                if !rag {
                    put(
                        fb,
                        x + xx,
                        y - 34 + yy,
                        if xx < -w + 2 { BLACK } else { DGREY },
                    );
                }
            }
        }
        for yy in 0..8 {
            for xx in -4..=4 {
                if xx * xx + (yy - 4) * (yy - 4) < 18 {
                    put(fb, x + xx, y - 42 + yy, BLACK);
                }
            }
        }
        put(fb, x - 2, y - 39, LRED);
        put(fb, x + 1, y - 39, LRED);
        let swing = if r.ally {
            let k = r.t % 80;
            if (30..50).contains(&k) {
                (k - 30) as i32
            } else {
                0
            }
        } else if r.t > 100 && r.t < 120 {
            (r.t - 100) as i32
        } else {
            0
        };
        let (hx, hy) = (x + 8, y - 30);
        for k in 0..30 {
            put(fb, hx + k / 6, hy - 18 + k, BROWN);
        }
        for k in 0..14 {
            let a = -1.2 + k as f32 * 0.12 + swing as f32 * 0.15;
            let bx = hx + 3 + (a.cos() * 14.0) as i32 - 14;
            let by = hy - 18 + (a.sin() * 8.0) as i32;
            put(fb, bx, by, LGREY);
            put(fb, bx, by + 1, GREY);
        }
    }
}

impl Level for Stage {
    fn update(&mut self, inp: &Input, st: &mut Stats, out: &mut Out) -> Status {
        self.t += 1;
        if let Some(t0) = self.over_t {
            if self.t - t0 > 60 {
                return Status::Over;
            }
        }
        if let Some(t0) = self.clear_t {
            self.fx.update();
            self.update_player(&Input::default(), st, out);
            if self.py > (self.map.h * T) as f32 {
                (self.px, self.py) = self.safe;
                self.vy = 0.0;
            }
            if let Some(r) = self.reaper.as_mut() {
                r.t += 1;
                if r.t == 60 {
                    self.fx.flash = 4;
                    out.play(Sfx::Thunder);
                }
                if r.t == 110 {
                    out.play(Sfx::Whip);
                }
            }
            let wait = if self.boss_kind == Some(BossKind::Null) {
                420
            } else {
                330
            };
            if self.t - t0 > wait {
                return Status::Cleared;
            }
            return Status::Playing;
        }
        self.update_player(inp, st, out);
        self.update_lamp();
        self.spawn_near(out);
        self.update_foes(st, out);
        self.update_shots(st, out);
        self.update_picks(st, out);
        self.update_world(st, out);
        // The boss.
        let mut boss_done = false;
        if let Some(b) = self.boss.as_mut() {
            let (px, py) = (self.px, self.py);
            let r = b.update(
                px,
                py,
                &mut self.eshots,
                &mut self.foes,
                &mut self.fx,
                out,
                &mut self.rng,
            );
            if b.touches(px, py, if self.crouch { 14.0 } else { 22.0 }) {
                let bx = b.center().0;
                self.hit_player(st, out, bx);
            }
            if r {
                boss_done = true;
            }
        }
        if boss_done {
            let b = self.boss.take();
            if let Some(b) = b {
                st.add(b.score());
                st.kills += 1;
                self.fx.text(
                    b.center().0,
                    b.center().1 - 30.0,
                    b.score().to_string(),
                    YELLOW,
                );
            }
            self.eshots.clear();
            for e in self.foes.iter_mut() {
                e.hp = 0;
            }
            self.clear_t = Some(self.t);
            out.music = Some(SongId::Clear);
            let (who, line) = match self.boss_kind {
                Some(BossKind::Gravedigger) => (Who::Null, "NULL: THE DEAD ARE MANY. YOU ARE ONE."),
                Some(BossKind::BogDaemon) => (Who::Null, "NULL: DEEPER, THEN. I AM WAITING."),
                Some(BossKind::Choir) => (Who::Null, "NULL: THEY SANG FOR ME. NOW SILENCE."),
                Some(BossKind::ClockWarden) => (Who::Null, "NULL: TIME IS MINE DOWN HERE."),
                Some(BossKind::FrostWyrm) => (Who::Null, "NULL: COLD. YOU WILL BE COLDER."),
                Some(BossKind::DaemonLord) => (Who::Null, "NULL: COME, THEN. MY KEEP IS OPEN."),
                _ => (Who::Reaper, "THE REAPER: WHAT NULL WAS, I TAKE AWAY."),
            };
            self.say = Some((who, line, self.t));
            if self.boss_kind == Some(BossKind::Null) {
                // The Reaper comes for what is left of NULL.
                self.reaper = Some(Reaper {
                    x: self.arena + 260.0,
                    y: (self.map.top_at(((self.arena + 260.0) / T as f32) as i32, 0) * T) as f32,
                    t: 0,
                    ally: false,
                });
            }
        }
        self.fx.update();
        self.update_camera();
        Status::Playing
    }

    fn draw(&self, fb: &mut Fb, st: &Stats, hi: u32) {
        let (sx, sy) = self.fx.offset(self.t);
        let (ox, oy) = (-(self.cx as i32) + sx, -(self.cy as i32) + sy);
        world::draw_sky(fb, self.spec.theme, self.cx, self.cy, self.t, self.fx.flash);
        if self.bolt > 0 {
            // A forked bolt down the sky.
            let mut x = self.bolt_x;
            let mut y = HUD_H;
            let mut k = self.bolt_x as u32 * 31 + 7;
            while y < H - 60 {
                k = k.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                let nx = x + ((k >> 16) % 9) as i32 - 4;
                fb.line(x, y, nx, y + 6, WHITE);
                fb.line(x + 1, y, nx + 1, y + 6, LBLUE);
                x = nx;
                y += 6;
            }
        }
        world::draw_deco(fb, &self.map, self.spec.theme, -ox, -oy, self.t);
        for s in &self.shrines {
            self.draw_shrine(fb, s, ox, oy);
        }
        if let Some(r) = &self.reaper {
            self.draw_reaper(fb, r, ox, oy);
        }
        world::draw_tiles(fb, &self.map, self.spec.theme, -ox, -oy, self.t);
        if let Some(b) = &self.boss {
            b.draw(fb, ox, oy, self.t);
        }
        for p in &self.picks {
            let (x, y) = (p.x as i32 + ox, p.y as i32 + oy);
            if p.t > 700 && p.t % 6 < 3 {
                continue;
            }
            match p.drop {
                Drop::Gems(_) => {
                    let c = [YELLOW, WHITE, YELLOW, ORANGE][(self.t / 6 % 4) as usize];
                    fb.set(x, y - 6, c);
                    fb.hline(x - 1, x + 1, y - 5, c);
                    fb.hline(x - 2, x + 2, y - 4, YELLOW);
                    fb.hline(x - 1, x + 1, y - 3, ORANGE);
                    fb.set(x, y - 2, ORANGE);
                }
                Drop::Weapon(w) => {
                    fb.rect(x - 5, y - 11, 11, 11, BLACK);
                    fb.frame(
                        x - 5,
                        y - 11,
                        11,
                        11,
                        if self.t % 16 < 8 { w.color() } else { WHITE },
                    );
                    fb.char(
                        x - 3,
                        y - 9,
                        w.name().chars().next().unwrap_or('?'),
                        w.color(),
                    );
                }
                Drop::Thread => {
                    for k in 0..7 {
                        let wy = ((self.t as f32 * 0.2 + k as f32).sin() * 1.5) as i32;
                        fb.set(x - 3 + k, y - 5 + wy, if k % 2 == 0 { LRED } else { RED });
                        fb.set(x - 3 + k, y - 4 + wy, RED);
                    }
                }
                Drop::Life => {
                    fb.disc(x, y - 5, 4, LGREY);
                    fb.hline(x - 1, x + 3, y - 5, LGREEN);
                    fb.set(x - 2, y - 7, WHITE);
                }
                Drop::Nothing => {}
            }
        }
        for e in &self.foes {
            e.draw(fb, e.x as i32 + ox, e.y as i32 + oy, self.t);
        }
        // NIRUX.
        if self.dead == 0
            && !(self.inv > 0 && self.inv < 100 && (self.t / 3) % 2 == 0 && self.hurt == 0)
        {
            let pose = if self.hurt > 0 {
                Pose::Hurt
            } else if self.climbing {
                Pose::Climb(self.climb_ph)
            } else if !self.grounded {
                Pose::Air(self.vy)
            } else if self.crouch {
                Pose::Crouch
            } else if self.vx.abs() > 0.2 {
                Pose::Run(self.run_ph)
            } else {
                Pose::Stand
            };
            let (x, y) = (self.px as i32 + ox, self.py as i32 + oy);
            let firing = self.muzzle > 0 || self.whip_t > 0;
            hero::side(
                fb,
                x,
                y,
                pose,
                self.f as i32,
                self.t,
                st.scarf,
                firing,
                self.aim_up,
            );
            if let Some((x0, y0, x1, y1)) = self.whip_box(st) {
                // The chain: a bright arc out to its reach.
                let (ax, ay) = (
                    (self.px + self.f * 6.0) as i32 + ox,
                    (self.py - if self.crouch { 7.0 } else { 14.0 }) as i32 + oy,
                );
                let (bx, by) = if self.aim_up {
                    ((x0 + x1) as i32 / 2 + ox, y0 as i32 + oy)
                } else {
                    (
                        (if self.f > 0.0 { x1 } else { x0 }) as i32 + ox,
                        ((y0 + y1) / 2.0) as i32 + oy,
                    )
                };
                let n = 10;
                for k in 0..=n {
                    let u = k as f32 / n as f32;
                    let sag = (u * std::f32::consts::PI).sin() * 4.0;
                    let px = ax as f32 + (bx - ax) as f32 * u;
                    let py = ay as f32 + (by - ay) as f32 * u - sag;
                    fb.set(px as i32, py as i32, if k % 2 == 0 { LGREY } else { LRED });
                }
                fb.disc(bx, by, 1, WHITE);
            }
        } else if self.dead > 0 && self.dead < 80 {
            // The helmet rolls, its visor going dark.
            let (x, y) = (
                (self.px - self.f * self.dead as f32 * 0.3) as i32 + ox,
                self.py as i32 + oy - 3,
            );
            fb.disc(x, y, 3, LGREY);
            fb.set(x + 2, y, if self.dead < 40 { LGREEN } else { DGREY });
        }
        for s in &self.shots {
            let (x, y) = (s.x as i32 + ox, s.y as i32 + oy);
            match s.kind {
                Shot::Bolt => {
                    let d = if s.vx > 0.0 { 1 } else { -1 };
                    let len = if s.big { 7 } else { 5 };
                    for k in 0..len {
                        let c = if k < 2 {
                            WHITE
                        } else if k < 4 {
                            CYAN
                        } else {
                            LBLUE
                        };
                        let yy = y + if s.vy != 0.0 {
                            (k as f32 * -s.vy.signum() * -0.9) as i32
                        } else {
                            0
                        };
                        fb.set(x - d * k, yy, c);
                        if s.big {
                            fb.set(x - d * k, yy + 1, if k < 3 { CYAN } else { BLUE });
                        }
                    }
                }
                Shot::Hammer => {
                    let a = (self.t / 3) % 4;
                    let r = if s.big { 4 } else { 3 };
                    fb.disc(x, y, r, YELLOW);
                    let (hx, hy) = [(r, 0), (0, r), (-r, 0), (0, -r)][a as usize];
                    fb.line(x, y, x + hx * 2, y + hy * 2, ORANGE);
                    fb.set(x, y, WHITE);
                }
                Shot::Disc => {
                    let r = if s.big { 4 } else { 3 };
                    fb.disc(x, y, r, LGREEN);
                    fb.disc(x, y, r - 2, BLACK);
                    let a = (self.t / 2) % 4;
                    let (hx, hy) = [(r, 0), (0, r), (-r, 0), (0, -r)][a as usize];
                    fb.set(x + hx, y + hy, WHITE);
                }
            }
        }
        for s in &self.eshots {
            let (x, y) = (s.x as i32 + ox, s.y as i32 + oy);
            match s.kind {
                EKind::Fire => {
                    fb.disc(x, y, 2, if self.t % 6 < 3 { ORANGE } else { RED });
                    fb.set(x, y, YELLOW);
                    fb.set(x - s.vx.signum() as i32 * 3, y, RED);
                }
                EKind::Clod => {
                    fb.disc(x, y, 2, BROWN);
                    fb.set(x - 1, y - 1, ORANGE);
                    fb.set(x + 1, y, BLACK);
                }
                EKind::Note => {
                    fb.disc(x, y + 1, 2, if self.t % 10 < 5 { WHITE } else { LGREY });
                    fb.line(x + 2, y + 1, x + 2, y - 5, LGREY);
                    fb.line(x + 2, y - 5, x + 4, y - 3, LGREY);
                }
                EKind::Gear => {
                    let a = self.t as f32 * 0.3 * s.vx.signum();
                    fb.circle(x, y, 4, ORANGE);
                    for k in 0..4 {
                        let b = a + k as f32 * 1.571;
                        fb.set(
                            x + (b.cos() * 5.0) as i32,
                            y + (b.sin() * 5.0) as i32,
                            YELLOW,
                        );
                    }
                    fb.set(x, y, BROWN);
                }
                EKind::Shard => {
                    fb.line(
                        x,
                        y,
                        x - (s.vx * 2.0) as i32,
                        y - (s.vy * 2.0) as i32,
                        LBLUE,
                    );
                    fb.set(x, y, WHITE);
                }
                EKind::Static => {
                    for k in 0..5 {
                        let h = (self.t as i32 + k * 7 + x) % 5 - 2;
                        fb.set(x + h, y + k - 2, if k % 2 == 0 { WHITE } else { LRED });
                    }
                }
                EKind::Wave => {
                    let c = if self.t % 4 < 2 { LRED } else { WHITE };
                    fb.line(x - 3, y - 3, x + 3, y + 1, c);
                    fb.line(x - 3, y + 1, x + 3, y - 3, RED);
                }
                EKind::Spit => {
                    fb.disc(x, y, 3, if self.t % 8 < 4 { LGREEN } else { GREEN });
                    fb.set(x - 1, y - 1, WHITE);
                }
            }
        }
        self.fx.draw(fb, ox, oy);
        self.draw_lamp(fb, ox, oy);
        if self.rain > 0 {
            world::draw_rain(fb, self.t, self.rain, self.cx);
        }
        if self.spec.fog {
            world::draw_fog(fb, self.t, self.cx, H - 46);
        }
        if self.fx.flash > 0 {
            let map: [u8; 16] = [11, 1, 15, 1, 14, 13, 14, 1, 15, 12, 15, 15, 15, 1, 3, 1];
            fb.remap(0, HUD_H, W, H - HUD_H, &map);
        }
        // What is said: ECHO in green, NULL in red, the world in white.
        if let Some((who, s, t0)) = self.say {
            let k = self.t - t0;
            if k < 260 {
                let (tag, c, sh) = match who {
                    Who::Echo => ("ECHO: ", LGREEN, GREEN),
                    Who::Null => ("", LRED, RED),
                    Who::World => ("", WHITE, DGREY),
                    Who::Reaper => ("", LGREY, BLACK),
                };
                let shown = (k as usize).min(s.len() + tag.len());
                let full = format!("{tag}{s}");
                let txt = &full[..shown.min(full.len())];
                let x = (W - text_w(&full)) / 2;
                let y = HUD_H + 8;
                fb.rect(x - 4, y - 3, text_w(&full) + 8, 13, BLACK);
                // ECHO's signal grows weaker the deeper NIRUX goes.
                let noise = who == Who::Echo
                    && self.stage_no > 0
                    && (self.t / 3) % (7 - self.stage_no.min(5) as u32) == 0;
                if !noise || k > s.len() as u32 + 30 {
                    fb.text_sh(
                        x,
                        y,
                        txt,
                        if who == Who::Null && k % 8 < 4 {
                            WHITE
                        } else {
                            c
                        },
                        sh,
                    );
                } else {
                    for (i, ch) in txt.chars().enumerate() {
                        let xx = x + i as i32 * 8;
                        if (i as u32 + self.t) % 3 == 0 {
                            fb.char(xx, y, '#', DGREY);
                        } else {
                            fb.char(xx, y, ch, c);
                        }
                    }
                }
            }
        }
        draw_hud(fb, st, hi, self.spec.hud, self.t);
        if let Some(b) = &self.boss {
            let (hp, max) = b.hp();
            draw_boss_bar(fb, b.name(), hp, max, self.t);
        }
    }

    fn bot(&mut self, st: &Stats) -> Input {
        self.bot_input(st)
    }

    fn border(&self) -> u8 {
        if self.fx.flash > 0 {
            WHITE
        } else {
            self.spec.border
        }
    }

    fn progress(&self) -> f32 {
        (self.px / self.length).clamp(0.0, 1.0)
    }

    fn info(&self) -> String {
        format!(
            "x {:.0}/{:.0} y {:.0} g {} vy {:.1} vx {:.1} foes {} boss {:?}",
            self.px,
            self.length,
            self.py,
            self.grounded,
            self.vy,
            self.vx,
            self.foes.len(),
            self.boss.as_ref().map(|b| b.hp())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{make_level, LEVELS};

    #[test]
    fn the_autopilot_clears_every_stage() {
        for lvl in 0..LEVELS {
            for d in [Difficulty::Easy, Difficulty::Normal] {
                let mut l = make_level(lvl, d, 0xC64 + lvl as u32);
                let mut st = Stats::new(9);
                let mut status = Status::Playing;
                let mut n = 0;
                while n < 40_000 && status == Status::Playing {
                    let inp = l.bot(&st);
                    let mut out = Out::default();
                    status = l.update(&inp, &mut st, &mut out);
                    st.lives = st.lives.max(3);
                    n += 1;
                    if std::env::var_os("NIRUX_TRACE").is_some() && n % 500 == 0 {
                        println!(
                            "{lvl} {d:?} {n}: {} {:?}",
                            l.info(),
                            (inp.right, inp.jump, inp.down, inp.fire)
                        );
                    }
                }
                assert_eq!(
                    status,
                    Status::Cleared,
                    "stage {} on {d:?}: {}",
                    lvl + 1,
                    l.info()
                );
            }
        }
    }
}
