//! The game: the title screen and menus, the prologue, the stage intros,
//! playing a stage, pause, stage clear, game over, high score entry and the
//! attract mode, as one state machine stepped 60 times a second.
pub mod boss;
pub mod common;
pub mod foes;
pub mod screens;
pub mod stage;
pub mod stages;
pub mod world;

use crate::audio::Cmd;
use crate::config::{Config, Difficulty, Save, Score};
use crate::fb::Fb;
use crate::input::Input;
use crate::palette::*;
use crate::sfx::Sfx;
use crate::songs::SongId;
use common::{Out, Stats, Status};

/// The seven stages of the Undercroft.
pub const LEVELS: usize = 7;

pub trait Level {
    fn update(&mut self, inp: &Input, st: &mut Stats, out: &mut Out) -> Status;
    fn draw(&self, fb: &mut Fb, st: &Stats, hi: u32);
    /// An autopilot for the attract mode and the self test.
    fn bot(&mut self, st: &Stats) -> Input;
    fn border(&self) -> u8;
    fn progress(&self) -> f32;
    /// A line about the state, for the self test.
    fn info(&self) -> String {
        String::new()
    }
}

pub struct Info {
    pub name: &'static str,
    pub mode: &'static str,
    /// ECHO's briefing; the signal grows weaker the deeper NIRUX goes.
    pub brief: [&'static str; 2],
    pub tip: [&'static str; 2],
    pub border: u8,
    pub song: SongId,
}

pub const INFO: [Info; LEVELS] = [
    Info {
        name: "PROCESS GRAVEYARD",
        mode: "WHERE THE TERMINATED LIE",
        brief: [
            "THE TERMINATED WALK AGAIN HERE.",
            "I AM STILL WITH YOU. FOR NOW.",
        ],
        tip: [
            "JUMP: SPACE, A   FIRE: CTRL, X, SQUARE",
            "HOLD UP AND FIRE TO SHOOT UPWARDS",
        ],
        border: BLUE,
        song: SongId::Stage1,
    },
    Info {
        name: "SWAP MARSH",
        mode: "WHERE THE FORGOTTEN SINK",
        brief: [
            "MY SIGNAL IS WEAKER DOWN HERE.",
            "THE LAMP SHOWS WHAT HIDES.",
        ],
        tip: [
            "LAMP: SHIFT, C OR TRIANGLE",
            "THE WATER TAKES A PIECE OF YOUR SCARF",
        ],
        border: GREEN,
        song: SongId::Stage2,
    },
    Info {
        name: "ARCHIVE CATHEDRAL",
        mode: "WHERE THE DELETED SING",
        brief: [
            "CLIMB, NIRUX. IT IS ALL STAIRS.",
            "THE CHOIR WAITS AT THE TOP.",
        ],
        tip: ["GHOSTS PASS THROUGH STONE", "THE LAMP HOLDS THEM STILL"],
        border: PURPLE,
        song: SongId::Stage3,
    },
    Info {
        name: "CRON TOWER",
        mode: "WHERE THE BELLS ARE WOUND",
        brief: ["THE LEDGES KEEP TIME. SO MUST", "YOU. WAIT FOR THE TICK."],
        tip: ["CLOCKWORK LEDGES FLICKER", "BEFORE THEY GO"],
        border: ORANGE,
        song: SongId::Stage4,
    },
    Info {
        name: "FROZEN CACHE",
        mode: "WHERE THE OLD DATA SLEEPS",
        brief: [
            "THE ICE... SLIPPERY. ICICLES.",
            "I AM... FADING. KEEP GOING.",
        ],
        tip: ["ON ICE YOU SLIDE: STOP EARLY", "ICICLES FALL WHEN YOU PASS"],
        border: LBLUE,
        song: SongId::Stage5,
    },
    Info {
        name: "KERNEL CRYPT",
        mode: "WHERE THE DAEMONS WAKE",
        brief: [
            "LAVA... DAEMONS... SOMEONE ELSE",
            "IS HERE. I CANNOT SEE WHO.",
        ],
        tip: [
            "LAVA BURNS A PIECE OF THE SCARF",
            "THE HAMMER GOES OVER SHIELDS",
        ],
        border: RED,
        song: SongId::Stage6,
    },
    Info {
        name: "NULL'S KEEP",
        mode: "WHERE NOTHING RETURNS",
        brief: ["##... NIRUX... ##... END IT.", "##... COME HOME."],
        tip: [
            "EVERYTHING YOU HAVE LEARNED",
            "NULL NEVER SHOWS ITS WHOLE SELF",
        ],
        border: PURPLE,
        song: SongId::Stage7,
    },
];

pub fn make_level(n: usize, d: Difficulty, seed: u32) -> Box<dyn Level> {
    use stage::{Spec, Stage};
    use world::Theme;
    let (theme, rain, fog, border, built) = match n {
        0 => (Theme::Graveyard, 60, false, BLUE, stages::graveyard()),
        1 => (Theme::Marsh, 18, true, GREEN, stages::marsh()),
        2 => (Theme::Cathedral, 0, false, PURPLE, stages::cathedral()),
        3 => (Theme::Tower, 30, false, ORANGE, stages::tower()),
        4 => (Theme::Ice, 0, true, LBLUE, stages::ice()),
        5 => (Theme::Crypt, 0, false, RED, stages::crypt()),
        _ => (Theme::Keep, 90, false, PURPLE, stages::keep()),
    };
    Box::new(Stage::new(
        Spec {
            theme,
            rain,
            fog,
            border,
            hud: border,
        },
        built,
        d,
        seed,
        n,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scr {
    Title,
    Select,
    Options,
    Scores,
    Intro,
    Play,
    Pause,
    Clear,
    Over,
    Entry,
    Victory,
    Demo,
    Controls,
    /// The prologue: how NULL took the Core and the Core woke NIRUX.
    Story,
}

pub struct Game {
    pub scr: Scr,
    pub t: u32,
    pub sel: usize,
    pub cfg: Config,
    pub save: Save,
    pub level: Option<Box<dyn Level>>,
    pub lvl: usize,
    pub stats: Stats,
    pub diff: Difficulty,
    pub cmds: Vec<Cmd>,
    pub quit: bool,
    pub fullscreen_req: Option<bool>,
    pub cfg_dirty: bool,
    pub idle: u32,
    pub entry: [u8; 3],
    pub entry_pos: usize,
    pub tally: (u32, u32, u32),
    pub tally_shown: u32,
    pub demo_left: u32,
    pub frames: u64,
    pub seed: u32,
    pub persist: bool,
    pub last_place: Option<usize>,
    /// Where the controls screen goes back to.
    pub controls_back: Scr,
    pub pad: crate::pad::Status,
    /// The prologue's page, and whether it plays as part of the attract
    /// loop (then it goes on to the high scores, not into a game).
    pub page: usize,
    pub page_t: u32,
    pub story_attract: bool,
    /// NIRUX (the first game) has been beaten here: welcome back.
    pub veteran: bool,
}

impl Game {
    pub fn new(cfg: Config, save: Save, persist: bool) -> Game {
        let diff = cfg.difficulty;
        let mut g = Game {
            scr: Scr::Title,
            t: 0,
            sel: 0,
            cfg,
            save,
            level: None,
            lvl: 0,
            stats: Stats::new(3),
            diff,
            cmds: Vec::new(),
            quit: false,
            fullscreen_req: None,
            cfg_dirty: false,
            idle: 0,
            entry: [b'A'; 3],
            entry_pos: 0,
            tally: (0, 0, 0),
            tally_shown: 0,
            demo_left: 0,
            frames: 0,
            seed: 0x1234_5678,
            persist,
            last_place: None,
            controls_back: Scr::Title,
            pad: crate::pad::Status::None,
            page: 0,
            page_t: 0,
            story_attract: false,
            veteran: false,
        };
        g.go(Scr::Title);
        g
    }

    fn music(&mut self, s: Option<SongId>) {
        self.cmds.push(Cmd::Music(s));
    }

    fn sfx(&mut self, s: Sfx) {
        self.cmds.push(Cmd::Sfx(s));
    }

    pub fn go(&mut self, s: Scr) {
        self.scr = s;
        self.t = 0;
        match s {
            Scr::Title => {
                self.music(Some(SongId::Title));
                self.sel = 0;
                self.idle = 0;
                self.level = None;
            }
            Scr::Select => self.sel = 0,
            Scr::Options => self.sel = 0,
            Scr::Intro => self.music(Some(INFO[self.lvl].song)),
            Scr::Clear => self.music(Some(SongId::Clear)),
            Scr::Over => self.music(Some(SongId::GameOver)),
            Scr::Victory => self.music(Some(SongId::Ending)),
            Scr::Story => {
                self.music(Some(SongId::Legend));
                self.page = 0;
                self.page_t = 0;
            }
            _ => {}
        }
    }

    fn next_seed(&mut self) -> u32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223)
            ^ self.frames as u32;
        self.seed | 1
    }

    pub fn start_run(&mut self, lvl: usize) {
        self.diff = self.cfg.difficulty;
        self.stats = Stats::new(self.diff.lives() + self.veteran as i32);
        self.lvl = lvl;
        self.go(Scr::Intro);
    }

    fn start_level(&mut self) {
        let seed = self.next_seed();
        self.level = Some(make_level(self.lvl, self.diff, seed));
        self.stats.kills = 0;
        self.stats.deaths_this_level = 0;
        self.go(Scr::Play);
    }

    fn start_demo(&mut self) {
        let seed = self.next_seed();
        let lvl = (seed >> 5) as usize % LEVELS;
        self.lvl = lvl;
        self.diff = Difficulty::Easy;
        self.stats = Stats::new(9);
        self.stats.weapon = [
            common::Weapon::Bolt,
            common::Weapon::Hammer,
            common::Weapon::Disc,
            common::Weapon::Whip,
        ][(seed >> 9) as usize % 4];
        self.level = Some(make_level(lvl, Difficulty::Easy, seed));
        self.demo_left = 60 * 40;
        self.scr = Scr::Demo;
        self.t = 0;
    }

    pub fn start_attract_demo(&mut self) {
        self.start_demo();
    }

    pub fn hi(&self) -> u32 {
        self.save.scores.first().map(|s| s.score).unwrap_or(0)
    }

    /// The border colour this frame.
    pub fn border(&self) -> u8 {
        match self.scr {
            Scr::Play | Scr::Demo | Scr::Pause => self
                .level
                .as_ref()
                .map(|l| l.border())
                .unwrap_or(INFO[self.lvl].border),
            Scr::Intro | Scr::Clear => INFO[self.lvl].border,
            Scr::Over => RED,
            Scr::Victory | Scr::Entry => PURPLE,
            Scr::Story => BLACK,
            _ => BLUE,
        }
    }

    fn apply_out(&mut self, out: Out) {
        for s in out.sfx {
            self.cmds.push(Cmd::Sfx(s));
        }
        if let Some(m) = out.music {
            self.cmds.push(Cmd::Music(Some(m)));
        }
    }

    fn menu_nav(&mut self, inp: &Input, n: usize) {
        if inp.p_up {
            self.sel = (self.sel + n - 1) % n;
            self.sfx(Sfx::MenuMove);
        }
        if inp.p_down {
            self.sel = (self.sel + 1) % n;
            self.sfx(Sfx::MenuMove);
        }
    }

    pub fn update(&mut self, inp: Input) {
        self.t = self.t.wrapping_add(1);
        self.frames += 1;
        let ok = inp.p_fire || inp.p_start || inp.p_jump;
        match self.scr {
            Scr::Title => {
                if inp.any_pressed() || inp.p_up || inp.p_down || inp.p_left || inp.p_right {
                    self.idle = 0;
                } else {
                    self.idle += 1;
                }
                self.menu_nav(&inp, 7);
                if self.sel == 2 && (inp.p_left || inp.p_right) {
                    self.cfg.difficulty = if inp.p_left {
                        self.cfg.difficulty.prev()
                    } else {
                        self.cfg.difficulty.next()
                    };
                    self.cfg_dirty = true;
                    self.sfx(Sfx::MenuMove);
                }
                if ok {
                    self.sfx(Sfx::MenuSelect);
                    match self.sel {
                        0 => {
                            self.start_run(0);
                            self.story_attract = false;
                            self.go(Scr::Story);
                        }
                        1 => self.go(Scr::Select),
                        2 => {
                            self.cfg.difficulty = self.cfg.difficulty.next();
                            self.cfg_dirty = true;
                        }
                        3 => {
                            self.controls_back = Scr::Title;
                            self.go(Scr::Controls);
                        }
                        4 => self.go(Scr::Options),
                        5 => self.go(Scr::Scores),
                        _ => self.quit = true,
                    }
                } else if inp.p_back {
                    self.sel = 6;
                }
                if self.idle > 60 * 22 {
                    self.story_attract = true;
                    self.go(Scr::Story);
                    self.idle = 1;
                }
            }
            Scr::Select => {
                self.menu_nav(&inp, LEVELS + 1);
                let reached = self.save.best_reached() as usize;
                if ok {
                    if self.sel == LEVELS {
                        self.go(Scr::Title);
                    } else if self.sel < reached {
                        self.sfx(Sfx::MenuSelect);
                        self.start_run(self.sel);
                    } else {
                        self.sfx(Sfx::Clang);
                    }
                } else if inp.p_back {
                    self.go(Scr::Title);
                }
            }
            Scr::Options => {
                self.menu_nav(&inp, 6);
                let dir = if inp.p_left {
                    -1
                } else if inp.p_right || ok {
                    1
                } else {
                    0
                };
                if dir != 0 {
                    let c = &mut self.cfg;
                    match self.sel {
                        0 => c.crt = !c.crt,
                        1 => {
                            c.fullscreen = !c.fullscreen;
                            self.fullscreen_req = Some(c.fullscreen);
                        }
                        2 => c.music = (c.music as i32 + dir).clamp(0, 10) as u8,
                        3 => c.sfx = (c.sfx as i32 + dir).clamp(0, 10) as u8,
                        4 => c.echo = !c.echo,
                        _ => {}
                    }
                    if self.sel == 5 && ok {
                        self.go(Scr::Title);
                    } else {
                        self.cfg_dirty = true;
                        let (m, s, e) = (self.cfg.music, self.cfg.sfx, self.cfg.echo);
                        self.cmds.push(Cmd::Volume {
                            music: m as f32 / 10.0,
                            sfx: s as f32 / 10.0,
                        });
                        self.cmds.push(Cmd::Echo(e));
                        self.sfx(Sfx::MenuMove);
                    }
                }
                if inp.p_back {
                    self.go(Scr::Title);
                }
            }
            Scr::Scores => {
                if ok || inp.p_back {
                    self.go(Scr::Title);
                } else if self.idle > 0 && self.t > 60 * 9 {
                    self.start_demo();
                }
            }
            Scr::Story => {
                self.page_t += 1;
                let full = screens::story_page_len(self.page) as u32 * 2 + 30;
                let attract_key = self.story_attract
                    && (inp.any_pressed() || inp.p_up || inp.p_down || inp.p_left || inp.p_right);
                if attract_key {
                    self.go(Scr::Title);
                } else if inp.p_back {
                    self.end_story();
                } else if ok && self.page_t < full {
                    self.page_t = full;
                } else if (ok && self.page_t > 12) || self.page_t > full + 300 {
                    self.page += 1;
                    self.page_t = 0;
                    self.sfx(if self.page == 2 {
                        Sfx::Bell
                    } else {
                        Sfx::MenuMove
                    });
                    if self.page >= screens::STORY_PAGES {
                        self.end_story();
                    }
                }
            }
            Scr::Intro => {
                if self.t > 270 || (self.t > 40 && ok) {
                    self.start_level();
                }
            }
            Scr::Play => {
                if inp.p_back || inp.p_start {
                    self.scr = Scr::Pause;
                    self.sel = 0;
                    self.sfx(Sfx::MenuMove);
                    return;
                }
                let mut out = Out::default();
                let status = match self.level.as_mut() {
                    Some(l) => l.update(&inp, &mut self.stats, &mut out),
                    None => Status::Over,
                };
                self.apply_out(out);
                match status {
                    Status::Playing => {}
                    Status::Cleared => {
                        let kills = self.stats.kills;
                        let bonus = 5000 * (self.lvl as u32 + 1) + kills * 20;
                        let nodeath = if self.stats.deaths_this_level == 0 {
                            10000 * (self.lvl as u32 + 1)
                        } else {
                            0
                        };
                        self.tally = (kills, bonus, nodeath);
                        self.tally_shown = 0;
                        self.stats.add(bonus + nodeath);
                        let next = (self.lvl + 2).min(LEVELS) as u8;
                        self.save.reach(self.diff, next);
                        if self.lvl + 1 >= LEVELS {
                            self.save.cleared = true;
                        }
                        if self.persist {
                            self.save.save();
                        }
                        self.go(Scr::Clear);
                    }
                    Status::Over => self.go(Scr::Over),
                }
            }
            Scr::Demo => {
                if inp.any_pressed() || inp.p_up || inp.p_down || inp.p_left || inp.p_right {
                    self.go(Scr::Title);
                    return;
                }
                let mut out = Out::default();
                let mut status = Status::Playing;
                if let Some(l) = self.level.as_mut() {
                    let bi = l.bot(&self.stats);
                    status = l.update(&bi, &mut self.stats, &mut out);
                }
                self.apply_out(out);
                self.demo_left = self.demo_left.saturating_sub(1);
                if self.demo_left == 0 || status != Status::Playing {
                    self.go(Scr::Title);
                }
            }
            Scr::Pause => {
                self.menu_nav(&inp, 3);
                if inp.p_back {
                    self.scr = Scr::Play;
                } else if ok {
                    match self.sel {
                        0 => self.scr = Scr::Play,
                        1 => {
                            self.controls_back = Scr::Pause;
                            self.scr = Scr::Controls;
                            self.t = 0;
                        }
                        _ => self.go(Scr::Title),
                    }
                }
            }
            Scr::Controls => {
                if (ok || inp.p_back) && self.t > 10 {
                    self.sfx(Sfx::MenuSelect);
                    if self.controls_back == Scr::Pause {
                        self.scr = Scr::Pause;
                        self.sel = 1;
                    } else {
                        self.go(Scr::Title);
                        self.sel = 3;
                    }
                }
            }
            Scr::Clear => {
                let total = self.tally.1 + self.tally.2;
                if self.t > 60 && self.tally_shown < total {
                    let step = (total / 60).max(250);
                    self.tally_shown = (self.tally_shown + step).min(total);
                    if self.t % 3 == 0 {
                        self.sfx(Sfx::Tally);
                    }
                }
                if self.t > 150 && ok {
                    if self.lvl + 1 < LEVELS {
                        self.lvl += 1;
                        self.go(Scr::Intro);
                    } else {
                        self.go(Scr::Victory);
                    }
                }
            }
            Scr::Over => {
                if self.t > 420 || (self.t > 90 && ok) {
                    self.after_run();
                }
            }
            Scr::Victory => {
                let end = 600 + screens::credits_stop() as u32 * 2 + 600;
                if (self.t > 180 && ok) || self.t > end {
                    self.after_run();
                }
            }
            Scr::Entry => {
                let e = &mut self.entry[self.entry_pos.min(2)];
                const SET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .";
                let i = SET.iter().position(|c| c == e).unwrap_or(0);
                if inp.p_up {
                    *e = SET[(i + 1) % SET.len()];
                    self.cmds.push(Cmd::Sfx(Sfx::MenuMove));
                }
                if inp.p_down {
                    *e = SET[(i + SET.len() - 1) % SET.len()];
                    self.cmds.push(Cmd::Sfx(Sfx::MenuMove));
                }
                if inp.p_left && self.entry_pos > 0 {
                    self.entry_pos -= 1;
                }
                if inp.p_right && self.entry_pos < 2 {
                    self.entry_pos += 1;
                }
                if ok || inp.p_start {
                    self.sfx(Sfx::MenuSelect);
                    if self.entry_pos < 2 && !inp.p_start {
                        self.entry_pos += 1;
                    } else {
                        let name: String = self.entry.iter().map(|&c| c as char).collect();
                        let place = self.save.insert(Score {
                            name,
                            score: self.stats.score,
                            sector: self.lvl as u8 + 1,
                            difficulty: self.diff,
                        });
                        self.last_place = Some(place);
                        if self.persist {
                            self.save.save();
                        }
                        self.go(Scr::Scores);
                    }
                }
            }
        }
    }

    fn end_story(&mut self) {
        if self.story_attract {
            self.story_attract = false;
            self.go(Scr::Scores);
        } else {
            self.go(Scr::Intro);
        }
    }

    /// The autopilot's input for the level being played (for --snap).
    pub fn bot_input(&mut self) -> Input {
        match self.level.as_mut() {
            Some(l) => l.bot(&self.stats),
            None => Input::default(),
        }
    }

    fn after_run(&mut self) {
        if self.save.qualifies(self.stats.score) {
            self.entry_pos = 0;
            self.go(Scr::Entry);
            self.music(Some(SongId::Title));
        } else {
            self.go(Scr::Title);
        }
    }

    pub fn draw(&self, fb: &mut Fb) {
        screens::draw(self, fb);
    }

    pub fn take_cmds(&mut self) -> Vec<Cmd> {
        std::mem::take(&mut self.cmds)
    }
}

#[cfg(test)]
mod story_tests {
    use super::*;

    #[test]
    fn the_prologue_plays_before_sector_one_and_in_the_attract_loop() {
        let mut g = Game::new(Config::default(), Save::default(), false);
        // Left alone, the title shows the prologue, then the high scores.
        for _ in 0..60 * 23 {
            g.update(Input::default());
        }
        assert_eq!(g.scr, Scr::Story);
        assert!(g.story_attract);
        for _ in 0..60 * 120 {
            g.update(Input::default());
            if g.scr != Scr::Story {
                break;
            }
        }
        assert_eq!(g.scr, Scr::Scores);
        // Start game: the prologue, and Esc skips to the sector intro.
        g.go(Scr::Title);
        g.update(Input {
            p_fire: true,
            ..Default::default()
        });
        assert_eq!(g.scr, Scr::Story);
        assert!(!g.story_attract);
        g.update(Input {
            p_back: true,
            ..Default::default()
        });
        assert_eq!((g.scr, g.lvl), (Scr::Intro, 0));
    }
}
