//! The NIRUX II logo grid and the window icon. Everything else is drawn by
//! code in the game modules and in `hero.rs`.
use std::sync::OnceLock;

pub struct Art {
    pub logo: Vec<&'static str>,
}

/// The NIRUX II logo letters on a grid; every `#` becomes a block of the
/// logo colours.
const LOGO: &[&str] = &[
    "###...###..#####..########...###...###..###...###....###.###",
    "####..###...###...###...###..###...###...###.###.....###.###",
    "#####.###...###...###...###..###...###....#####......###.###",
    "###.#####...###...########...###...###.....###.......###.###",
    "###..####...###...###.###....###...###....#####......###.###",
    "###...###...###...###..###...###...###...###.###.....###.###",
    "###...###..#####..###...###...#######...###...###....###.###",
];

pub fn get() -> &'static Art {
    static A: OnceLock<Art> = OnceLock::new();
    A.get_or_init(|| Art {
        logo: LOGO.to_vec(),
    })
}

/// The 32x32 window icon: the hero's helmet and scarf, as hex rows.
pub fn icon_rows() -> Vec<String> {
    let s = crate::hero::portrait(7.0, 20, 1.0);
    let mut rows = vec![String::from("................................"); 32];
    let (ox, oy) = (2, 6);
    for y in 0..s.h.min(32 - oy) {
        let mut r: Vec<u8> = rows[(y + oy) as usize].bytes().collect();
        for x in 0..s.w.min(32 - ox) {
            let p = s.px[(y * s.w + x) as usize];
            if p < 16 {
                r[(x + ox) as usize] = b"0123456789abcdef"[p as usize];
            }
        }
        rows[(y + oy) as usize] = String::from_utf8(r).unwrap_or_default();
    }
    rows
}

#[cfg(test)]
mod tests {
    #[test]
    fn logo_and_icon() {
        let a = super::get();
        assert!(a.logo.iter().all(|r| r.len() == a.logo[0].len()));
        let i = super::icon_rows();
        assert_eq!(i.len(), 32);
        assert!(i.iter().all(|r| r.len() == 32));
        assert!(i.iter().any(|r| r.contains('d')), "the visor shows");
    }
}
