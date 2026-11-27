//! The dead and the damned of the Undercroft: what each enemy is, how it
//! moves and how it looks. Every one is drawn by code in the C64 colours.
use crate::fb::Fb;
use crate::palette::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A terminated process risen from its grave: slow, stubborn.
    Zombie,
    /// A small orphaned process: hops at you in swarms.
    Orphan,
    /// Winged, swoops and throws fire.
    Daemon,
    /// A deleted file's ghost: floats through walls, half the time unseen.
    Ghost,
    /// Armoured, its shield stops shots from the front.
    Knight,
    /// Splits in two when destroyed.
    ForkBomb,
    /// A small half of a fork bomb.
    ForkBit,
    /// Rides the cron bells' wave across the screen.
    Wraith,
    /// Breaks and drops what it holds.
    Urn,
    /// A blade on a chain, swinging from the clock tower's beams. It cannot
    /// be destroyed; time the way past.
    Pendulum,
    /// Hangs from a cave roof and falls when NIRUX walks under it.
    Icicle,
}

impl Kind {
    pub fn hp(self) -> i32 {
        match self {
            Kind::Zombie => 2,
            Kind::Orphan => 1,
            Kind::Daemon => 3,
            Kind::Ghost => 2,
            Kind::Knight => 6,
            Kind::ForkBomb => 3,
            Kind::ForkBit => 1,
            Kind::Wraith => 1,
            Kind::Urn => 1,
            Kind::Pendulum => 9999,
            Kind::Icicle => 1,
        }
    }
    pub fn score(self) -> u32 {
        match self {
            Kind::Zombie => 100,
            Kind::Orphan => 50,
            Kind::Daemon => 300,
            Kind::Ghost => 250,
            Kind::Knight => 600,
            Kind::ForkBomb => 200,
            Kind::ForkBit => 100,
            Kind::Wraith => 100,
            Kind::Urn => 10,
            Kind::Pendulum => 0,
            Kind::Icicle => 50,
        }
    }
    /// The hit box: half width and height above the feet.
    pub fn size(self) -> (f32, f32) {
        match self {
            Kind::Zombie => (4.0, 20.0),
            Kind::Orphan => (4.0, 10.0),
            Kind::Daemon => (6.0, 10.0),
            Kind::Ghost => (6.0, 14.0),
            Kind::Knight => (5.0, 22.0),
            Kind::ForkBomb => (5.0, 10.0),
            Kind::ForkBit => (3.0, 6.0),
            Kind::Wraith => (6.0, 8.0),
            Kind::Urn => (4.0, 13.0),
            Kind::Pendulum => (6.0, 8.0),
            Kind::Icicle => (3.0, 12.0),
        }
    }
}

/// What a broken urn (or a placed reward) gives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drop {
    Nothing,
    Gems(u8),
    Weapon(super::common::Weapon),
    Thread,
    Life,
}

#[derive(Clone, Debug)]
pub struct Foe {
    pub kind: Kind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub hp: i32,
    pub t: u32,
    pub f: f32,
    /// Rising out of the ground (zombies), counting down.
    pub rise: u32,
    pub flash: u8,
    pub cd: i32,
    pub base_y: f32,
    pub state: u8,
    pub drop: Drop,
    pub on_ground: bool,
    /// A ghost is solid and seen while this is set (or while lit).
    pub shown: bool,
    pub lit: bool,
    /// Where it hangs from or started (pendulums, icicles).
    pub ax: f32,
    pub ay: f32,
}

impl Foe {
    pub fn new(kind: Kind, x: f32, y: f32) -> Foe {
        Foe {
            kind,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            hp: kind.hp(),
            t: 0,
            f: -1.0,
            rise: if kind == Kind::Zombie { 50 } else { 0 },
            flash: 0,
            cd: 60,
            base_y: y,
            state: 0,
            drop: Drop::Nothing,
            on_ground: false,
            shown: kind != Kind::Ghost,
            lit: false,
            ax: x,
            ay: y,
        }
    }

    pub fn tangible(&self) -> bool {
        match self.kind {
            Kind::Ghost => self.shown || self.lit,
            _ => true,
        }
    }

