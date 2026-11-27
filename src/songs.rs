//! The NIRUX II soundtrack, composed for this game: gothic and minor, with
//! organ-like pulse chords, plucked harpsichord runs, a tolling bell, and
//! the hero's call from NIRUX coming back in a darker key. Voice 1 plays
//! bass and drums, voice 2 chords or an echo of the lead, voice 3 the lead.
use crate::music::{Instr, Pitch, Song, SongB, Step};
use crate::synth::{Wave, BP, LP};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SongId {
    Title,
    Legend,
    Stage1,
    Stage2,
    Stage3,
    Stage4,
    Stage5,
    Stage6,
    Stage7,
    FinalBoss,
    Boss,
    GameOver,
    Clear,
    Ending,
}

pub const ALL: &[SongId] = &[
    SongId::Title,
    SongId::Legend,
    SongId::Stage1,
    SongId::Stage2,
    SongId::Stage3,
    SongId::Stage4,
    SongId::Stage5,
    SongId::Stage6,
    SongId::Stage7,
    SongId::FinalBoss,
    SongId::Boss,
    SongId::GameOver,
    SongId::Clear,
    SongId::Ending,
];

impl SongId {
    pub fn name(self) -> &'static str {
        match self {
            SongId::Title => "title",
            SongId::Legend => "legend",
            SongId::Stage1 => "stage1",
            SongId::Stage2 => "stage2",
            SongId::Stage3 => "stage3",
            SongId::Stage4 => "stage4",
            SongId::Stage5 => "stage5",
            SongId::Stage6 => "stage6",
            SongId::Stage7 => "stage7",
            SongId::FinalBoss => "finalboss",
            SongId::Boss => "boss",
            SongId::GameOver => "gameover",
            SongId::Clear => "clear",
            SongId::Ending => "ending",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            SongId::Title => "Daemons (title theme)",
            SongId::Legend => "Descent (the prologue)",
            SongId::Stage1 => "Graveyard (stage 1)",
            SongId::Stage2 => "Marsh (stage 2)",
            SongId::Stage3 => "Cathedral (stage 3)",
            SongId::Stage4 => "Clockwork (stage 4)",
            SongId::Stage5 => "Frost (stage 5)",
            SongId::Stage6 => "Kernel (stage 6)",
            SongId::Stage7 => "The Keep (stage 7)",
            SongId::FinalBoss => "Null (the last battle)",
            SongId::Boss => "The Bells Toll (bosses)",
            SongId::GameOver => "Signal Lost (game over)",
            SongId::Clear => "Deeper (stage clear)",
            SongId::Ending => "Home (the ending)",
        }
    }
    pub fn from_name(s: &str) -> Option<SongId> {
        ALL.iter().copied().find(|i| i.name() == s)
    }
}

// Instrument numbers used by every song.
const BASS: u8 = 0;
const KICK: u8 = 1;
const SNARE: u8 = 2;
const HAT: u8 = 3;
const ARP: u8 = 4;
const LEAD: u8 = 5;
const ECHO: u8 = 6;
const PAD: u8 = 7;
const SAW: u8 = 8;
const PLUCK: u8 = 9;
const DR: [u8; 3] = [KICK, SNARE, HAT];

fn st(wave: Wave, pitch: Pitch, ticks: u8) -> Step {
    Step { wave, pitch, ticks }
}

