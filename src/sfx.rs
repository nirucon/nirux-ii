//! Sound effects for the same synthesiser: short programs of sweeps on the
//! two effect voices. Each segment glides its frequency smoothly (per
//! sample) and its volume per 5 ms step; a busy voice is taken over only by
//! an effect of the same or higher priority, so a laser never cuts off an
//! explosion.
use crate::synth::{Synth, Wave, MUSIC_VOICES};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    Bolt,
    Hammer,
    Disc,
    Whip,
    EnemyShot,
    Hit,
    ExplodeSmall,
    ExplodeBig,
    PowerUp,
    Gem,
    Mend,
    ExtraLife,
    PlayerDie,
    ScarfTear,
    Jump,
    Land,
    Bell,
    Thunder,
    Splash,
    Rise,
    Lamp,
    Crumble,
    MenuMove,
    MenuSelect,
    Alarm,
    Tally,
    Clang,
}

#[derive(Clone, Copy, Debug)]
pub struct Seg {
    pub wave: Wave,
    pub f0: f32,
    pub f1: f32,
    pub ms: f32,
    pub v0: f32,
    pub v1: f32,
    pub pw: f32,
}

const fn s(wave: Wave, f0: f32, f1: f32, ms: f32, v0: f32, v1: f32, pw: f32) -> Seg {
    Seg {
        wave,
        f0,
        f1,
        ms,
        v0,
        v1,
        pw,
    }
}

use Wave::{Noise as N, Pulse as P, Saw as W, Tri as T};

macro_rules! segs {
    ($($e:expr),* $(,)?) => {{
        const S: &[Seg] = &[$($e),*];
        S
    }};
}

