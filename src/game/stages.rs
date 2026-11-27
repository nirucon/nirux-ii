//! The stages, built column by column: ground and hills, pits, water,
//! platforms, ladders, the graves the dead rise from, urns, shrines, and
//! what is said and rung along the way.
use super::boss::BossKind;
use super::common::Weapon;
use super::foes::{Drop, Kind};
use super::stage::{Built, Ev, Placed, Shrine, Trigger, Who};
use super::world::{Deco, Map, T};

pub const ROWS: i32 = 25;
const G: i32 = 21;

/// A cursor that lays the stage out from left to right.
struct B {
    m: Map,
    x: i32,
    g: i32,
    trig: Vec<Trigger>,
    placed: Vec<Placed>,
    shrines: Vec<Shrine>,
}

// Builder helpers; later stages use the rest.
#[allow(dead_code)]
impl B {
    fn new(w: i32) -> B {
        B {
            m: Map::new(w, ROWS),
            x: 0,
            g: G,
            trig: Vec::new(),
            placed: Vec::new(),
            shrines: Vec::new(),
        }
    }

    fn px(&self, dx: i32) -> f32 {
        ((self.x + dx) * T) as f32 + 4.0
    }

    fn floor_y(&self) -> f32 {
        (self.g * T) as f32
    }

    /// Flat ground for n columns.
    fn flat(&mut self, n: i32) -> &mut Self {
        let g = self.g;
        self.m.ground(self.x, self.x + n - 1, g);
        self.x += n;
        self
    }

    /// The ground steps up (dh < 0) or down, then runs flat.
    fn step(&mut self, dh: i32, n: i32) -> &mut Self {
        self.g = (self.g + dh).clamp(8, ROWS - 2);
        self.flat(n)
    }

    fn pit(&mut self, n: i32) -> &mut Self {
        self.x += n;
        self
    }

    /// Black water from the ground's level down, n columns wide.
    fn pool(&mut self, n: i32) -> &mut Self {
        let g = self.g;
        self.m.water(self.x, self.x + n - 1, g + 1);
        self.x += n;
        self
    }

    fn plat(&mut self, dx: i32, dy: i32, len: i32) -> &mut Self {
        let (x, y) = (self.x + dx, self.g + dy);
        self.m.plat(x, y, len);
        self
    }

    fn crumble(&mut self, dx: i32, dy: i32, len: i32) -> &mut Self {
        let (x, y) = (self.x + dx, self.g + dy);
        self.m.crumbles(x, y, len);
        self
    }

    fn hidden(&mut self, dx: i32, dy: i32, len: i32) -> &mut Self {
        let (x, y) = (self.x + dx, self.g + dy);
        self.m.hidden(x, y, len);
        self
    }

    fn deco(&mut self, dx: i32, d: Deco) -> &mut Self {
        let (x, y) = ((self.x + dx) * T + 4, self.g * T);
        self.m.deco(x, y, d);
        self
    }

    fn deco_at(&mut self, dx: i32, dy: i32, d: Deco) -> &mut Self {
        let (x, y) = ((self.x + dx) * T + 4, (self.g + dy) * T);
        self.m.deco(x, y, d);
        self
    }

    fn foe(&mut self, dx: i32, dy: i32, kind: Kind, n: u8) -> &mut Self {
        let (x, y) = (self.px(dx), ((self.g + dy) * T) as f32);
        self.placed.push(Placed {
            x,
            y,
            kind,
            drop: Drop::Nothing,
            n,
            done: false,
        });
        self
    }

    /// A grave with one of the dead waiting in it.
    fn grave(&mut self, dx: i32) -> &mut Self {
        self.deco(dx, Deco::Grave);
        self.foe(dx, 0, Kind::Zombie, 1)
    }

    fn urn(&mut self, dx: i32, dy: i32, drop: Drop) -> &mut Self {
        let (x, y) = (self.px(dx), ((self.g + dy) * T) as f32);
        self.placed.push(Placed {
            x,
            y,
            kind: Kind::Urn,
            drop,
            n: 1,
            done: false,
        });
        self
    }

    fn say(&mut self, dx: i32, who: Who, s: &'static str) -> &mut Self {
        let x = self.px(dx);
        self.trig.push(Trigger {
            x,
            ev: Ev::Say(who, s),
            done: false,
        });
        self
    }

    fn ev(&mut self, dx: i32, ev: Ev) -> &mut Self {
        let x = self.px(dx);
        self.trig.push(Trigger { x, ev, done: false });
        self
    }