/// The instrument set; `bright` opens the bass filter envelope wider.
fn kit(bright: f32) -> Vec<Instr> {
    use Wave::*;
    vec![
        // 0 bass: pulse through the low-pass filter, a snappy envelope.
        Instr {
            wave: Pulse,
            adsr: [0, 8, 9, 4],
            pw: 0.28,
            pwm: 0.012,
            filter: true,
            fenv: Some((3.2 * bright, 1.0, 9)),
            vol: 1.0,
            ..Default::default()
        },
        // 1 kick: a noise click, then a falling triangle.
        Instr {
            wave: Tri,
            adsr: [0, 6, 0, 2],
            prog: vec![st(Noise, Pitch::Abs(96.0), 1), st(Tri, Pitch::Abs(57.0), 1)],
            slide: -3.2,
            vol: 1.25,
            ..Default::default()
        },
        // 2 snare: noise, a pulse body, then darker noise.
        Instr {
            wave: Noise,
            adsr: [0, 7, 0, 3],
            prog: vec![
                st(Noise, Pitch::Abs(104.0), 1),
                st(Pulse, Pitch::Abs(57.0), 1),
                st(Noise, Pitch::Abs(94.0), 1),
            ],
            slide: -0.35,
            pw: 0.4,
            vol: 1.0,
            ..Default::default()
        },
        // 3 hi-hat.
        Instr {
            wave: Noise,
            adsr: [0, 3, 0, 1],
            prog: vec![st(Noise, Pitch::Abs(124.0), 1)],
            vol: 0.5,
            gate: 1,
            ..Default::default()
        },
        // 4 arpeggio chord: the shimmering SID chord.
        Instr {
            wave: Pulse,
            adsr: [0, 9, 7, 7],
            pw: 0.22,
            pwm: 0.006,
            vol: 0.62,
            ..Default::default()
        },
        // 5 lead: pulse with a slowly turning width and delayed vibrato.
        Instr {
            wave: Pulse,
            adsr: [1, 9, 10, 8],
            pw: 0.42,
            pwm: 0.004,
            vib: (14, 0.3, 8.0),
            vol: 1.1,
            ..Default::default()
        },
        // 6 echo of the lead, softer.
        Instr {
            wave: Pulse,
            adsr: [1, 9, 8, 8],
            pw: 0.3,
            pwm: 0.004,
            vib: (14, 0.3, 8.0),
            vol: 0.42,
            ..Default::default()
        },
        // 7 pad: a slow triangle.
        Instr {
            wave: Tri,
            adsr: [9, 10, 12, 10],
            vib: (30, 0.15, 10.0),
            vol: 1.1,
            ..Default::default()
        },
        // 8 saw lead for the big moments.
        Instr {
            wave: Saw,
            adsr: [1, 9, 11, 8],
            vib: (12, 0.32, 8.0),
            vol: 0.85,
            ..Default::default()
        },
        // 9 pluck: short bright pulse for runs.
        Instr {
            wave: Pulse,
            adsr: [0, 6, 4, 5],
            pw: 0.5,
            pwm: 0.01,
            vol: 0.8,
            ..Default::default()
        },
    ]
}

pub fn build(id: SongId) -> Song {
    match id {
        SongId::Title => title(),
        SongId::Legend => legend(),
        SongId::Stage1 => stage1(),
        SongId::Stage2 => stage2(),
        SongId::Stage3 => stage3(),
        SongId::Stage4 => stage4(),
        SongId::Stage5 => stage5(),
        SongId::Stage6 => stage6(),
        SongId::Stage7 => stage7(),
        SongId::FinalBoss => final_boss(),
        SongId::Boss => boss(),
        SongId::GameOver => game_over(),
        SongId::Clear => clear(),
        SongId::Ending => ending(),
    }
}

