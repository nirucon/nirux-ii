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
    Choir,
    ClockWarden,
    FrostWyrm,
    DaemonLord,
    Null,
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
    vx: f32,
    vy: f32,
    /// Where the Frost Wyrm's body has been, head first.
    trail: Vec<(f32, f32)>,
    /// The arc's start and end for the Frost Wyrm.
    arc: (f32, f32),
    cycles: u32,
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
            (BossKind::Choir, Difficulty::Easy) => 55,
            (BossKind::Choir, Difficulty::Normal) => 75,
            (BossKind::Choir, Difficulty::Hard) => 95,
            (BossKind::ClockWarden, Difficulty::Easy) => 65,
            (BossKind::ClockWarden, Difficulty::Normal) => 85,
            (BossKind::ClockWarden, Difficulty::Hard) => 105,
            (BossKind::FrostWyrm, Difficulty::Easy) => 65,
            (BossKind::FrostWyrm, Difficulty::Normal) => 85,
            (BossKind::FrostWyrm, Difficulty::Hard) => 105,
            (BossKind::DaemonLord, Difficulty::Easy) => 75,
            (BossKind::DaemonLord, Difficulty::Normal) => 95,
            (BossKind::DaemonLord, Difficulty::Hard) => 120,
            (BossKind::Null, Difficulty::Easy) => 110,
            (BossKind::Null, Difficulty::Normal) => 140,
            (BossKind::Null, Difficulty::Hard) => 170,
        };
        let (x, y) = match kind {
            BossKind::Choir => (arena + 160.0, floor - 90.0),
            BossKind::DaemonLord => (arena + 220.0, floor - 110.0),
            BossKind::Null => (arena + 230.0, floor - 64.0),
            BossKind::FrostWyrm => (arena + 160.0, floor),
            _ => (arena + 230.0, floor),
        };
        Boss {
            kind,
            x,
            y,
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
            vx: 0.0,
            vy: 0.0,
            trail: Vec::new(),
            arc: (arena + 40.0, arena + 280.0),
            cycles: 0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self.kind {
            BossKind::Gravedigger => "THE GRAVEDIGGER",
            BossKind::BogDaemon => "THE BOG DAEMON",
            BossKind::Choir => "THE CHOIR",
            BossKind::ClockWarden => "THE CLOCKWORK WARDEN",
            BossKind::FrostWyrm => "THE FROST WYRM",
            BossKind::DaemonLord => "THE DAEMON LORD",
            BossKind::Null => "NULL",
        }
    }

    pub fn score(&self) -> u32 {
        match self.kind {
            BossKind::Gravedigger => 10000,
            BossKind::BogDaemon => 15000,
            BossKind::Choir => 20000,
            BossKind::ClockWarden => 25000,
            BossKind::FrostWyrm => 30000,
            BossKind::DaemonLord => 40000,
            BossKind::Null => 100000,
        }
    }

    pub fn hp(&self) -> (i32, i32) {
        (self.hp, self.max)
    }

    pub fn center(&self) -> (f32, f32) {
        match self.kind {
            BossKind::Gravedigger => (self.x, self.y - 24.0),
            BossKind::BogDaemon => (self.x, self.floor - 40.0),
            BossKind::ClockWarden => (self.x, self.y - 26.0),
            _ => (self.x, self.y),
        }
    }

    fn visible(&self) -> bool {
        match self.kind {
            BossKind::Gravedigger => !matches!(self.state, 4),
            BossKind::BogDaemon => matches!(self.state, 1..=3),
            BossKind::FrostWyrm => self.state != 0,
            BossKind::Null => self.state != 3,
            _ => true,
        }
    }

    fn vulnerable(&self) -> bool {
        self.dying == 0
            && match self.kind {
                BossKind::Gravedigger => matches!(self.state, 1 | 2 | 5 | 6),
                BossKind::BogDaemon => self.state == 2,
                BossKind::FrostWyrm => self.state == 1 || self.state == 2,
                BossKind::Null => self.state == 1,
                _ => self.state >= 1,
            }
    }

    pub fn hurtbox_hit(&self, x: f32, y: f32, r: f32) -> bool {
        if !self.vulnerable() {
            return false;
        }
        if self.kind == BossKind::Choir {
            return self
                .heads()
                .iter()
                .any(|&(hx, hy)| (x - hx).abs() < 9.0 + r && (y - hy).abs() < 9.0 + r);
        }
        let (cx, cy, hw, hh) = match self.kind {
            BossKind::Gravedigger => (self.x, self.y - 22.0, 12.0, 22.0),
            BossKind::BogDaemon => (self.x, self.floor - 38.0, 17.0, 22.0),
            BossKind::ClockWarden => (self.x, self.y - 26.0, 14.0, 26.0),
            BossKind::FrostWyrm => (self.x, self.y, 10.0, 10.0),
            BossKind::DaemonLord => (self.x, self.y, 15.0, 13.0),
            _ => (self.x, self.y, 16.0, 20.0),
        };
        (x - cx).abs() < hw + r && (y - cy).abs() < hh + r
    }

    /// The Choir's singing heads that are left (one for each third).
    fn heads(&self) -> Vec<(f32, f32)> {
        let n = ((self.hp.max(0) * 3 + self.max - 1) / self.max.max(1)).clamp(0, 3);
        (0..n)
            .map(|k| {
                let a = self.t as f32 * 0.012 * self.agg + k as f32 * std::f32::consts::TAU / 3.0;
                (self.x + a.cos() * 56.0, self.y + a.sin() * 30.0)
            })
            .collect()
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
            BossKind::Choir => self
                .heads()
                .iter()
                .any(|&(hx, hy)| (px - hx).abs() < 10.0 && hy > py - ph - 6.0 && hy < py + 4.0),
            BossKind::ClockWarden => {
                (px - self.x).abs() < 14.0 && py > self.y - 50.0 && py - ph < self.y
            }
            BossKind::FrostWyrm => {
                let hit = |x: f32, y: f32, r: f32| {
                    (px - x).abs() < r + 3.0 && y > py - ph - r && y < py + r
                };
                hit(self.x, self.y, 9.0)
                    || self.trail.iter().step_by(4).any(|&(x, y)| hit(x, y, 6.0))
            }
            BossKind::DaemonLord => {
                (px - self.x).abs() < 16.0 && self.y + 12.0 > py - ph && self.y - 12.0 < py
            }
            BossKind::Null => {
                (px - self.x).abs() < 16.0 && self.y + 22.0 > py - ph && self.y - 22.0 < py
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
            BossKind::FrostWyrm => {
                self.state == 0 && self.st > 30 && (self.target - x).abs() < 24.0
            }
            BossKind::ClockWarden => self.state == 3 && self.vy > 0.0 && (self.x - x).abs() < 50.0,
            BossKind::DaemonLord => self.state == 3 && (self.x - x).abs() < 40.0,
            _ => false,
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
            return self.dying
                > if self.kind == BossKind::Null {
                    220
                } else {
                    140
                };
        }
        let rage = if self.hp * 2 < self.max { 1.35 } else { 1.0 };
        let agg = self.agg * rage;
        match self.kind {
            BossKind::Gravedigger => self.gravedigger(px, py, shots, foes, fx, out, rng, agg),
            BossKind::BogDaemon => self.bog(px, py, shots, fx, out, rng, agg),
            BossKind::Choir => self.choir(px, py, shots, out, agg),
            BossKind::ClockWarden => self.clock(px, shots, fx, out, rng, agg),
            BossKind::FrostWyrm => self.wyrm(px, py, shots, fx, out, rng, agg),
            BossKind::DaemonLord => self.lord(px, py, shots, foes, fx, out, rng, agg),
            BossKind::Null => self.null(px, py, shots, fx, out, rng, agg),
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

    fn aimed(&self, from: (f32, f32), to: (f32, f32), sp: f32, kind: EKind) -> EShot {
        let a = (to.1 - from.1).atan2(to.0 - from.0);
        EShot {
            x: from.0,
            y: from.1,
            vx: a.cos() * sp,
            vy: a.sin() * sp,
            g: 0.0,
            kind,
            life: 300,
        }
    }

    /// The Choir of the Deleted: three heads singing in a slow ring, notes
    /// aimed at NIRUX, and now and then a chorus all round.
    fn choir(&mut self, px: f32, py: f32, shots: &mut Vec<EShot>, out: &mut Out, agg: f32) {
        if self.state == 0 {
            if self.st > 60 {
                self.state = 1;
            }
            return;
        }
        self.x = self.arena + 160.0 + (self.t as f32 * 0.006).sin() * 70.0;
        self.y = self.floor - 92.0 + (self.t as f32 * 0.02).sin() * 8.0;
        let every = (110.0 / agg) as u32;
        for (k, &(hx, hy)) in self.heads().iter().enumerate() {
            if (self.t + k as u32 * every / 3) % every == 0 {
                shots.push(self.aimed((hx, hy), (px, py - 12.0), 1.25 * agg, EKind::Note));
                out.play(Sfx::EnemyShot);
            }
        }
        if self.t % 420 == 300 {
            for k in 0..10 {
                let a = k as f32 * std::f32::consts::TAU / 10.0;
                shots.push(EShot {
                    x: self.x,
                    y: self.y,
                    vx: a.cos() * 1.0,
                    vy: a.sin() * 1.0,
                    g: 0.0,
                    kind: EKind::Note,
                    life: 300,
                });
            }
            out.play(Sfx::Bell);
        }
    }

    /// The Clockwork Warden: walks at NIRUX, rolls gears along the floor
    /// (jump them), leaps and slams a shock wave out both ways.
    fn clock(
        &mut self,
        px: f32,
        shots: &mut Vec<EShot>,
        fx: &mut Fx,
        out: &mut Out,
        rng: &mut Rng,
        agg: f32,
    ) {
        let (lo, hi) = (self.arena + 24.0, self.arena + 296.0);
        match self.state {
            0 => {
                if self.st > 60 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            1 => {
                self.f = (px - self.x).signum();
                if (px - self.x).abs() > 40.0 {
                    self.x += self.f * 0.5 * agg;
                }
                self.x = self.x.clamp(lo, hi);
                if self.st > (120.0 / agg) as u32 {
                    self.st = 0;
                    self.state = if rng.chance(0.55) { 2 } else { 3 };
                    if self.state == 3 {
                        self.vy = -5.0;
                        self.vx = ((px - self.x) / 50.0).clamp(-2.5, 2.5);
                    }
                    self.shots_left = if agg > 1.2 { 3 } else { 2 };
                }
            }
            2 => {
                if self.st % 30 == 10 && self.shots_left > 0 {
                    self.shots_left -= 1;
                    shots.push(EShot {
                        x: self.x + self.f * 14.0,
                        y: self.floor - 5.0,
                        vx: self.f * 1.8 * agg,
                        vy: 0.0,
                        g: 0.0,
                        kind: EKind::Gear,
                        life: 260,
                    });
                    out.play(Sfx::Clang);
                }
                if self.shots_left == 0 && self.st > 50 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            _ => {
                self.x = (self.x + self.vx).clamp(lo, hi);
                self.vy += 0.2;
                self.y += self.vy;
                if self.y >= self.floor {
                    self.y = self.floor;
                    fx.shake = 5.0;
                    out.play(Sfx::ExplodeSmall);
                    for d in [-1.0, 1.0] {
                        shots.push(EShot {
                            x: self.x + d * 16.0,
                            y: self.floor - 3.0,
                            vx: d * 2.2,
                            vy: 0.0,
                            g: 0.0,
                            kind: EKind::Wave,
                            life: 150,
                        });
                    }
                    self.state = 1;
                    self.st = 0;
                }
            }
        }
    }

    /// The Frost Wyrm: cracks show where it will break out, then it arcs
    /// over the arena and dives; every other time it rests at an edge and
    /// breathes ice.
    #[allow(clippy::too_many_arguments)]
    fn wyrm(
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
            0 => {
                if self.st == 1 {
                    self.cycles += 1;
                    self.target = if self.cycles % 3 == 0 {
                        if px > self.arena + 160.0 {
                            self.arena + 34.0
                        } else {
                            self.arena + 286.0
                        }
                    } else {
                        (px + rng.s() * 30.0).clamp(self.arena + 30.0, self.arena + 290.0)
                    };
                    self.trail.clear();
                }
                if self.st % 4 == 0 {
                    fx.sparks(self.target + rng.s() * 8.0, self.floor, 1, -1.57);
                }
                if self.st > (70.0 / agg) as u32 {
                    self.st = 0;
                    out.play(Sfx::Crumble);
                    fx.shake = 3.0;
                    if self.cycles % 3 == 0 {
                        self.state = 2;
                        self.x = self.target;
                        self.y = self.floor - 40.0;
                    } else {
                        self.state = 1;
                        let to = if self.target < self.arena + 160.0 {
                            self.target + 150.0
                        } else {
                            self.target - 150.0
                        };
                        self.arc = (self.target, to);
                    }
                }
            }
            1 => {
                let dur = (110.0 / agg) as u32;
                let u = self.st as f32 / dur as f32;
                self.x = self.arc.0 + (self.arc.1 - self.arc.0) * u;
                self.y = self.floor - (u * std::f32::consts::PI).sin() * 105.0;
                self.trail.insert(0, (self.x, self.y));
                self.trail.truncate(40);
                if self.st >= dur {
                    self.state = 0;
                    self.st = 0;
                    fx.shake = 3.0;
                    out.play(Sfx::Crumble);
                }
            }
            _ => {
                // Resting at the edge, the head up, breathing shards.
                self.f = (px - self.x).signum();
                if self.st % (40.0 / agg) as u32 == 20 {
                    for k in -1..=1 {
                        let mut sh =
                            self.aimed((self.x, self.y), (px, py - 10.0), 1.8 * agg, EKind::Shard);
                        let (c, sn) = ((k as f32) * 0.2f32).sin_cos();
                        let (vx, vy) = (sh.vx * sn - sh.vy * c, sh.vx * c + sh.vy * sn);
                        sh.vx = vx;
                        sh.vy = vy;
                        shots.push(sh);
                    }
                    out.play(Sfx::EnemyShot);
                }
                if self.st > 170 {
                    self.state = 0;
                    self.st = 0;
                }
            }
        }
    }

    /// The Daemon Lord: hovers on great wings, rains fire, dives, and calls
    /// its daemons when it is hurt.
    #[allow(clippy::too_many_arguments)]
    fn lord(
        &mut self,
        px: f32,
        py: f32,
        shots: &mut Vec<EShot>,
        foes: &mut Vec<Foe>,
        fx: &mut Fx,
        out: &mut Out,
        rng: &mut Rng,
        agg: f32,
    ) {
        match self.state {
            0 => {
                if self.st > 60 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            1 => {
                let tx = (px + (self.t as f32 * 0.015).sin() * 80.0)
                    .clamp(self.arena + 30.0, self.arena + 290.0);
                self.x += (tx - self.x) * 0.02;
                self.y +=
                    (self.floor - 112.0 + (self.t as f32 * 0.04).sin() * 10.0 - self.y) * 0.05;
                self.f = (px - self.x).signum();
                if self.st > (140.0 / agg) as u32 {
                    self.st = 0;
                    let r = rng.f();
                    self.state = if self.hp * 2 < self.max && r < 0.25 && foes.len() < 3 {
                        4
                    } else if r < 0.6 {
                        2
                    } else {
                        3
                    };
                    if self.state == 3 {
                        self.target = px;
                    }
                }
            }
            2 => {
                // Fire rain over where NIRUX stands.
                if self.st % 12 == 0 && self.st < 72 {
                    let x = px + rng.s() * 70.0;
                    shots.push(EShot {
                        x,
                        y: self.floor - 170.0,
                        vx: 0.0,
                        vy: 0.6,
                        g: 0.05 * agg,
                        kind: EKind::Fire,
                        life: 300,
                    });
                    out.play(Sfx::EnemyShot);
                }
                if self.st > 110 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            3 => {
                // A dive at where NIRUX was, then up again.
                let k = self.st as f32 / 70.0;
                if k < 1.0 {
                    self.x += (self.target - self.x) * 0.06;
                    self.y += (self.floor - 22.0 - self.y) * 0.08;
                } else {
                    self.y += (self.floor - 112.0 - self.y) * 0.05;
                }
                if self.st == 40 {
                    out.play(Sfx::Whip);
                    fx.shake = 2.0;
                }
                if self.st > 130 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            _ => {
                if self.st == 20 {
                    out.play(Sfx::Alarm);
                    for d in [-1.0, 1.0] {
                        let mut e = Foe::new(Kind::Daemon, self.x + d * 30.0, self.y);
                        e.cd = 90;
                        foes.push(e);
                    }
                }
                if self.st > 60 {
                    self.state = 1;
                    self.st = 0;
                }
            }
        }
        let _ = py;
    }

    /// NULL: never whole. A mass of static with red eyes that fires bursts
    /// and flickers from place to place; then it splits and fires all
    /// round; at the end it sends sweeps of static across the floor.
    #[allow(clippy::too_many_arguments)]
    fn null(
        &mut self,
        px: f32,
        py: f32,
        shots: &mut Vec<EShot>,
        fx: &mut Fx,
        out: &mut Out,
        rng: &mut Rng,
        agg: f32,
    ) {
        let phase = if self.hp * 3 > self.max * 2 {
            1
        } else if self.hp * 3 > self.max {
            2
        } else {
            3
        };
        match self.state {
            0 => {
                if self.st > 90 {
                    self.state = 1;
                    self.st = 0;
                }
            }
            1 => {
                self.y += (self.target_y() - self.y) * 0.04;
                self.x += (self.spots[self.shots_left as usize % 3] - self.x) * 0.03;
                self.f = (px - self.x).signum();
                let every = (80.0 / agg) as u32;
                let k = self.st % every;
                if k == 0 || k == 8 || k == 16 {
                    shots.push(self.aimed(
                        (self.x, self.y),
                        (px, py - 12.0),
                        1.7 * agg,
                        EKind::Static,
                    ));
                    out.play(Sfx::EnemyShot);
                }
                if phase >= 2 && self.st % 150 == 75 {
                    for n in 0..8 {
                        let a = n as f32 * std::f32::consts::FRAC_PI_4 + self.t as f32 * 0.01;
                        shots.push(EShot {
                            x: self.x + a.cos() * 26.0,
                            y: self.y + a.sin() * 26.0,
                            vx: a.cos() * 1.2,
                            vy: a.sin() * 1.2,
                            g: 0.0,
                            kind: EKind::Static,
                            life: 260,
                        });
                    }
                    out.play(Sfx::Thunder);
                }
                if phase == 3 && self.st % 200 == 100 {
                    // A sweep of static: high (duck) or low (jump).
                    let high = (self.t / 200) % 2 == 0;
                    let y = if high {
                        self.floor - 17.0
                    } else {
                        self.floor - 3.0
                    };
                    let from_left = px > self.arena + 160.0;
                    for n in 0..5 {
                        let x = if from_left {
                            self.arena - 10.0 - n as f32 * 9.0
                        } else {
                            self.arena + 330.0 + n as f32 * 9.0
                        };
                        shots.push(EShot {
                            x,
                            y,
                            vx: if from_left { 2.3 } else { -2.3 },
                            vy: 0.0,
                            g: 0.0,
                            kind: EKind::Wave,
                            life: 260,
                        });
                    }
                    out.play(Sfx::Alarm);
                }
                if self.st > 260 {
                    self.state = 3;
                    self.st = 0;
                }
            }
            _ => {
                // Gone into the static, and back somewhere else.
                if self.st == 1 {
                    out.play(Sfx::Thunder);
                    fx.flash = 2;
                }
                if self.st == 30 {
                    self.shots_left = (self.shots_left + 1 + rng.below(2) as u8) % 3;
                    self.x = self.spots[self.shots_left as usize];
                    self.y = self.target_y();
                }
                if self.st > 50 {
                    self.state = 1;
                    self.st = 0;
                }
            }
        }
    }

    fn target_y(&self) -> f32 {
        self.floor - 50.0 - (self.shots_left as f32) * 14.0
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
            BossKind::Choir => {
                // Pale singing heads, mouths open, the notes rising.
                let fade = if self.state == 0 {
                    (self.st / 4) as i32
                } else {
                    16
                };
                for (k, (hx, hy)) in self.heads().iter().enumerate() {
                    let (x, y) = (*hx as i32 + ox, *hy as i32 + oy);
                    for yy in -8..=8 {
                        for xx in -7..=7 {
                            if xx * xx * 64 + yy * yy * 49 > 49 * 64 {
                                continue;
                            }
                            if !super::world::dith(x + xx, y + yy, fade) {
                                continue;
                            }
                            let col = if yy > 4 && (xx + (t / 6) as i32) % 3 == 0 {
                                CLEAR
                            } else if xx < -4 {
                                GREY
                            } else {
                                LGREY
                            };
                            if col != CLEAR {
                                fb.set(x + xx, y + yy, c(col));
                            }
                        }
                    }
                    fb.rect(x - 4, y - 3, 3, 3, BLACK);
                    fb.rect(x + 2, y - 3, 3, 3, BLACK);
                    let open = ((t / 10) as usize + k) % 3;
                    fb.rect(x - 2, y + 2, 5, 1 + open as i32 * 2, BLACK);
                    fb.set(x - 3, y - 2, if t % 30 < 15 { LRED } else { PURPLE });
                    fb.set(x + 3, y - 2, if t % 30 < 15 { LRED } else { PURPLE });
                }
            }
            BossKind::ClockWarden => {
                let (x, y) = (self.x as i32 + ox, self.y as i32 + oy);
                let f = if self.f < 0.0 { -1 } else { 1 };
                let step = if self.state == 1 {
                    ((self.t / 12) % 2) as i32
                } else {
                    0
                };
                // Legs of iron.
                fb.rect(x - 9 - step, y - 14, 5, 14, c(DGREY));
                fb.rect(x + 4 + step, y - 14, 5, 14, c(GREY));
                // The case, with the clock face on its chest.
                fb.rect(x - 13, y - 50, 26, 36, c(BROWN));
                fb.frame(x - 13, y - 50, 26, 36, c(ORANGE));
                fb.disc(x, y - 36, 10, c(YELLOW));
                fb.circle(x, y - 36, 10, c(BROWN));
                for k in 0..12 {
                    let a = k as f32 * std::f32::consts::TAU / 12.0;
                    fb.set(
                        x + (a.cos() * 8.0) as i32,
                        y - 36 + (a.sin() * 8.0) as i32,
                        BLACK,
                    );
                }
                let (ha, ma) = (t as f32 * 0.01, t as f32 * 0.12);
                fb.line(
                    x,
                    y - 36,
                    x + (ha.sin() * 5.0) as i32,
                    y - 36 - (ha.cos() * 5.0) as i32,
                    BLACK,
                );
                fb.line(
                    x,
                    y - 36,
                    x + (ma.sin() * 8.0) as i32,
                    y - 36 - (ma.cos() * 8.0) as i32,
                    RED,
                );
                // The pendulum swinging inside, the head with one red eye.
                let sw = ((t as f32 * 0.1).sin() * 5.0) as i32;
                fb.line(x, y - 25, x + sw, y - 17, GREY);
                fb.disc(x + sw, y - 17, 2, c(YELLOW));
                fb.rect(x - 7, y - 58, 14, 8, c(DGREY));
                fb.rect(
                    x + f * 2 - 1,
                    y - 56,
                    3,
                    3,
                    if t % 20 < 14 { LRED } else { RED },
                );
                // Arms that hold gears.
                fb.line(x - 13, y - 46, x - 18, y - 30, c(GREY));
                fb.line(x + 13, y - 46, x + 18, y - 30, c(GREY));
                fb.circle(x + f * 18, y - 28, 4, c(ORANGE));
            }
            BossKind::FrostWyrm => {
                if self.state == 0 {
                    // Cracks in the ice where it will come.
                    let (x, y) = (self.target as i32 + ox, self.floor as i32 + oy);
                    let n = (self.st / 6) as i32;
                    for k in 0..n.min(10) {
                        fb.set(x - k, y - 1 + (k % 2), WHITE);
                        fb.set(x + k, y - 1 + ((k + 1) % 2), WHITE);
                    }
                    return;
                }
                // The body, tail first, scales light and dark.
                for (k, &(bx, by)) in self.trail.iter().enumerate().rev() {
                    if k % 2 == 1 {
                        continue;
                    }
                    let r = (7 - k as i32 / 7).max(2);
                    let (x, y) = (bx as i32 + ox, by as i32 + oy);
                    if by < self.floor + 2.0 {
                        fb.disc(x, y, r, c(if k % 4 == 0 { LBLUE } else { BLUE }));
                        fb.set(x, y - r, WHITE);
                    }
                }
                // The head, jaws and frost.
                let (x, y) = (self.x as i32 + ox, self.y as i32 + oy);
                let f = if self.state == 1 {
                    if self.arc.1 > self.arc.0 {
                        1
                    } else {
                        -1
                    }
                } else if self.f < 0.0 {
                    -1
                } else {
                    1
                };
                fb.disc(x, y, 9, c(LBLUE));
                fb.disc(x + f * 6, y + 2, 6, c(CYAN));
                fb.hline(x + f * 2, x + f * 12, y + 3, BLACK);
                fb.set(x + f * 3, y - 4, YELLOW);
                fb.set(x + f * 4, y - 4, YELLOW);
                for k in 0..4 {
                    fb.line(
                        x - f * (2 + k * 2),
                        y - 8,
                        x - f * (5 + k * 3),
                        y - 14 + k,
                        WHITE,
                    );
                }
                if self.state == 2 {
                    // Its neck down into the ice.
                    for k in 0..5 {
                        fb.disc(x - f * k, y + 10 + k * 6, 6, c(BLUE));
                    }
                }
            }
            BossKind::DaemonLord => {
                let (x, y) = (self.x as i32 + ox, self.y as i32 + oy);
                let flap = ((self.t / 6) % 4) as i32;
                let wy = [-10, -4, 4, -4][flap as usize];
                for side in [-1, 1] {
                    for k in 0..4 {
                        let tip = (x + side * (14 + k * 9), y - 8 + wy + k * 2);
                        fb.line(x + side * 6, y - 6, tip.0, tip.1, c(RED));
                        fb.line(tip.0, tip.1, x + side * (10 + k * 7), y + 6, c(BROWN));
                    }
                }
                fb.disc(x, y, 10, c(LRED));
                fb.disc(x, y + 6, 8, c(RED));
                fb.disc(x, y - 12, 6, c(LRED));
                for side in [-1, 1] {
                    fb.line(x + side * 4, y - 16, x + side * 9, y - 26, c(LGREY));
                    fb.line(x + side * 9, y - 26, x + side * 7, y - 28, c(WHITE));
                }
                fb.set(x - 2, y - 13, YELLOW);
                fb.set(x + 2, y - 13, YELLOW);
                fb.hline(x - 3, x + 3, y - 9, BLACK);
                fb.disc(x, y - 1, 2, if t % 20 < 10 { YELLOW } else { ORANGE });
            }
            BossKind::Null => {
                // Never whole: a jagged mass of static, two red eyes in it.
                let (x, y) = (self.x as i32 + ox, self.y as i32 + oy);
                let gone = if self.state == 3 {
                    (self.st.min(30) as i32) / 2
                } else {
                    0
                };
                let back = if self.state == 3 && self.st > 30 {
                    (self.st as i32 - 30) / 2
                } else {
                    16
                };
                let dissolve = if self.dying > 0 {
                    (self.dying as i32) / 12
                } else {
                    0
                };
                let mut r = self.t.wrapping_mul(2_654_435_761);
                for yy in -26..=26 {
                    r = r.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                    let jag = (r >> 28) as i32 - 8;
                    let w = ((26 * 26 - yy * yy) as f32).sqrt() as i32 * 2 / 3 + jag / 3;
                    for xx in -w..=w {
                        r = r.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                        let k = (r >> 24) & 15;
                        let keep = (k as i32) >= gone + dissolve && (k as i32) < back.max(1);
                        if !keep {
                            continue;
                        }
                        let col = match (r >> 20) & 7 {
                            0 => WHITE,
                            1 | 2 => GREY,
                            3 => {
                                if t % 9 == 0 {
                                    RED
                                } else {
                                    DGREY
                                }
                            }
                            _ => {
                                if (xx + yy) % 2 == 0 {
                                    BLACK
                                } else {
                                    DGREY
                                }
                            }
                        };
                        fb.set(x + xx, y + yy, if fl { WHITE } else { col });
                    }
                }
                if self.state != 3 && self.dying < 120 {
                    let f = if self.f < 0.0 { -1 } else { 1 };
                    for side in [-1, 1] {
                        fb.rect(x + side * 7 + f * 2 - 1, y - 8, 4, 2, LRED);
                        fb.set(x + side * 7 + f * 2, y - 7, WHITE);
                    }
                    fb.hline(x - 6, x + 6, y + 6 + ((t / 5) % 2) as i32, RED);
                }
                // Its fragments in the second half.
                if self.hp * 3 <= self.max * 2 && self.state == 1 {
                    for n in 0..4 {
                        let a = t as f32 * 0.04 + n as f32 * std::f32::consts::FRAC_PI_2;
                        let (fx, fy) = (x + (a.cos() * 34.0) as i32, y + (a.sin() * 20.0) as i32);
                        for k in 0..6 {
                            fb.set(
                                fx + (k * 7 % 5) - 2,
                                fy + (k * 3 % 5) - 2,
                                if k % 2 == 0 { GREY } else { RED },
                            );
                        }
                    }
                }
            }
        }
    }
}