    fn shrine(&mut self, dx: i32) -> &mut Self {
        let (x, y) = (self.px(dx), self.floor_y());
        self.shrines.push(Shrine { x, y, lit: false });
        self
    }

    fn ladder(&mut self, dx: i32, top_dy: i32) -> &mut Self {
        let (x, top) = (self.x + dx, self.g + top_dy);
        let bottom = self.g - 1;
        self.m.ladder_at(x, top, bottom);
        self
    }

    /// The arena: 40 flat columns with walls the camera stops at.
    fn arena(mut self, boss: BossKind, water: bool) -> Built {
        let a = self.x;
        if water {
            let g = self.g;
            self.m.ground(a, a + 4, g);
            self.m.water(a + 5, a + 34, g + 1);
            // Islands between the places it rises, and ledges above them.
            self.m.plat(a + 13, g, 5);
            self.m.plat(a + 23, g, 5);
            self.m.plat(a + 14, g - 5, 3);
            self.m.plat(a + 24, g - 5, 3);
            self.m.ground(a + 35, a + 39, g);
            self.x = a + 40;
        } else {
            self.flat(40);
            self.m.deco(a * T + 230, self.g * T, Deco::Tomb);
            self.m.deco(a * T + 40, self.g * T, Deco::Cross);
            self.m.deco(a * T + 300, self.g * T, Deco::Cross);
        }
        let arena = (a * T) as f32;
        self.trig.push(Trigger {
            x: arena + 40.0,
            ev: Ev::Boss(boss),
            done: false,
        });
        // The stage ends with the arena; the camera holds it once the
        // boss comes.
        self.m.crop(a + 40);
        Built {
            start: (20.0, (G * T) as f32),
            arena,
            map: self.m,
            triggers: self.trig,
            placed: self.placed,
            shrines: self.shrines,
        }
    }
}

/// Stage 1: the Process Graveyard. Rain and lightning over the graves of
/// terminated programs; the dead rise as NIRUX passes.
pub fn graveyard() -> Built {
    let mut b = B::new(900);
    // The gate.
    b.deco(2, Deco::Lantern)
        .deco(6, Deco::Fence)
        .deco(12, Deco::Tree);
    b.say(6, Who::Echo, "THE GRAVES OPEN. KEEP MOVING.");
    b.flat(20);
    b.grave(4).grave(12).deco(8, Deco::Cross).flat(18);
    b.urn(4, 0, Drop::Gems(3))
        .grave(10)
        .deco(14, Deco::Skull)
        .flat(16);
    // Up the first hill.
    b.step(-2, 10).grave(4).deco(8, Deco::Cross);
    b.urn(6, 0, Drop::Weapon(Weapon::Hammer)).flat(12);
    b.say(0, Who::Echo, "A HAMMER. IT FALLS THROUGH THEM.");
    b.step(1, 8).grave(3).grave(7).flat(10);
    b.step(1, 6).deco(2, Deco::Tree).flat(6);
    // The first pit.
    b.pit(3);
    b.flat(6).grave(2).foe(4, 0, Kind::Orphan, 3).flat(10);
    // A mausoleum: up on its roof, or round.
    b.deco(2, Deco::Lantern).deco(4, Deco::Column).flat(4);
    let m0 = b.x;
    let g = b.g;
    b.m.bricks(m0, g - 4, m0 + 12, g - 1);
    b.m.ground(m0, m0 + 12, g);
    b.m.ladder_at(m0 + 6, g - 11, g - 5);
    b.m.bricks(m0 + 3, g - 11, m0 + 9, g - 11);
    b.urn(9, -11, Drop::Thread);
    b.urn(4, -4, Drop::Gems(4));
    b.foe(10, -4, Kind::Zombie, 1);
    b.x += 13;
    b.flat(8).foe(2, 0, Kind::Orphan, 4).flat(10);
    b.shrine(4).deco(8, Deco::Lantern).flat(12);
    // The chapel: the cron bells ring for the first time.
    b.say(0, Who::Echo, "THE BELLS. DUCK WHAT COMES.");
    b.deco(14, Deco::Chapel).flat(10);
    b.ev(0, Ev::Bells);
    b.grave(6).deco(12, Deco::Cross).flat(24);
    b.step(-1, 6).grave(3).flat(8);
    // A crumbling bridge over the dark.
    b.say(0, Who::World, "THE OLD BRIDGE");
    let x0 = b.x;
    b.pit(12);
    b.m.crumbles(x0, b.g, 12);
    b.foe(6, -8, Kind::Daemon, 1);
    b.flat(8)
        .urn(3, 0, Drop::Weapon(Weapon::Bolt))
        .grave(6)
        .flat(10);
    b.step(-2, 6)
        .plat(2, -4, 4)
        .urn(3, -4, Drop::Gems(5))
        .flat(10);
    b.step(2, 8)
        .grave(2)
        .grave(6)
        .foe(4, -9, Kind::Daemon, 1)
        .flat(12);
    b.ev(0, Ev::Rain(110));
    b.ev(2, Ev::Thunder);
    b.say(4, Who::Null, "NULL: YOU CAME DOWN AFTER ME? GOOD.");
    b.pit(4);
    b.flat(10).foe(4, 0, Kind::Orphan, 5).flat(6);
    b.shrine(4).flat(10);
    // Tiers of graves on wooden walks.
    b.plat(2, -4, 6)
        .plat(10, -7, 5)
        .urn(12, -7, Drop::Weapon(Weapon::Whip))
        .grave(4)
        .flat(18);
    b.grave(3).grave(9).deco(6, Deco::Cross).flat(14);
    // Iron spikes in a dip: jump them.
    b.flat(4);
    let sx = b.x;
    b.step(1, 3);
    b.m.spikes(sx, sx + 2, b.g - 1);
    b.step(-1, 8).foe(4, -9, Kind::Daemon, 1).grave(6).flat(10);
    b.pit(3);
    b.flat(6).foe(2, 0, Kind::Orphan, 4).flat(10);
    // The Reaper looks on from the last hill.
    b.step(-2, 6);
    b.grave(2).grave(5).grave(9).grave(13);
    b.ev(0, Ev::Reaper);
    b.flat(18);
    b.say(0, Who::Echo, "SOMETHING DIGS BELOW. BE READY.");
    b.step(2, 10).urn(5, 0, Drop::Thread).flat(8);
    b.arena(BossKind::Gravedigger, false)
}