/// "Daemons": D minor. A tolling intro, then the hero's call from NIRUX
/// (D, A, up to D) on the saw over an organ of pulse chords, and a harsh
/// harpsichord run in the bridge.
fn title() -> Song {
    let mut b = SongB::new("title", 6, kit(0.9));
    b.song.res = 0.6;
    let intro = "Dm:16 Bb:16 Gm:16 A:16";
    b.cutoff(0, 220.0).cutoff(64, 1800.0);
    b.groove(0, intro, "b.......b.......", 2, BASS, DR);
    b.groove(1, intro, "c...............", 4, ARP, DR);
    b.mel(2, PAD, "a5:16 | f5:16 | d5:16 | c#5:16");
    b.mark_loop();
    let at = b.sync("call");
    let call = "Dm:16 Bb:16 F:16 C:16 Dm:16 Bb:16 Gm:16 A:16";
    b.cutoff(at, 1600.0).cutoff(at + 127, 2600.0);
    b.groove(0, call, "K.boS.boK.boS.bo", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "d5:4 a5:4 d6:8 | f6:4 d6:4 bb5:8 | a5:4 c6:4 f6:8 | e6:6 d6:2 c6:8 |
         d5:4 a5:4 d6:8 | f6:4 g6:4 d6:8 | bb5:4 d6:4 g6:8 | a6:12 e6:4",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    let at = b.sync("organ");
    let organ = "Gm:16 Dm:16 Eb:16 A:16 Gm:16 Dm:16 Bb:16 A:16";
    b.cutoff(at, 2200.0);
    b.groove(0, organ, "K.obS.obK.obS.oS", 2, BASS, DR);
    b.groove(1, organ, "C.......C.......", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "g5:4 bb5:4 d6:4 g6:4 | f6:6 e6:2 d6:8 | eb6:4 d6:4 c6:4 bb5:4 | a5:12 c#6:4 |
         d6:4 bb5:4 g5:4 bb5:4 | a5:4 f5:4 d5:8 | bb5:4 c6:4 d6:4 f6:4 | e6:8 c#6:8",
    );
    let at = b.sync("run");
    let run = "Dm:16 Gm:16 A:16 Dm:16";
    b.cutoff(at, 1000.0).cutoff(at + 63, 2800.0);
    b.groove(0, run, "K...b...S...b..o", 2, BASS, DR);
    b.groove(1, run, "c...............", 4, ARP, DR);
    b.mel(
        2,
        PLUCK,
        "d5 f5 a5 d6 f6 a6 f6 d6 a5 d6 f6 a6 d7 a6 f6 d6 |
         g5 bb5 d6 g6 bb6 g6 d6 bb5 g5 bb5 d6 g6 bb6 d7 bb6 g6 |
         a5 c#6 e6 a6 c#7 a6 e6 c#6 a5 e6 a6 c#7 e7 c#7 a6 e6 |
         @8 d6:4 a5:4 f5:4 d5:4",
    );
    b.finish()
}

/// "Descent": the call, slow and far away, as NIRUX goes down.
fn legend() -> Song {
    let mut b = SongB::new("legend", 8, kit(0.7));
    b.song.res = 0.65;
    let dark = "Am:16 F:16 Dm:16 E:16";
    b.cutoff(0, 180.0).cutoff(64, 1100.0);
    b.groove(0, dark, "b...............", 2, BASS, DR);
    b.groove(1, dark, "c...............", 4, ARP, DR);
    b.mel(2, PAD, "e5:16 | c5:16 | a4:16 | g#4:16");
    b.mark_loop();
    let at = b.sync("call");
    let call = "Am:16 F:16 C:16 G:16 Am:16 F:16 Dm:16 E:16";
    b.cutoff(at, 1000.0).cutoff(at + 127, 1900.0);
    b.groove(0, call, "K.......S.......", 2, BASS, DR);
    b.mel(
        2,
        LEAD,
        "a4:4 e5:4 a5:8 | c6:4 a5:4 f5:8 | e5:4 g5:4 c6:8 | b5:6 a5:2 g5:8 |
         a4:4 e5:4 a5:8 | c6:4 d6:4 a5:8 | f5:4 a5:4 d6:8 | e6:12 b5:4",
    );
    b.echo(2, 1, at, 128, 4, ECHO);
    b.finish()
}

/// "Graveyard": A minor, driving, a harpsichord run over a marching bass.
fn stage1() -> Song {
    let mut b = SongB::new("stage1", 5, kit(1.1));
    b.song.res = 0.55;
    let intro = "Am:16 G:16 F:16 E:16";
    b.cutoff(0, 500.0).cutoff(63, 2400.0);
    b.groove(0, intro, "K.bbS.bbK.bbS.bb", 2, BASS, DR);
    b.groove(1, intro, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.rest(2, 64);
    b.mark_loop();
    let at = b.sync("verse");
    let verse = "Am:16 G:16 F:16 E:16 Am:16 G:16 F:16 E:16";
    b.cutoff(at, 1900.0);
    b.groove(0, verse, "K.boS.boK.boS.bo", 2, BASS, DR);
    b.groove(1, verse, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "a5:6 c6:2 e6:8 | d6:4 c6:4 b5:8 | a5:4 c6:4 f6:6 e6:2 | e6:12 g#5:4 |
         a5:6 c6:2 e6:8 | g6:4 f6:4 e6:8 | d6:4 c6:4 b5:4 a5:4 | g#5:12 e5:4",
    );
    let at = b.sync("chorus");
    let chorus = "F:16 G:16 Am:16 Am:16 F:16 G:16 E:16 E:16";
    b.cutoff(at, 2600.0);
    b.groove(0, chorus, "K.obS.obK.obS.oS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "a5:4 c6:4 f6:8 | g6:4 f6:4 d6:8 | e6:4 a6:4 c7:8 | b6:6 a6:2 e6:8 |
         f6:4 a6:4 c7:8 | d7:4 c7:4 b6:8 | g#6:4 b6:4 e7:8 | e7:16",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    let at = b.sync("run");
    let run = "Am:16 Dm:16 E:16 Am:16";
    b.cutoff(at, 1200.0).cutoff(at + 63, 2600.0);
    b.groove(0, run, "K.b.S.b.K.b.S.bS", 2, BASS, DR);
    b.groove(1, run, "c.......c.......", 4, ARP, DR);
    b.mel(
        2,
        PLUCK,
        "a5 c6 e6 a6 e6 c6 a5 c6 e6 a6 c7 a6 e6 c6 a5 e5 |
         d6 f6 a6 d7 a6 f6 d6 f6 a6 d7 f7 d7 a6 f6 d6 a5 |
         e6 g#6 b6 e7 b6 g#6 e6 g#6 b6 e7 g#7 e7 b6 g#6 e6 b5 |
         @8 a5:4 e5:4 c5:4 a4:4",
    );
    b.finish()
}

/// "Marsh": E minor, slow and murky, a swaying bass and a lonely lead.
fn stage2() -> Song {
    let mut b = SongB::new("stage2", 6, kit(0.8));
    b.song.res = 0.7;
    let intro = "Em:16 C:16 Am:16 B:16";
    b.cutoff(0, 300.0).cutoff(63, 1500.0);
    b.groove(0, intro, "b..ob..ob..ob..o", 2, BASS, DR);
    b.groove(1, intro, "c.......c.......", 4, ARP, DR);
    b.mel(2, PAD, "b4:16 | g4:16 | e4:16 | d#5:16");
    b.mark_loop();
    let at = b.sync("verse");
    let verse = "Em:16 C:16 Am:16 B:16 Em:16 C:16 D:16 B:16";
    b.cutoff(at, 1300.0);
    b.groove(0, verse, "K..oS..oK.boS..o", 2, BASS, DR);
    b.groove(1, verse, "c.....c.....c...", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "e5:8 g5:4 b5:4 | c6:8 b5:4 g5:4 | a5:6 g5:2 e5:8 | f#5:12 d#5:4 |
         e5:8 g5:4 b5:4 | e6:6 d6:2 c6:8 | d6:4 c6:4 a5:4 f#5:4 | b5:16",
    );
    let at = b.sync("deep");
    let deep = "C:16 D:16 Bm:16 Em:16 Am:16 B:16 Em:16 Em:16";
    b.cutoff(at, 1800.0).cutoff(at + 127, 2300.0);
    b.groove(0, deep, "K.obS.obK.obS.oS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "g5:4 c6:4 e6:8 | f#6:4 a6:4 d6:8 | f#6:4 d6:4 b5:8 | g5:6 b5:2 e6:8 |
         c6:4 e6:4 a6:8 | f#6:4 d#6:4 b5:8 | e6:12 b5:4 | e5:16",
    );
    b.echo(2, 1, at, 128, 4, ECHO);
    b.finish()
}

/// "Cathedral": G minor, an organ of pulse chords, a slow solemn lead
/// that climbs like the stairs.
fn stage3() -> Song {
    let mut b = SongB::new("stage3", 7, kit(0.8));
    b.song.res = 0.65;
    let intro = "Gm:16 Eb:16 Cm:16 D:16";
    b.cutoff(0, 400.0).cutoff(63, 1600.0);
    b.groove(0, intro, "b.......b.......", 2, BASS, DR);
    b.groove(1, intro, "C...............", 4, ARP, DR);
    b.mel(2, PAD, "d5:16 | g5:16 | eb5:16 | f#5:16");
    b.mark_loop();
    let at = b.sync("hymn");
    let hymn = "Gm:16 Dm:16 Eb:16 Bb:16 Cm:16 Gm:16 Am:16 D:16";
    b.cutoff(at, 1500.0);
    b.groove(0, hymn, "K.......S...b...", 2, BASS, DR);
    b.groove(1, hymn, "C.......C.......", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "g5:8 bb5:4 d6:4 | f6:8 d6:8 | eb6:6 d6:2 c6:8 | d6:12 bb5:4 |
         c6:8 eb6:4 g6:4 | g6:6 f6:2 d6:8 | c6:4 d6:4 eb6:4 c6:4 | d6:16",
    );
    let at = b.sync("rise");
    let rise = "Eb:16 F:16 Gm:16 Gm:16 Cm:16 D:16 Gm:16 D:16";
    b.cutoff(at, 2200.0).cutoff(at + 127, 2800.0);
    b.groove(0, rise, "K.obS.obK.obS.oS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "g5:4 bb5:4 eb6:8 | a5:4 c6:4 f6:8 | bb5:4 d6:4 g6:8 | g6:16 |
         eb6:4 g6:4 c7:8 | a6:4 f#6:4 d6:8 | g6:4 d6:4 bb5:8 | a5:8 f#5:8",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    b.finish()
}

/// "Clockwork": B minor, ticking hats, a staccato pluck that keeps time.
fn stage4() -> Song {
    let mut b = SongB::new("stage4", 5, kit(1.1));
    b.song.res = 0.5;
    let intro = "Bm:16 G:16 Em:16 F#:16";
    b.cutoff(0, 600.0).cutoff(63, 2200.0);
    b.groove(0, intro, "KhbhShbhKhbhShbh", 2, BASS, DR);
    b.groove(1, intro, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.rest(2, 64);
    b.mark_loop();
    let at = b.sync("tick");
    let tick = "Bm:16 G:16 Em:16 F#:16 Bm:16 G:16 A:16 F#:16";
    b.cutoff(at, 2000.0);
    b.groove(0, tick, "K.boS.boK.boS.bo", 2, BASS, DR);
    b.groove(1, tick, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.mel(
        2,
        PLUCK,
        "b5:2 r:2 d6:2 r:2 f#6:4 d6:4 | g5:2 r:2 b5:2 r:2 d6:8 | e5:2 r:2 g5:2 r:2 b5:4 e6:4 | c#6:8 a#5:8 |
         b5:2 r:2 d6:2 r:2 f#6:4 b6:4 | g6:4 f#6:4 e6:4 d6:4 | c#6:4 e6:4 a6:8 | f#6:16",
    );
    let at = b.sync("wind");
    let wind = "G:16 A:16 F#m:16 Bm:16 G:16 A:16 F#:16 F#:16";
    b.cutoff(at, 2600.0);
    b.groove(0, wind, "K.obS.obK.obS.oS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "b5:4 d6:4 g6:8 | c#6:4 e6:4 a6:8 | a6:4 f#6:4 c#6:8 | d6:6 c#6:2 b5:8 |
         g6:4 b6:4 d7:8 | e7:4 c#7:4 a6:8 | f#6:4 a#6:4 c#7:8 | c#7:16",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    b.finish()
}

/// "Frost": F sharp minor, sparse and glassy, bells in the arpeggio.
fn stage5() -> Song {
    let mut b = SongB::new("stage5", 7, kit(0.7));
    b.song.res = 0.6;
    let intro = "F#m:16 D:16 Bm:16 C#:16";
    b.cutoff(0, 800.0).cutoff(63, 2600.0);
    b.groove(0, intro, "b...............", 2, BASS, DR);
    b.groove(1, intro, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.mel(2, PAD, "c#6:16 | a5:16 | f#5:16 | g#5:16");
    b.mark_loop();
    let at = b.sync("snow");
    let snow = "F#m:16 D:16 A:16 E:16 F#m:16 D:16 Bm:16 C#:16";
    b.cutoff(at, 2400.0);
    b.groove(0, snow, "K.......S..ob...", 2, BASS, DR);
    b.groove(1, snow, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "f#6:8 e6:4 c#6:4 | d6:12 a5:4 | e6:8 c#6:4 a5:4 | b5:12 g#5:4 |
         a5:4 c#6:4 f#6:8 | a6:6 g#6:2 f#6:8 | d6:4 f#6:4 b6:8 | g#6:16",
    );
    b.finish()
}

/// "Kernel": D minor, hammering, a riff in the bass, fire in the lead.
fn stage6() -> Song {
    let mut b = SongB::new("stage6", 4, kit(1.3));
    b.song.res = 0.5;
    let intro = "Dm:16 Dm:16 Bb:16 A:16";
    b.cutoff(0, 700.0).cutoff(63, 2600.0);
    b.groove(0, intro, "KbbbSbbbKbbbSbSS", 2, BASS, DR);
    b.groove(1, intro, "c...c...c...c.c.", 4, ARP, DR);
    b.rest(2, 64);
    b.mark_loop();
    let at = b.sync("riff");
    let riff = "Dm:16 C:16 Bb:16 A:16 Dm:16 C:16 Gm:16 A:16";
    b.cutoff(at, 2400.0);
    b.groove(0, riff, "KobbSobbKobbSobS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "d6:2 d6:2 f6:4 a6:8 | g6:2 g6:2 e6:4 c6:8 | f6:4 d6:4 bb5:4 d6:4 | e6:8 c#6:8 |
         d6:2 d6:2 f6:4 a6:4 d7:4 | c7:4 bb6:4 g6:8 | bb6:4 a6:4 g6:4 f6:4 | e6:12 a5:4",
    );
    b.echo(2, 1, at, 128, 2, ECHO);
    b.finish()
}

/// "The Keep": C minor, grand and dark, the hero's call in a minor key over
/// marching drums.
fn stage7() -> Song {
    let mut b = SongB::new("stage7", 6, kit(1.0));
    b.song.res = 0.6;
    let intro = "Cm:16 Ab:16 Fm:16 G:16";
    b.cutoff(0, 300.0).cutoff(63, 1800.0);
    b.groove(0, intro, "K.......K...S...", 2, BASS, DR);
    b.groove(1, intro, "C...............", 4, ARP, DR);
    b.mel(2, PAD, "g5:16 | eb5:16 | c5:16 | b4:16");
    b.mark_loop();
    let at = b.sync("call");
    let call = "Cm:16 Ab:16 Eb:16 Bb:16 Cm:16 Ab:16 Fm:16 G:16";
    b.cutoff(at, 1700.0).cutoff(at + 127, 2600.0);
    b.groove(0, call, "K.boS.boK.boSSSS", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "c5:4 g5:4 c6:8 | eb6:4 c6:4 ab5:8 | g5:4 bb5:4 eb6:8 | d6:6 c6:2 bb5:8 |
         c5:4 g5:4 c6:8 | eb6:4 f6:4 c6:8 | ab5:4 c6:4 f6:8 | g6:12 d6:4",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    b.finish()
}

/// "Null": E minor, the fastest of all, the bass never resting, the lead
/// broken up like static.
fn final_boss() -> Song {
    let mut b = SongB::new("finalboss", 4, kit(1.4));
    b.song.res = 0.45;
    b.mark_loop();
    let at = b.sync("a");
    let a = "Em:16 Em:16 C:16 B:16 Em:16 Em:16 C:16 D:16";
    b.cutoff(at, 2400.0).cutoff(at + 127, 3400.0);
    b.groove(0, a, "KbobSbobKbobSbSS", 2, BASS, DR);
    b.groove(1, a, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.mel(
        2,
        SAW,
        "e6:2 r:2 e6:2 g6:2 b6:4 e7:4 | d7:2 r:2 b6:2 g6:2 e6:8 | c7:4 b6:4 g6:4 e6:4 | f#6:8 d#6:8 |
         e6:2 g6:2 b6:2 e7:2 g7:4 e7:4 | d7:4 b6:4 g6:8 | a6:4 g6:4 e6:4 c6:4 | d6:8 f#6:8",
    );
    let at = b.sync("b");
    let bb = "Am:16 Em:16 F:16 B:16";
    b.cutoff(at, 1800.0).cutoff(at + 63, 3400.0);
    b.groove(0, bb, "K.b.S.b.KKb.S.SS", 2, BASS, DR);
    b.groove(1, bb, "c...c...c...c...", 4, ARP, DR);
    b.mel(
        2,
        PLUCK,
        "a5 c6 e6 a6 e6 c6 a5 c6 e6 a6 c7 a6 e6 c6 a5 e5 |
         e5 g5 b5 e6 b5 g5 e5 g5 b5 e6 g6 e6 b5 g5 e5 b4 |
         f5 a5 c6 f6 c6 a5 f5 a5 c6 f6 a6 f6 c6 a5 f5 c5 |
         b5 d#6 f#6 b6 f#6 d#6 b5 d#6 f#6 b6 d#7 b6 f#6 d#6 b5 f#5",
    );
    b.finish()
}

/// "The Bells Toll": C minor, fast, the bass hammering under a saw that
/// climbs, and a bell on the downbeat of every phrase.
fn boss() -> Song {
    let mut b = SongB::new("boss", 4, kit(1.3));
    b.song.res = 0.5;
    b.mark_loop();
    let at = b.sync("a");
    let a = "Cm:16 Cm:16 Ab:16 G:16 Cm:16 Cm:16 Ab:16 G:16";
    b.cutoff(at, 2200.0).cutoff(at + 127, 3000.0);
    b.groove(0, a, "KobobSobKobobSoS", 2, BASS, DR);
    b.groove(1, a, "c.c.c.c.c.c.c.c.", 4, ARP, DR);
    b.mel(
        2,
        SAW,
        "c6:4 eb6:4 g6:4 c7:4 | b6:4 g6:4 eb6:4 d6:4 | c6:4 ab6:8 g6:4 | g6:12 b5:4 |
         c6:2 c6:2 eb6:4 g6:4 ab6:4 | g6:4 f6:4 eb6:4 d6:4 | eb6:4 c6:4 ab5:4 c6:4 | b5:16",
    );
    let at = b.sync("b");
    let bb = "Fm:16 Cm:16 Db:16 G:16";
    b.cutoff(at, 1600.0).cutoff(at + 63, 3200.0);
    b.groove(0, bb, "K.b.S.b.KKb.S.SS", 2, BASS, DR);
    b.groove(1, bb, "c...c...c...c...", 4, ARP, DR);
    b.mel(
        2,
        PLUCK,
        "f5 ab5 c6 f6 c6 ab5 f5 ab5 c6 f6 ab6 f6 c6 ab5 f5 c5 |
         c5 eb5 g5 c6 g5 eb5 c5 eb5 g5 c6 eb6 c6 g5 eb5 c5 g4 |
         db5 f5 ab5 db6 ab5 f5 db5 f5 ab5 db6 f6 db6 ab5 f5 db5 ab4 |
         g5 b5 d6 g6 d6 b5 g5 b5 d6 g6 b6 g6 d6 b5 g5 d5",
    );
    b.finish()
}

/// "To Be Continued": D minor turning to F major, then away again.
fn ending() -> Song {
    let mut b = SongB::new("ending", 6, kit(1.0));
    b.song.res = 0.55;
    let intro = "Dm:16 Bb:16 F:16 C:16";
    b.cutoff(0, 400.0).cutoff(63, 1800.0);
    b.groove(0, intro, "b.......o.......", 2, BASS, DR);
    b.groove(1, intro, "c...............", 4, ARP, DR);
    b.mel(2, PAD, "f5:16 | d5:16 | c5:16 | e5:16");
    b.mark_loop();
    let at = b.sync("a");
    let a = "Dm:16 Bb:16 F:16 C:16 Dm:16 Bb:16 Gm:16 A:16";
    b.cutoff(at, 1800.0);
    b.groove(0, a, "K.boS.boK.boS.bo", 2, BASS, DR);
    b.mel(
        2,
        SAW,
        "d5:4 a5:4 d6:8 | f6:4 d6:4 bb5:8 | a5:4 c6:4 f6:8 | e6:6 d6:2 c6:8 |
         d5:4 a5:4 d6:8 | f6:4 g6:4 a6:8 | bb6:4 a6:4 g6:8 | a6:16",
    );
    b.echo(2, 1, at, 128, 3, ECHO);
    b.finish()
}

fn game_over() -> Song {
    let mut b = SongB::new("gameover", 8, kit(0.8));
    b.song.res = 0.6;
    b.song.mode = LP;
    b.cutoff(0, 1600.0).cutoff(80, 400.0);
    let ch = "Am:16 F:16 Dm:16 E:16 Am:32";
    b.groove(0, ch, "b.......b.......", 2, BASS, DR);
    b.groove(1, ch, "c...............", 4, ARP, DR);
    b.mel(
        2,
        LEAD,
        "e5:8 c5:8 | c5:8 a4:8 | d5:8 f5:8 | e5:12 g#4:4 | a4:28 r:4",
    );
    b.finish()
}

fn clear() -> Song {
    let mut b = SongB::new("clear", 5, kit(1.3));
    b.song.res = 0.4;
    b.song.mode = LP | BP;
    b.cutoff(0, 2600.0);
    let ch = "C:8 F:8 G:8 C:24";
    b.groove(0, "C:8 F:8 G:8", "K.o.S.o.", 2, BASS, DR);
    b.groove(0, "C:24", "K...............o.......", 2, BASS, DR);
    b.groove(1, ch, "C.......", 4, ARP, DR);
    b.mel(
        2,
        SAW,
        "c5:2 e5:2 g5:2 c6:2 | a5:2 c6:2 f6:4 | g5:2 b5:2 d6:2 g6:2 | e6:4 c7:16 r:4",
    );
    b.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn songs_are_well_formed() {
        for &id in ALL {
            let s = build(id);
            assert!(s.problems.is_empty(), "{id:?}: {:?}", s.problems);
            assert!(s.rows() >= 32, "{id:?}");
            assert!(s.seconds() > 4.0, "{id:?}");
            assert_eq!(SongId::from_name(id.name()), Some(id));
        }
        assert!(build(SongId::Title).seconds() > 40.0);
        assert!(build(SongId::Title).loop_row.is_some());
        assert!(build(SongId::GameOver).loop_row.is_none());
        let _ = (BP, LP);
    }
}
