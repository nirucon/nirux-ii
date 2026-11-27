//! The bosses: the Gravedigger, who buries the dead of the Process
//! Graveyard and digs up more, and the Bog Daemon of the Swap Marsh, who
//! rises from the black water where it chooses.
use super::common::*;
use super::foes::{Foe, Kind};
use super::stage::{EKind, EShot};
use crate::config::Difficulty;
use crate::fb::Fb;
use crate::palette::*;
use crate::sfx::Sfx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossKind {
    Gravedigger,
    BogDaemon,
}

pub struct Boss {
    pub kind: BossKind,
    pub x: f32,
    pub y: f32,
    arena: f32,
    floor: f32,
    hp: i32,
    max: i32,
    state: u8,
    st: u32,
    t: u32,
    flash: u8,
    pub dying: u32,
    agg: f32,
    f: f32,
    /// Underground or under water: where it will come up.
    target: f32,
    spots: [f32; 3],
    shots_left: u8,
}

impl Boss {
    pub fn new(kind: BossKind, arena: f32, floor: f32, d: Difficulty) -> Boss {
        // The Bog Daemon's floor is the water, a row below the islands.
        let floor = if kind == BossKind::BogDaemon {
            floor + 8.0
        } else {
            floor
        };
        let max = match (kind, d) {
            (BossKind::Gravedigger, Difficulty::Easy) => 40,
            (BossKind::Gravedigger, Difficulty::Normal) => 55,
            (BossKind::Gravedigger, Difficulty::Hard) => 70,
            (BossKind::BogDaemon, Difficulty::Easy) => 55,
            (BossKind::BogDaemon, Difficulty::Normal) => 75,
            (BossKind::BogDaemon, Difficulty::Hard) => 95,
        };
        Boss {
            kind,
            x: arena + 230.0,
            y: floor,
            arena,
            floor,
            hp: max,
            max,
            state: 0,
            st: 0,
            t: 0,
            flash: 0,
            dying: 0,
            agg: d.aggression(),
            f: -1.0,
            target: arena + 230.0,
            spots: [arena + 72.0, arena + 160.0, arena + 248.0],
            shots_left: 0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self.kind {
            BossKind::Gravedigger => "THE GRAVEDIGGER",
            BossKind::BogDaemon => "THE BOG DAEMON",
        }
    }

    pub fn score(&self) -> u32 {
        match self.kind {
            BossKind::Gravedigger => 10000,
            BossKind::BogDaemon => 15000,
        }
    }

    pub fn hp(&self) -> (i32, i32) {
        (self.hp, self.max)
    }

    pub fn center(&self) -> (f32, f32) {
        match self.kind {
            BossKind::Gravedigger => (self.x, self.y - 24.0),
            BossKind::BogDaemon => (self.x, self.floor - 40.0),
        }
    }

    fn visible(&self) -> bool {
        match self.kind {
            BossKind::Gravedigger => !matches!(self.state, 4),
            BossKind::BogDaemon => matches!(self.state, 1..=3),
        }
    }

    fn vulnerable(&self) -> bool {
        self.dying == 0
            && match self.kind {
                BossKind::Gravedigger => matches!(self.state, 1 | 2 | 5 | 6),
                BossKind::BogDaemon => self.state == 2,
            }
    }

    pub fn hurtbox_hit(&self, x: f32, y: f32, r: f32) -> bool {
        if !self.vulnerable() {
            return false;
        }
        let (cx, cy, hw, hh) = match self.kind {
            BossKind::Gravedigger => (self.x, self.y - 22.0, 12.0, 22.0),
            BossKind::BogDaemon => (self.x, self.floor - 38.0, 17.0, 22.0),
        };
        (x - cx).abs() < hw + r && (y - cy).abs() < hh + r
    }

    pub fn damage(&mut self, dmg: i32, fx: &mut Fx, out: &mut Out) {
        if !self.vulnerable() {
            return;
        }
        self.hp -= dmg;
        self.flash = 4;
        out.play(Sfx::Hit);
        let (cx, cy) = self.center();
        fx.sparks(cx, cy, 2, -1.57);
        if self.hp <= 0 && self.dying == 0 {
            self.dying = 1;
            out.play(Sfx::ExplodeBig);
        }
    }

    pub fn touches(&self, px: f32, py: f32, ph: f32) -> bool {
        if self.dying > 0 || !self.visible() {
            return false;
        }
        match self.kind {
            BossKind::Gravedigger => {
                let body = (px - self.x).abs() < 14.0 && py > self.y - 40.0 && py - ph < self.y;
                // The shovel striking up out of the ground.
                let strike = self.state == 5
                    && self.st < 26
                    && (px - self.x).abs() < 14.0
                    && py - ph < self.y
                    && py > self.y - 56.0;
                body || strike
            }
            BossKind::BogDaemon => {
                let body =
                    (px - self.x).abs() < 18.0 && py > self.floor - 60.0 && py - ph < self.floor;
                // The sweep of its arm over the water.
                let sweep = self.state == 2 && (140..170).contains(&self.st) && {
                    let reach = (self.st - 140) as f32 * 9.0;
                    let ax = self.x + self.f * reach;
                    (px - ax).abs() < 12.0 && py > self.floor - 30.0 && py - ph < self.floor - 6.0
                };
                body || sweep
            }
        }
    }

    /// Something that will hurt soon near (x, y) (for the autopilot).
    pub fn threat(&self, x: f32, _y: f32) -> bool {
        match self.kind {
            BossKind::Gravedigger => {
                self.state == 4 && self.st > 40 && (self.target - x).abs() < 20.0
            }
            BossKind::BogDaemon => {
                self.state == 2 && (128..165).contains(&self.st) && (x - self.x).abs() < 160.0
            }
        }
    }

    /// One frame; true when it is gone and the stage is won.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        px: f32,
        py: f32,
        shots: &mut Vec<EShot>,
        foes: &mut Vec<Foe>,
        fx: &mut Fx,
        out: &mut Out,
        rng: &mut Rng,
    ) -> bool {
        self.t += 1;
        self.st += 1;
        self.flash = self.flash.saturating_sub(1);
        if self.dying > 0 {
            self.dying += 1;
            let (cx, cy) = self.center();
            if self.dying % 8 == 0 {
                fx.explode(cx + rng.s() * 20.0, cy + rng.s() * 20.0, 1.5);
                out.play(Sfx::ExplodeSmall);
            }
            fx.shake = fx.shake.max(2.0);
            if self.dying == 100 {
                fx.explode(cx, cy, 5.0);
                fx.flash = 4;
                out.play(Sfx::ExplodeBig);
            }
            return self.dying > 140;
        }
        let rage = if self.hp * 2 < self.max { 1.35 } else { 1.0 };
        let agg = self.agg * rage;
        match self.kind {
            BossKind::Gravedigger => self.gravedigger(px, py, shots, foes, fx, out, rng, agg),
            BossKind::BogDaemon => self.bog(px, py, shots, fx, out, rng, agg),
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn gravedigger(
        &mut self,
        px: f32,
        _py: f32,
        shots: &mut Vec<EShot>,
        foes: &mut Vec<Foe>,
        fx: &mut Fx,
        out: &mut Out,
        rng: &mut Rng,
        agg: f32,
    ) {
        let lo = self.arena + 24.0;
        let hi = self.arena + 296.0;
        match self.state {
            // Climbing out of the big grave.
            0 => {
                if self.st % 6 == 0 {
                    fx.sparks(self.x + rng.s() * 12.0, self.y, 2, -1.57);
                }
                if self.st == 1 {
                    out.play(Sfx::Rise);
                }
                if self.st > 80 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            // Walking at NIRUX, shovel over the shoulder.
            1 => {
                self.f = (px - self.x).signum();
                if (px - self.x).abs() > 30.0 {
                    self.x += self.f * 0.45 * agg;
                }
                self.x = self.x.clamp(lo, hi);
                if self.st > (110.0 / agg) as u32 {
                    self.st = 0;
                    let r = rng.f();
                    self.state = if self.hp * 2 < self.max && r < 0.25 && foes.len() < 4 {
                        6
                    } else if r < 0.6 {
                        2
                    } else {
                        3
                    };
                    self.shots_left = if agg > 1.2 { 4 } else { 3 };
                }
            }
            // Flinging clods and skulls in arcs.
            2 => {
                self.f = (px - self.x).signum();
                if self.st % 16 == 0 && self.shots_left > 0 {
                    self.shots_left -= 1;
                    let k = self.shots_left as f32;
                    let dist = (px - self.x) + rng.s() * 20.0 + (k - 1.5) * 18.0;
                    let air = 52.0;
                    shots.push(EShot {
                        x: self.x + self.f * 14.0,
                        y: self.y - 36.0,
                        vx: dist / air,
                        vy: -3.6,
                        g: 0.13,
                        kind: EKind::Clod,
                        life: 200,
                    });
                    out.play(Sfx::EnemyShot);
                }
                if self.shots_left == 0 && self.st > 40 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            // Digging down into the earth.
            3 => {
                if self.st % 4 == 0 {
                    fx.sparks(self.x + rng.s() * 10.0, self.y, 2, -1.57);
                }
                if self.st > 40 {
                    self.state = 4;
                    self.st = 0;
                    self.target = px;
                    out.play(Sfx::Crumble);
                }
            }
            // Under the ground: the earth stirs where it will come up,
            // following NIRUX, then stops (that is the moment to move).
            4 => {
                if self.st < 50 {
                    self.target += (px - self.target).clamp(-1.2, 1.2) * agg;
                }
                self.target = self.target.clamp(lo, hi);
                if self.st % 3 == 0 {
                    fx.sparks(self.target + rng.s() * 8.0, self.floor, 1, -1.57);
                }
                if self.st > 70 {
                    self.x = self.target;
                    self.state = 5;
                    self.st = 0;
                    out.play(Sfx::Whip);
                    fx.shake = 4.0;
                    for _ in 0..12 {
                        fx.sparks(self.x, self.y, 1, -1.57 + rng.s() * 0.6);
                    }
                }
            }
            // Out with the shovel high.
            5 => {
                if self.st > 50 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            // Calling the dead up at the arena's edges.
            6 => {
                if self.st == 20 {
                    out.play(Sfx::Rise);
                    for x in [lo + 4.0, hi - 4.0] {
                        let mut z = Foe::new(Kind::Zombie, x, self.floor);
                        z.f = if x < self.x { 1.0 } else { -1.0 };
                        foes.push(z);
                    }
                }
                if self.st > 60 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn bog(
        &mut self,
        px: f32,
        py: f32,
        shots: &mut Vec<EShot>,
        fx: &mut Fx,
        out: &mut Out,
        rng: &mut Rng,
        agg: f32,
    ) {
        match self.state {
            // Under the water, choosing where to rise: bubbles there.
            0 => {
                if self.st == 1 {
                    // Not right under NIRUX, and not where it was.
                    let mut best = self.spots[0];
                    let mut bd = -1.0;
                    for &s in &self.spots {
                        let d = (s - px).abs() + rng.f() * 40.0
                            - if (s - self.x).abs() < 1.0 { 80.0 } else { 0.0 };
                        if d > bd && (s - px).abs() > 30.0 {
                            bd = d;
                            best = s;
                        }
                    }
                    self.target = best;
                }
                if self.st % 5 == 0 {
                    fx.sparks(self.target + rng.s() * 10.0, self.floor - 2.0, 1, -1.57);
                }
                if self.st > (60.0 / agg) as u32 {
                    self.x = self.target;
                    self.state = 1;
                    self.st = 0;
                    out.play(Sfx::Splash);
                }
            }
            // Rising.
            1 => {
                if self.st > 30 {
                    self.state = 2;
                    self.st = 0;
                    self.shots_left = 0;
                }
            }
            // Up: spit volleys, then a sweep of the arm over the water.
            2 => {
                self.f = (px - self.x).signum();
                let every = (48.0 / agg) as u32;
                if self.st % every == every / 2 && self.st < 125 {
                    let n = if agg > 1.2 { 5 } else { 3 };
                    let a0 = (py - 14.0 - (self.floor - 50.0)).atan2(px - self.x);
                    for k in 0..n {
                        let a = a0 + (k as f32 - (n - 1) as f32 / 2.0) * 0.22;
                        let sp = 1.5 * agg;
                        shots.push(EShot {
                            x: self.x + self.f * 8.0,
                            y: self.floor - 48.0,
                            vx: a.cos() * sp,
                            vy: a.sin() * sp,
                            g: 0.0,
                            kind: EKind::Spit,
                            life: 240,
                        });
                    }
                    out.play(Sfx::EnemyShot);
                }
                if self.st == 128 {
                    out.play(Sfx::Alarm);
                }
                if self.st == 140 {
                    out.play(Sfx::Whip);
                    fx.shake = 3.0;
                }
                if self.st > 200 {
                    self.state = 3;
                    self.st = 0;
                }
            }
            // Sinking back.
            _ => {
                if self.st > 30 {
                    self.state = 0;
                    self.st = 0;
                    out.play(Sfx::Splash);
                }
            }
        }
    }

    pub fn draw(&self, fb: &mut Fb, ox: i32, oy: i32, t: u32) {
        let fl = self.flash > 0 && self.flash % 2 == 0 || (self.dying > 0 && self.dying % 6 < 3);
        let c = |c: u8| if fl { WHITE } else { c };
        match self.kind {
            BossKind::Gravedigger => {
                // Where it will come up: the earth heaves.
                if self.state == 4 {
                    let x = self.target as i32 + ox;
                    let y = self.floor as i32 + oy;
                    let h = (self.st / 10) as i32 + 1;
                    for k in -6i32..=6 {
                        let hh = h - k.abs() / 2;
                        if hh > 0 {
                            fb.line(x + k, y, x + k, y - hh, BROWN);
                        }
                    }
                    return;
                }
                let sink = match self.state {
                    0 => (80 - self.st.min(80) as i32) * 46 / 80,
                    3 => self.st.min(40) as i32 * 46 / 40,
                    5 if self.st < 16 => (16 - self.st as i32) * 3,
                    _ => 0,
                };
                let x = self.x as i32 + ox;
                let y = self.y as i32 + oy;
                let f = if self.f < 0.0 { -1 } else { 1 };
                let ground = y;
                let put = |fb: &mut Fb, px: i32, py: i32, col: u8| {
                    if py + sink < ground {
                        fb.set(px, py + sink, col);
                    }
                };
                let walk = if self.state == 1 {
                    ((self.t / 10) % 2) as i32
                } else {
                    0
                };
                // Boots and legs.
                for yy in 0..10 {
                    put(fb, x - 5 - walk, y - 1 - yy, c(BLACK));
                    put(fb, x - 4 - walk, y - 1 - yy, c(DGREY));
                    put(fb, x + 4 + walk, y - 1 - yy, c(BLACK));
                    put(fb, x + 5 + walk, y - 1 - yy, c(DGREY));
                }
                // The long coat, hunched.
                for yy in 8..36 {
                    let w = 9 + (36 - yy) / 6;
                    for xx in -w..=w {
                        let lean = (yy - 8) / 7 * f;
                        let col = if xx * f < -w + 3 {
                            BLACK
                        } else if (xx + yy) % 7 == 0 {
                            BROWN
                        } else {
                            DGREY
                        };
                        put(fb, x + xx + lean, y - yy, c(col));
                    }
                }
                // A hanging lantern.
                let lx = x - f * 10;
                put(fb, lx, y - 22, c(BLACK));
                for yy in 0..4 {
                    for xx in -1..=1 {
                        put(
                            fb,
                            lx + xx,
                            y - 18 + yy,
                            if t % 30 < 26 { YELLOW } else { ORANGE },
                        );
                    }
                }
                // The head: a pale face under a wide black hat.
                let hx = x + f * 6;
                for yy in 0..7 {
                    for xx in -3..=3 {
                        put(fb, hx + xx, y - 42 + yy, c(LGREY));
                    }
                }
                put(fb, hx + f, y - 39, c(YELLOW));
                put(fb, hx + f * 3, y - 39, c(YELLOW));
                put(fb, hx + f * 2, y - 37, c(BLACK));
                for xx in -9..=9 {
                    put(fb, hx + xx, y - 43, c(BLACK));
                    put(fb, hx + xx, y - 44, c(BLACK));
                }
                for yy in 45..51 {
                    for xx in -4..=4 {
                        put(fb, hx + xx, y - yy, c(BLACK));
                    }
                }
                // The shovel: over the shoulder, thrust forward when it
                // flings, raised high when it strikes from below.
                let (sx0, sy0, sx1, sy1) = match self.state {
                    2 if self.st % 16 < 8 => (x, y - 26, x + f * 22, y - 40),
                    5 => (x + f * 2, y - 20, x + f * 6, y - 58),
                    _ => (x - f * 4, y - 20, x + f * 10, y - 44),
                };
                let n = 24;
                for k in 0..=n {
                    let px = sx0 + (sx1 - sx0) * k / n;
                    let py = sy0 + (sy1 - sy0) * k / n;
                    put(fb, px, py, c(BROWN));
                }
                for yy in -3..=3 {
                    for xx in -2..=2 {
                        put(
                            fb,
                            sx1 + xx,
                            sy1 + yy,
                            c(if xx == -2 { LGREY } else { GREY }),
                        );
                    }
                }
            }
            BossKind::BogDaemon => {
                let x = self.x as i32 + ox;
                let wy = self.floor as i32 + oy;
                if self.state == 0 {
                    return;
                }
                let up = match self.state {
                    1 => self.st.min(30) as i32 * 60 / 30,
                    3 => (30 - self.st.min(30) as i32) * 60 / 30,
                    _ => 60 + ((self.t as f32 * 0.08).sin() * 2.0) as i32,
                };
                let top = wy - up;
                let f = if self.f < 0.0 { -1 } else { 1 };
                // The body, rising out of the water, dripping.
                for yy in 0..up {
                    let y = top + yy;
                    let w = (12 + yy / 3).min(24);
                    for xx in -w..=w {
                        let col = if xx.abs() > w - 2 {
                            BLACK
                        } else if (xx * 3 + yy * 5 + (self.t / 4) as i32) % 13 == 0 {
                            LGREEN
                        } else if xx < -w / 2 {
                            BLACK
                        } else {
                            GREEN
                        };
                        if y < wy {
                            fb.set(x + xx, y, c(col));
                        }
                    }
                }
                // Horns, eyes and a wide mouth.
                for k in 0..8 {
                    fb.set(x - 10 - k / 2, top + 2 - k, c(LGREY));
                    fb.set(x + 10 + k / 2, top + 2 - k, c(LGREY));
                }
                let eye = if self.state == 2 { YELLOW } else { ORANGE };
                fb.rect(x - 7, top + 8, 4, 3, c(eye));
                fb.rect(x + 4, top + 8, 4, 3, c(eye));
                fb.set(x - 6 + f, top + 9, BLACK);
                fb.set(x + 5 + f, top + 9, BLACK);
                let open = self.state == 2 && self.st % (48.0 / self.agg) as u32 > 16;
                fb.rect(x - 8, top + 16, 17, if open { 6 } else { 2 }, c(BLACK));
                if open {
                    fb.hline(x - 7, x + 7, top + 17, LRED);
                    for k in 0..4 {
                        fb.set(x - 6 + k * 4, top + 16, WHITE);
                    }
                }
                // Drips running down.
                for k in 0..5 {
                    let dx = -14 + k * 7;
                    let dy = ((self.t + k as u32 * 13) % 30) as i32;
                    if top + 20 + dy < wy {
                        fb.set(x + dx, top + 20 + dy, LGREEN);
                    }
                }
                // The arm sweeping over the water.
                if self.state == 2 && (128..170).contains(&self.st) {
                    let reach = if self.st < 140 {
                        0
                    } else {
                        ((self.st - 140) * 9) as i32
                    };
                    let ay = wy - 18;
                    let ax0 = x + f * 18;
                    let ax1 = ax0 + f * reach;
                    if self.st < 140 && self.st % 4 < 2 {
                        fb.disc(ax0, ay, 3, LRED);
                    }
                    fb.line(ax0, ay, ax1, ay, c(GREEN));
                    fb.line(ax0, ay + 1, ax1, ay + 1, c(BLACK));
                    fb.line(ax0, ay - 1, ax1, ay - 1, c(LGREEN));
                    for k in 0..3 {
                        fb.line(ax1, ay, ax1 + f * 5, ay - 4 + k * 4, c(WHITE));
                    }
                }
                // The water closes round it.
                for xx in -26..=26 {
                    if (xx + (self.t / 3) as i32) % 4 < 2 {
                        fb.set(x + xx, wy - 1, LGREEN);
                    }
                }
            }
        }
    }
}