    pub fn draw(&self, fb: &mut Fb, sx: i32, sy: i32, t: u32) {
        let fl = self.flash > 0 && self.flash % 2 == 0;
        let c = |c: u8| if fl { WHITE } else { c };
        let f = if self.f < 0.0 { -1 } else { 1 };
        let fx = |d: i32| sx + d * f;
        match self.kind {
            Kind::Zombie => {
                // Rising: only the part above the ground shows.
                let cut = if self.rise > 0 {
                    sy - (50 - self.rise as i32) * 20 / 50
                } else {
                    -999
                };
                let put = |fb: &mut Fb, x: i32, y: i32, col: u8| {
                    if y < sy && y >= cut {
                        fb.set(x, y, col);
                    }
                };
                let ph = (self.t / 10) % 2;
                // Legs, dragging.
                for k in 0..7 {
                    put(
                        fb,
                        fx(-1 - if ph == 0 { k / 3 } else { 0 }),
                        sy - 1 - k,
                        c(BROWN),
                    );
                    put(
                        fb,
                        fx(1 + if ph == 1 { k / 3 } else { 0 }),
                        sy - 1 - k,
                        c(BROWN),
                    );
                }
                // A ragged body, hunched forward.
                for y in 8..17 {
                    let lean = (y - 8) / 4;
                    for x in -3..=2 {
                        let col = if (x + y) % 5 == 0 { DGREY } else { GREY };
                        put(fb, fx(x + lean), sy - y, c(col));
                    }
                }
                // Arms out.
                for k in 0..6 {
                    put(fb, fx(3 + k), sy - 15 + (k / 3), c(GREEN));
                }
                // The head, sunk low, with dim eyes.
                for y in 16..21 {
                    for x in 0..5 {
                        put(fb, fx(x + 1), sy - y, c(GREEN));
                    }
                }
                put(fb, fx(4), sy - 19, c(if t % 40 < 36 { LRED } else { RED }));
                put(fb, fx(2), sy - 19, c(BLACK));
                put(fb, fx(3), sy - 17, c(BLACK));
            }
            Kind::Orphan => {
                let hop = if self.on_ground { 0 } else { 1 };
                fb.rect(sx - 3, sy - 7, 7, 6, c(PURPLE));
                fb.disc(sx, sy - 6, 3, c(PURPLE));
                fb.set(fx(1), sy - 6, c(YELLOW));
                fb.set(fx(3), sy - 6, c(YELLOW));
                fb.set(sx, sy - 10, c(PURPLE));
                fb.set(sx - 1, sy - 9, c(PURPLE));
                fb.set(fx(-2), sy - 1 + hop, c(BLACK));
                fb.set(fx(2), sy - 1 - hop, c(BLACK));
            }
            Kind::Daemon => {
                let flap = ((self.t / 5) % 4) as i32;
                let wy = [-6, -3, 0, -3][flap as usize];
                for side in [-1, 1] {
                    fb.line(sx, sy - 7, sx + side * 10, sy - 7 + wy, c(RED));
                    fb.line(sx + side * 10, sy - 7 + wy, sx + side * 6, sy - 3, c(RED));
                    fb.line(
                        sx + side * 3,
                        sy - 6,
                        sx + side * 7,
                        sy - 4 + wy / 2,
                        c(BROWN),
                    );
                }
                fb.rect(sx - 3, sy - 10, 6, 8, c(LRED));
                fb.disc(sx, sy - 11, 3, c(LRED));
                fb.set(sx - 3, sy - 15, c(LGREY));
                fb.set(sx + 3, sy - 15, c(LGREY));
                fb.set(sx - 1, sy - 11, c(YELLOW));
                fb.set(sx + 1, sy - 11, c(YELLOW));
                fb.line(sx, sy - 2, sx - f * 4, sy + 2, c(RED));
            }
            Kind::Ghost => {
                let seen = self.shown || self.lit;
                if !seen && (t + self.t) % 9 != 0 {
                    // A faint outline now and then.
                    if (t / 2 + self.t) % 5 == 0 {
                        fb.set(sx - 5, sy - 12, DGREY);
                        fb.set(sx + 4, sy - 4, DGREY);
                    }
                    return;
                }
                // A page with a folded corner, hollow eyes and a ragged hem.
                let body = if self.lit { WHITE } else { LGREY };
                for y in 0..14 {
                    for x in -5..=5 {
                        let corner = x > 5 - (14 - y) && y > 10;
                        if corner {
                            continue;
                        }
                        let hem = y < 2 && (x + (self.t / 6) as i32) % 3 == 0;
                        if hem {
                            continue;
                        }
                        let dither = !self.lit && (x + y + t as i32) % 2 == 0;
                        if dither && !seen {
                            continue;
                        }
                        fb.set(sx + x, sy - 2 - y, c(if dither { GREY } else { body }));
                    }
                }
                fb.line(sx + 2, sy - 13, sx + 5, sy - 13, GREY);
                fb.line(sx + 2, sy - 13, sx + 2, sy - 16, GREY);
                fb.rect(sx - 3, sy - 11, 2, 3, BLACK);
                fb.rect(sx + 1, sy - 11, 2, 3, BLACK);
                fb.hline(sx - 2, sx + 1, sy - 6, BLACK);
                for k in 0..3 {
                    fb.hline(
                        sx - 3,
                        sx + 2 - k,
                        sy - 9 + k * 3 + 6,
                        if self.lit { GREY } else { DGREY },
                    );
                }
            }
            Kind::Knight => {
                let ph = (self.t / 12) % 2;
                fb.rect(fx(-1), sy - 8, 2, 8, c(DGREY));
                fb.rect(fx(1 + ph as i32), sy - 8, 2, 8, c(GREY));
                fb.rect(sx - 4, sy - 18, 8, 10, c(GREY));
                fb.hline(sx - 4, sx + 3, sy - 18, c(LGREY));
                fb.disc(sx, sy - 21, 3, c(GREY));
                fb.hline(fx(0), fx(3), sy - 21, c(RED));
                fb.set(sx, sy - 25, c(RED));
                fb.set(sx - f, sy - 26, c(RED));
                // The shield in front.
                fb.rect(fx(4).min(fx(7)), sy - 19, 4, 12, c(PURPLE));
                fb.line(fx(5), sy - 17, fx(5), sy - 9, c(LGREY));
                // The spear behind, lowered when it lunges.
                let lunge = self.state == 2;
                let (ex, ey) = if lunge {
                    (fx(16), sy - 12)
                } else {
                    (fx(-2), sy - 30)
                };
                fb.line(fx(-2), sy - 12, ex, ey, c(BROWN));
                fb.set(ex, ey, c(WHITE));
            }
            Kind::ForkBomb | Kind::ForkBit => {
                let r = if self.kind == Kind::ForkBomb { 5 } else { 3 };
                fb.disc(sx, sy - r, r, c(BLUE));
                fb.disc(sx - 1, sy - r - 1, r - 2, c(LBLUE));
                fb.set(sx - 2, sy - r - 2, WHITE);
                // The fuse burns.
                fb.line(sx, sy - r * 2, sx + 2, sy - r * 2 - 3, BROWN);
                fb.set(
                    sx + 2,
                    sy - r * 2 - 4,
                    if t % 4 < 2 { YELLOW } else { ORANGE },
                );
            }
            Kind::Wraith => {
                for k in 0..12 {
                    let tail = sx - f * k * 2;
                    let w = ((self.t as f32 * 0.4 + k as f32).sin() * 2.0) as i32;
                    let col = if k < 3 {
                        LGREY
                    } else if k < 7 {
                        GREY
                    } else {
                        DGREY
                    };
                    fb.set(tail, sy - 4 + w, col);
                    if k < 8 {
                        fb.set(tail, sy - 5 + w, col);
                    }
                }
                fb.disc(sx, sy - 5, 3, c(LGREY));
                fb.set(fx(1), sy - 6, c(LRED));
                fb.set(fx(2), sy - 6, c(LRED));
            }
            Kind::Pendulum => {
                // The chain from the anchor, the blade at its end.
                let anchor = (
                    sx + (self.ax - self.x) as i32,
                    sy + (self.ay - self.y) as i32,
                );
                let n = 14;
                for k in 0..n {
                    let x = anchor.0 + (sx - anchor.0) * k / n;
                    let y = anchor.1 + (sy - 8 - anchor.1) * k / n;
                    fb.set(x, y, if k % 2 == 0 { GREY } else { DGREY });
                }
                fb.disc(anchor.0, anchor.1, 2, BROWN);
                fb.disc(sx, sy - 6, 6, LGREY);
                fb.disc(sx, sy - 6, 4, GREY);
                fb.hline(sx - 6, sx + 6, sy - 6, WHITE);
                fb.disc(sx, sy - 6, 1, BLACK);
            }
            Kind::Icicle => {
                let shake = if self.state == 1 {
                    ((t / 2) % 2) as i32
                } else {
                    0
                };
                for k in 0..12 {
                    let w = (12 - k) / 4;
                    fb.hline(
                        sx - w + shake,
                        sx + w + shake,
                        sy - 12 + k,
                        if k < 3 {
                            WHITE
                        } else if w > 0 {
                            LBLUE
                        } else {
                            CYAN
                        },
                    );
                }
            }
            Kind::Urn => {
                fb.rect(sx - 3, sy - 9, 7, 8, c(ORANGE));
                fb.hline(sx - 2, sx + 2, sy - 10, c(ORANGE));
                fb.hline(sx - 3, sx + 3, sy - 6, c(BROWN));
                fb.set(sx - 2, sy - 8, c(YELLOW));
                fb.hline(sx - 2, sx + 2, sy - 1, c(BROWN));
            }
        }
    }
}