pub fn def(x: Sfx) -> (u8, &'static [Seg]) {
    match x {
        Sfx::Bolt => (1, segs![s(P, 1500.0, 700.0, 50.0, 0.5, 0.0, 0.2)]),
        Sfx::Hammer => (
            1,
            segs![
                s(T, 300.0, 900.0, 60.0, 0.6, 0.4, 0.5),
                s(N, 2000.0, 900.0, 50.0, 0.3, 0.0, 0.5),
            ],
        ),
        Sfx::Disc => (1, segs![s(P, 900.0, 2400.0, 70.0, 0.45, 0.0, 0.1)]),
        Sfx::Whip => (
            1,
            segs![
                s(N, 1500.0, 7000.0, 60.0, 0.1, 0.6, 0.5),
                s(N, 7000.0, 3000.0, 40.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::EnemyShot => (1, segs![s(T, 700.0, 200.0, 110.0, 0.55, 0.0, 0.5)]),
        Sfx::Hit => (2, segs![s(N, 6000.0, 2500.0, 35.0, 0.5, 0.0, 0.5)]),
        Sfx::ExplodeSmall => (
            3,
            segs![
                s(N, 3000.0, 1200.0, 40.0, 1.0, 0.8, 0.5),
                s(N, 1200.0, 150.0, 300.0, 0.8, 0.0, 0.5),
            ],
        ),
        Sfx::ExplodeBig => (
            5,
            segs![
                s(N, 4000.0, 900.0, 60.0, 1.0, 1.0, 0.5),
                s(N, 900.0, 50.0, 1200.0, 1.0, 0.0, 0.5),
            ],
        ),
        Sfx::PowerUp => (
            4,
            segs![
                s(P, 440.0, 440.0, 50.0, 0.6, 0.6, 0.5),
                s(P, 587.0, 587.0, 50.0, 0.6, 0.6, 0.5),
                s(P, 698.0, 698.0, 50.0, 0.6, 0.6, 0.4),
                s(P, 880.0, 880.0, 200.0, 0.6, 0.0, 0.3),
            ],
        ),
        Sfx::Gem => (
            2,
            segs![
                s(P, 1760.0, 1760.0, 35.0, 0.45, 0.45, 0.3),
                s(P, 2637.0, 2637.0, 90.0, 0.45, 0.0, 0.3),
            ],
        ),
        Sfx::Mend => (
            4,
            segs![
                s(T, 587.0, 587.0, 80.0, 0.6, 0.6, 0.5),
                s(T, 880.0, 880.0, 80.0, 0.6, 0.6, 0.5),
                s(T, 1175.0, 1175.0, 260.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::ExtraLife => (
            4,
            segs![
                s(P, 587.0, 587.0, 70.0, 0.6, 0.6, 0.5),
                s(P, 880.0, 880.0, 70.0, 0.6, 0.6, 0.5),
                s(P, 1175.0, 1175.0, 70.0, 0.6, 0.6, 0.5),
                s(P, 1760.0, 1760.0, 300.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::PlayerDie => (
            6,
            segs![
                s(W, 900.0, 600.0, 80.0, 0.8, 0.8, 0.5),
                s(T, 600.0, 80.0, 700.0, 0.9, 0.0, 0.5),
            ],
        ),
        Sfx::ScarfTear => (
            5,
            segs![
                s(N, 5000.0, 2000.0, 80.0, 0.8, 0.5, 0.5),
                s(W, 400.0, 150.0, 180.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::Jump => (1, segs![s(P, 300.0, 700.0, 70.0, 0.35, 0.0, 0.3)]),
        Sfx::Land => (1, segs![s(N, 900.0, 300.0, 40.0, 0.3, 0.0, 0.5)]),
        // A bell: a bright strike that rings out low.
        Sfx::Bell => (
            5,
            segs![
                s(P, 1568.0, 1560.0, 30.0, 0.8, 0.6, 0.12),
                s(T, 392.0, 390.0, 1400.0, 0.9, 0.0, 0.5),
            ],
        ),
        Sfx::Thunder => (
            3,
            segs![
                s(N, 600.0, 300.0, 120.0, 0.4, 1.0, 0.5),
                s(N, 300.0, 60.0, 1600.0, 1.0, 0.0, 0.5),
            ],
        ),
        Sfx::Splash => (2, segs![s(N, 2500.0, 600.0, 220.0, 0.6, 0.0, 0.5)]),
        Sfx::Rise => (
            2,
            segs![
                s(N, 200.0, 900.0, 300.0, 0.2, 0.5, 0.5),
                s(N, 900.0, 300.0, 200.0, 0.5, 0.0, 0.5)
            ],
        ),
        Sfx::Lamp => (1, segs![s(T, 1200.0, 2400.0, 90.0, 0.3, 0.0, 0.5)]),
        Sfx::Crumble => (2, segs![s(N, 1200.0, 200.0, 350.0, 0.6, 0.0, 0.5)]),
        Sfx::MenuMove => (1, segs![s(T, 1400.0, 1300.0, 35.0, 0.6, 0.0, 0.5)]),
        Sfx::MenuSelect => (
            2,
            segs![
                s(P, 587.0, 587.0, 60.0, 0.6, 0.6, 0.5),
                s(P, 1175.0, 1175.0, 150.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::Alarm => (
            5,
            segs![
                s(P, 466.0, 466.0, 220.0, 0.6, 0.6, 0.5),
                s(P, 440.0, 440.0, 220.0, 0.6, 0.6, 0.5),
                s(P, 466.0, 466.0, 220.0, 0.6, 0.6, 0.5),
                s(P, 440.0, 440.0, 400.0, 0.6, 0.0, 0.5),
            ],
        ),
        Sfx::Tally => (1, segs![s(P, 2200.0, 2200.0, 25.0, 0.4, 0.0, 0.3)]),
        Sfx::Clang => (
            2,
            segs![
                s(P, 1500.0, 1400.0, 20.0, 0.5, 0.4, 0.1),
                s(T, 700.0, 650.0, 90.0, 0.4, 0.0, 0.5),
            ],
        ),
    }
}

#[derive(Clone, Debug, Default)]
pub struct Slot {
    segs: Option<&'static [Seg]>,
    seg: usize,
    t: f32,
    prio: u8,
}

/// 5 ms steps.
pub const STEP_MS: f32 = 5.0;

pub fn start(slots: &mut [Slot; 2], synth: &mut Synth, x: Sfx) {
    let (prio, segs) = def(x);
    // A free voice, else the lower priority one, else nothing.
    let pick = (0..2).find(|&i| slots[i].segs.is_none()).or_else(|| {
        let i = if slots[0].prio <= slots[1].prio { 0 } else { 1 };
        (slots[i].prio <= prio).then_some(i)
    });
    let Some(i) = pick else {
        return;
    };
    slots[i] = Slot {
        segs: Some(segs),
        seg: 0,
        t: 0.0,
        prio,
    };
    let v = &mut synth.v[MUSIC_VOICES + i];
    v.env.set(0, 0, 15, 3);
    v.retrigger();
    apply(&slots[i], v, true);
}

fn apply(slot: &Slot, v: &mut crate::synth::Voice, new_seg: bool) {
    let Some(segs) = slot.segs else {
        return;
    };
    let g = segs[slot.seg];
    let k = (slot.t / g.ms.max(1.0)).min(1.0);
    v.vol = g.v0 + (g.v1 - g.v0) * k;
    if new_seg {
        v.wave = g.wave;
        v.pw = g.pw;
        v.freq = g.f0;
        let samples = (g.ms / 1000.0 * crate::synth::RATE as f32).max(1.0);
        v.fmul = (g.f1 / g.f0).powf(1.0 / samples);
    }
}

/// Advance every effect by one 5 ms step.
pub fn step(slots: &mut [Slot; 2], synth: &mut Synth) {
    for (i, slot) in slots.iter_mut().enumerate() {
        let v = &mut synth.v[MUSIC_VOICES + i];
        let Some(segs) = slot.segs else {
            continue;
        };
        slot.t += STEP_MS;
        let mut new_seg = false;
        if slot.t >= segs[slot.seg].ms {
            slot.t = 0.0;
            slot.seg += 1;
            new_seg = true;
            if slot.seg >= segs.len() {
                slot.segs = None;
                slot.prio = 0;
                v.set_gate(false);
                v.fmul = 1.0;
                continue;
            }
        }
        apply(slot, v, new_seg);
    }
}