/// Stage 2: the Swap Marsh. Fog, black water and sinking logs, orphans in
/// swarms, ghost files, the stilt village and the Bog Daemon.
pub fn marsh() -> Built {
    let mut b = B::new(900);
    b.deco(4, Deco::Reeds)
        .deco(10, Deco::Stump)
        .deco(14, Deco::Tree);
    b.say(6, Who::Echo, "THE MARSH SWALLOWS THE FALLEN.");
    b.flat(22);
    b.foe(4, 0, Kind::Orphan, 3).deco(8, Deco::Reeds).flat(12);
    // Pools with logs to cross.
    b.pool(3).flat(4).pool(4).flat(6);
    b.deco(2, Deco::Reeds).foe(3, 0, Kind::Orphan, 4).flat(10);
    let x0 = b.x;
    b.pool(14);
    b.m.plat(x0 + 3, b.g, 3);
    b.m.plat(x0 + 9, b.g, 3);
    b.flat(8).urn(4, 0, Drop::Weapon(Weapon::Disc)).flat(6);
    b.say(0, Who::Echo, "A DISC. IT BOUNCES OFF WALLS.");
    // Sinking logs.
    let x0 = b.x;
    b.pool(16);
    b.m.crumbles(x0 + 2, b.g, 3);
    b.m.crumbles(x0 + 7, b.g, 3);
    b.m.crumbles(x0 + 12, b.g, 2);
    b.foe(8, -8, Kind::Daemon, 1);
    b.flat(10).deco(4, Deco::Tree).flat(8);
    // The hidden way: the lamp shows it.
    b.say(0, Who::Echo, "HOLD THE LAMP. SEE WHAT HIDES.");
    b.flat(6);
    let x0 = b.x;
    b.pool(26);
    for k in 0..4 {
        b.m.hidden(x0 + 2 + k * 6, b.g - (k % 2), 3);
    }
    b.foe(14, -6, Kind::Ghost, 1);
    b.flat(8)
        .urn(3, 0, Drop::Thread)
        .foe(6, -4, Kind::Ghost, 1)
        .flat(10);
    // A helmet for the one who looks up: high on a hidden ledge.
    b.hidden(-8, -5, 3).urn(-7, -5, Drop::Life);
    b.shrine(4).deco(8, Deco::Lantern).flat(12);
    // The stilt village.
    b.deco(4, Deco::Hut).flat(8);
    let v0 = b.x;
    b.flat(30);
    let g = b.g;
    b.m.plat(v0 + 2, g - 6, 10);
    b.m.ladder_at(v0 + 1, g - 6, g - 1);
    b.m.deco((v0 + 7) * T, (g - 6) * T, Deco::Hut);
    b.m.plat(v0 + 16, g - 7, 10);
    b.m.ladder_at(v0 + 26, g - 7, g - 1);
    b.m.deco((v0 + 21) * T, (g - 7) * T, Deco::Hut);
    b.placed.push(Placed {
        x: ((v0 + 9) * T) as f32,
        y: ((g - 6) * T) as f32,
        kind: Kind::Urn,
        drop: Drop::Weapon(Weapon::Bolt),
        n: 1,
        done: false,
    });
    b.placed.push(Placed {
        x: ((v0 + 18) * T) as f32,
        y: ((g - 7) * T) as f32,
        kind: Kind::Urn,
        drop: Drop::Gems(6),
        n: 1,
        done: false,
    });
    b.placed.push(Placed {
        x: ((v0 + 10) * T) as f32,
        y: (g * T) as f32,
        kind: Kind::ForkBomb,
        drop: Drop::Nothing,
        n: 1,
        done: false,
    });
    b.placed.push(Placed {
        x: ((v0 + 24) * T) as f32,
        y: (g * T) as f32,
        kind: Kind::Knight,
        drop: Drop::Gems(3),
        n: 1,
        done: false,
    });
    b.say(-20, Who::Echo, "A SHIELD. HIT IT FROM ABOVE.");
    b.pool(4).flat(10).foe(4, 0, Kind::Orphan, 5).flat(6);
    // The bells over the marsh.
    b.ev(0, Ev::Bells);
    b.flat(26);
    b.pool(3)
        .flat(6)
        .foe(2, -7, Kind::Daemon, 1)
        .pool(3)
        .flat(10);
    let x0 = b.x;
    b.pool(18);
    b.m.crumbles(x0 + 2, b.g, 2);
    b.m.plat(x0 + 6, b.g - 2, 3);
    b.m.crumbles(x0 + 11, b.g, 2);
    b.m.hidden(x0 + 15, b.g - 1, 2);
    b.foe(9, -6, Kind::Ghost, 1);
    b.flat(8).foe(4, 0, Kind::ForkBomb, 1).flat(10);
    b.say(0, Who::Null, "NULL: THE WATER KEEPS WHAT IT TAKES.");
    b.shrine(4).flat(12);
    b.foe(4, 0, Kind::Knight, 1)
        .foe(16, 0, Kind::ForkBomb, 1)
        .flat(24);
    b.step(-2, 6)
        .foe(3, -9, Kind::Daemon, 1)
        .plat(1, -4, 4)
        .urn(2, -4, Drop::Gems(4))
        .flat(8);
    b.step(2, 10).foe(2, 0, Kind::Orphan, 6).flat(10);
    b.say(0, Who::Echo, "IT RISES. STAND ON THE ISLANDS.");
    b.urn(6, 0, Drop::Thread).flat(12);
    b.arena(BossKind::BogDaemon, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::world;

    fn check(b: &Built) {
        let w = b.map.w;
        assert!(b.arena > 0.0 && b.arena < (w * T) as f32);
        // The start stands on ground.
        let tx = (b.start.0 / T as f32) as i32;
        assert_eq!(b.map.top_at(tx, 0), G);
        // No wall on the way is taller than a jump can clear (4 rows),
        // and no pit wider than a jump (5 columns) without platforms.
        let mut prev = b.map.top_at(0, 0);
        let mut gap = 0;
        for x in 1..(b.arena / T as f32) as i32 - 1 {
            let from = (prev - 6).clamp(7, ROWS);
            // Something to stand on: ground, a ledge, a log, a hidden step.
            let any = (from..ROWS).any(|y| {
                let v = b.map.at(x, y);
                world::Map::floor(v) && !world::Map::ladder(v)
            });
            if !any {
                gap += 1;
                assert!(gap <= 5, "gap too wide at column {x}");
                continue;
            }
            gap = 0;
            // A wall is solid ground only; ledges can be jumped through.
            if let Some(top) = (from..ROWS).find(|&y| world::Map::solid(b.map.at(x, y))) {
                assert!(
                    prev - top <= 4,
                    "wall too high at column {x}: {prev} to {top}"
                );
                prev = top;
            }
        }
    }

    #[test]
    fn every_line_fits_the_screen() {
        for b in [graveyard(), marsh()] {
            for t in &b.triggers {
                if let Ev::Say(w, s) = &t.ev {
                    let tag = if *w == Who::Echo { 6 } else { 0 };
                    assert!(s.len() + tag <= 38, "too long: {s}");
                }
            }
        }
    }

    #[test]
    fn stages_are_passable() {
        check(&graveyard());
        check(&marsh());
    }
}
