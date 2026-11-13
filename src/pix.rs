//! Pictures in half blocks: a canvas of pixels two to a character cell
//! (the upper half drawn in the foreground colour of `▀`, the lower in its
//! background), and the country of the road painted on it from a few
//! shapes. Everything here is drawn in code, so a scene fits any width.

use crate::trail::{Region, Sky};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

pub type Rgb = (u8, u8, u8);

pub struct Canvas {
    pub w: usize,
    pub h: usize,
    px: Vec<Rgb>,
}

/// A mix of two colours, t from 0 (a) to 1 (b).
pub fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
    (m(a.0, b.0), m(a.1, b.1), m(a.2, b.2))
}

/// A small fixed hash, for texture that stays put from frame to frame.
pub fn hash(x: usize, y: usize, salt: u64) -> f64 {
    let mut z = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9) ^ salt.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 29;
    (z >> 11) as f64 / (1u64 << 53) as f64
}

impl Canvas {
    pub fn new(w: usize, h: usize, fill: Rgb) -> Canvas {
        Canvas { w, h, px: vec![fill; w * h] }
    }

    pub fn get(&self, x: usize, y: usize) -> Rgb {
        self.px[y * self.w + x]
    }

    pub fn set(&mut self, x: i32, y: i32, c: Rgb) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.px[y as usize * self.w + x as usize] = c;
        }
    }

    pub fn rect(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgb) {
        for y in y0..y1 {
            for x in x0..x1 {
                self.set(x, y, c);
            }
        }
    }

    /// The sky, from `top` at the top row to `low` at the horizon and below.
    pub fn sky(&mut self, top: Rgb, low: Rgb) {
        let h = self.h.max(2) as f64;
        for y in 0..self.h {
            let c = mix(top, low, y as f64 / (h * 0.6));
            for x in 0..self.w {
                self.px[y * self.w + x] = c;
            }
        }
    }

    /// Fill each column from the height `top(x)` (in pixels from the top,
    /// fractional) to the bottom, shading a little with depth.
    pub fn land(&mut self, top: impl Fn(f64) -> f64, c: Rgb, shade: Rgb) {
        for x in 0..self.w {
            let t = top(x as f64).max(0.0);
            let start = t.round() as usize;
            for y in start..self.h {
                let depth = (y as f64 - t) / (self.h as f64 - t).max(1.0);
                self.px[y * self.w + x] = mix(c, shade, depth);
            }
        }
    }

    /// A filled triangle-ish peak: apex at (x, y), spreading `slope` pixels
    /// sideways per pixel down, down to `base`.
    pub fn peak(&mut self, x: f64, y: f64, slope: f64, base: f64, c: Rgb, snow: Option<Rgb>) {
        let mut row = y;
        while row < base {
            let half = (row - y) * slope;
            for px in (x - half).round() as i32..=(x + half).round() as i32 {
                let col = match snow {
                    Some(s) if row - y < ((base - y) * 0.25).max(1.0) => s,
                    _ => c,
                };
                self.set(px, row as i32, col);
            }
            row += 1.0;
        }
    }

    /// A sprite from rows of letters; `pal` gives each letter its colour,
    /// None for clear.
    pub fn sprite(&mut self, x: i32, y: i32, rows: &[&str], pal: impl Fn(char) -> Option<Rgb>) {
        for (dy, row) in rows.iter().enumerate() {
            for (dx, ch) in row.chars().enumerate() {
                if let Some(c) = pal(ch) {
                    self.set(x + dx as i32, y + dy as i32, c);
                }
            }
        }
    }

    /// The canvas as lines of half blocks, two pixel rows to a line.
    pub fn lines(&self) -> Vec<Line<'static>> {
        let rgb = |c: Rgb| Color::Rgb(c.0, c.1, c.2);
        (0..self.h / 2)
            .map(|row| {
                let spans: Vec<Span> = (0..self.w)
                    .map(|x| {
                        let (top, low) = (self.get(x, row * 2), self.get(x, row * 2 + 1));
                        Span::styled("\u{2580}", Style::new().fg(rgb(top)).bg(rgb(low)))
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }
}

// --- the country ---------------------------------------------------------

const SKY_TOP: Rgb = (70, 120, 200);
const SKY_LOW: Rgb = (180, 210, 235);
const GRASS: Rgb = (78, 150, 62);
const STRAW: Rgb = (176, 158, 88);
const SAGE: Rgb = (138, 146, 104);
const SAND: Rgb = (192, 172, 124);
const PINE: Rgb = (34, 84, 52);
const RIDGE: Rgb = (84, 100, 140);
const SNOW: Rgb = (236, 240, 248);
const WATER: Rgb = (56, 104, 168);
const ROCK: Rgb = (150, 140, 132);
const CLAY: Rgb = (192, 150, 110);
const ADOBE: Rgb = (204, 172, 128);
const LOG: Rgb = (118, 86, 56);
const CANVAS_WHITE: Rgb = (236, 230, 210);
const WOOD: Rgb = (126, 82, 44);
const DARK: Rgb = (40, 32, 28);
const OX: Rgb = (110, 72, 44);

/// What the picture is of: the country, the weather and the season.
#[derive(Clone, Copy)]
pub struct View {
    pub region: Region,
    pub sky: Sky,
    pub month: u32,
}

fn ground_colour(v: View) -> Rgb {
    let base = match v.region {
        Region::Prairie => GRASS,
        Region::HighPlains => STRAW,
        Region::Mountains => SAGE,
        Region::SnakeDesert => SAND,
        Region::BlueMountains => (70, 130, 70),
        Region::Columbia => (60, 132, 66),
    };
    // the grass browns as the summer goes
    let dry = match v.month {
        8 => 0.2,
        9 => 0.35,
        10.. => 0.5,
        _ => 0.0,
    };
    let c = mix(base, STRAW, dry);
    if matches!(v.sky, Sky::Snow | Sky::Blizzard) { mix(c, SNOW, 0.7) } else { c }
}

/// Sky, far country and near ground for a region; returns the horizon row.
fn backdrop(c: &mut Canvas, v: View, salt: u64) -> f64 {
    let (w, h) = (c.w as f64, c.h as f64);
    let (top, low) = match v.sky {
        Sky::Fair => (SKY_TOP, SKY_LOW),
        Sky::Rain => ((96, 104, 120), (150, 156, 166)),
        Sky::Downpour => ((64, 70, 84), (120, 124, 134)),
        Sky::Snow | Sky::Blizzard => ((150, 156, 168), (210, 214, 222)),
    };
    let low = if v.region == Region::SnakeDesert && v.sky == Sky::Fair { (200, 214, 226) } else { low };
    c.sky(top, low);
    let horizon = h * 0.55;
    let s = salt as f64;
    // the far country
    match v.region {
        Region::Prairie => {}
        Region::HighPlains => {
            // low bluffs on the edge of sight
            c.land(|x| horizon - 2.0 - ((x / w * 9.0 + s).sin().max(0.2) * 3.0).round(), mix(CLAY, low, 0.3), mix(CLAY, low, 0.2));
        }
        Region::Mountains => {
            for i in 0..5 {
                let x = w * (0.1 + 0.2 * i as f64) + (hash(i, 1, salt) - 0.5) * w * 0.1;
                let tall = h * (0.18 + hash(i, 2, salt) * 0.15);
                c.peak(x, tall, 1.6, horizon + 1.0, mix(RIDGE, low, 0.1), Some(SNOW));
            }
        }
        Region::SnakeDesert => {
            c.land(|x| horizon - 1.5 - (x / w * 5.0 + s).sin().abs() * 1.5, mix(RIDGE, low, 0.55), mix(RIDGE, low, 0.45));
        }
        Region::BlueMountains => {
            c.land(|x| horizon - 6.0 - (x / w * 7.0 + s).sin() * 3.0 - (x / w * 17.0).sin(), mix(RIDGE, low, 0.2), RIDGE);
        }
        Region::Columbia => {
            // the snow cone of Mount Hood
            c.peak(w * 0.7, h * 0.12, 1.25, horizon + 1.0, mix(RIDGE, low, 0.3), Some(SNOW));
            c.land(|x| horizon - 2.0 - (x / w * 11.0 + s).sin().abs() * 2.0, mix(PINE, low, 0.35), PINE);
        }
    }
    // the near ground, gently rolling
    let g = ground_colour(v);
    let roll = if v.region == Region::Prairie { 1.5 } else { 0.8 };
    c.land(|x| horizon + (x / w * 6.0 + s).sin() * roll, g, mix(g, DARK, 0.25));
    // texture: tufts, sage and stones
    let speck = match v.region {
        Region::SnakeDesert | Region::Mountains => (96, 112, 80),
        Region::HighPlains => (140, 128, 70),
        _ => mix(g, DARK, 0.3),
    };
    for y in (horizon as usize + 2)..c.h {
        for x in 0..c.w {
            if hash(x, y, salt + 7) < 0.06 {
                c.set(x as i32, y as i32, speck);
            }
        }
    }
    // trees on the mountain country
    if matches!(v.region, Region::BlueMountains | Region::Columbia) {
        for i in 0..(c.w / 6) {
            if hash(i, 3, salt) < 0.6 {
                let x = (i * 6) as f64 + hash(i, 4, salt) * 4.0;
                let y = horizon - 1.0 + hash(i, 5, salt) * 3.0;
                c.peak(x, y - 4.0, 0.5, y + 1.0, PINE, None);
            }
        }
    }
    horizon
}

/// Rain or snow falling over the picture.
fn weather(c: &mut Canvas, v: View) {
    let (p, col) = match v.sky {
        Sky::Rain => (0.05, (170, 180, 200)),
        Sky::Downpour => (0.12, (170, 180, 200)),
        Sky::Snow => (0.05, SNOW),
        Sky::Blizzard => (0.16, SNOW),
        Sky::Fair => return,
    };
    for y in 0..c.h {
        for x in 0..c.w {
            if hash(x, y, 99) < p {
                c.set(x as i32, y as i32, col);
            }
        }
    }
}

fn wagon_colour(ch: char) -> Option<Rgb> {
    match ch {
        'c' => Some(CANVAS_WHITE),
        's' => Some((200, 192, 170)),
        'b' => Some(WOOD),
        'w' => Some(DARK),
        'o' => Some(OX),
        'h' => Some((84, 54, 34)),
        't' => Some((90, 60, 34)),
        _ => None,
    }
}

/// A covered wagon and its team, facing west (left), standing on `ground`.
pub fn wagon(c: &mut Canvas, x: i32, ground: i32) {
    let rows = [
        "        sccccs ",
        "       cccccccc",
        " h oo  cccccccc",
        "hhoooo bbbbbbbb",
        "  o o tw  bb  w",
    ];
    c.sprite(x, ground - rows.len() as i32 + 1, &rows, wagon_colour);
}

/// The wagon drawn large, for the title.
pub fn big_wagon(c: &mut Canvas, x: i32, ground: i32) {
    let rows = [
        "               sccccccccccs    ",
        "             ccccccccccccccccc ",
        "            cccccccccccccccccc ",
        "    hh      cccccccccccccccccc ",
        "   hhhoooo   ccccccccccccccccc ",
        "  hhhooooooo bbbbbbbbbbbbbbbbbb",
        "  h  oooooootbbbbbbbbbbbbbbbbbb",
        "     oo   oo  ww         ww    ",
        "     o     o wwww       wwww   ",
        "     o     o  ww         ww    ",
    ];
    c.sprite(x, ground - rows.len() as i32 + 1, &rows, wagon_colour);
}

fn river(c: &mut Canvas, y0: f64, depth: f64, salt: u64) {
    let w = c.w as f64;
    for x in 0..c.w {
        let top = y0 + (x as f64 / w * 4.0 + salt as f64).sin() * 0.8;
        for y in top.round() as i32..(top + depth).round() as i32 {
            let ripple = hash(x, y as usize, salt + 3) < 0.12;
            c.set(x as i32, y, if ripple { (140, 180, 220) } else { WATER });
        }
    }
}

/// A round crown of leaves, for the broadleaf trees of the valleys.
fn blob(c: &mut Canvas, x: f64, y: f64, rx: f64, ry: f64, col: Rgb) {
    for py in (y - ry).floor() as i32..=(y + ry).ceil() as i32 {
        for px in (x - rx).floor() as i32..=(x + rx).ceil() as i32 {
            let (dx, dy) = ((px as f64 - x) / rx, (py as f64 - y) / ry);
            if dx * dx + dy * dy <= 1.0 {
                let shade = if dy < -0.3 { mix(col, (150, 200, 110), 0.3) } else { col };
                c.set(px, py, shade);
            }
        }
    }
}

fn fort(c: &mut Canvas, x: i32, ground: i32, wall: Rgb, flag: bool) {
    let w = (c.w as i32 / 3).max(14);
    c.rect(x, ground - 4, x + w, ground + 1, wall);
    // blockhouses at the corners
    c.rect(x - 1, ground - 6, x + 3, ground + 1, mix(wall, DARK, 0.15));
    c.rect(x + w - 3, ground - 6, x + w + 1, ground + 1, mix(wall, DARK, 0.15));
    // the gate
    c.rect(x + w / 2 - 1, ground - 2, x + w / 2 + 2, ground + 1, DARK);
    if flag {
        c.rect(x + w / 2, ground - 10, x + w / 2 + 1, ground - 4, DARK);
        c.rect(x + w / 2 + 1, ground - 10, x + w / 2 + 5, ground - 8, (200, 40, 40));
    }
}

/// The picture for a landmark, `w` cells wide and `rows` cells high.
pub fn landmark(name: &str, kind: &crate::route::Kind, v: View, w: usize, rows: usize) -> Canvas {
    use crate::route::Kind;
    let mut c = Canvas::new(w, rows * 2, SKY_LOW);
    let salt = name.bytes().map(|b| b as u64).sum::<u64>();
    let horizon = backdrop(&mut c, v, salt);
    let (fw, fh) = (w as f64, (rows * 2) as f64);
    let ground = (fh - 2.0) as i32;
    match name {
        "Independence, Missouri" => {
            for i in 0..5 {
                let x = (fw * 0.15) as i32 + i * 9;
                let tall = 4 + (hash(i as usize, 1, 5) * 4.0) as i32;
                c.rect(x, horizon as i32 - tall, x + 7, horizon as i32 + 2, mix(CLAY, WOOD, hash(i as usize, 2, 5)));
                c.rect(x + 2, horizon as i32 - tall + 2, x + 4, horizon as i32 - tall + 3, DARK);
            }
            wagon(&mut c, (fw * 0.65) as i32, ground);
        }
        "Chimney Rock" => {
            c.peak(fw * 0.5, horizon - fh * 0.2, 2.2, horizon + 1.0, CLAY, None);
            c.rect((fw * 0.5) as i32 - 1, (horizon - fh * 0.5) as i32, (fw * 0.5) as i32 + 2, (horizon - fh * 0.2) as i32 + 1, mix(CLAY, SNOW, 0.15));
        }
        "Scotts Bluff" => {
            let top = horizon - fh * 0.3;
            c.land(|x| if (x / fw * 3.0).fract() < 0.75 { top + (x / fw * 23.0).sin().abs() * 1.5 } else { horizon }, CLAY, mix(CLAY, DARK, 0.25));
            c.land(|x| horizon + (x / fw * 6.0).sin() * 0.8, ground_colour(v), mix(ground_colour(v), DARK, 0.25));
        }
        "Independence Rock" => {
            let (cx, rx, ry) = (fw * 0.5, fw * 0.3, fh * 0.42);
            for x in 0..w {
                let d = (x as f64 - cx) / rx;
                if d.abs() < 1.0 {
                    let top = horizon + 1.0 - ry * (1.0 - d * d).sqrt();
                    for y in top as i32..=horizon as i32 + 1 {
                        c.set(x as i32, y, mix(ROCK, DARK, (y as f64 - top) / ry * 0.4));
                    }
                }
            }
            river(&mut c, horizon + 3.0, 2.0, salt);
        }
        "South Pass" => {
            c.land(|x| horizon - 3.0 - ((x / fw - 0.5).abs() * 2.0).powi(2) * fh * 0.3, mix(SAGE, RIDGE, 0.3), SAGE);
            c.land(|x| horizon + (x / fw * 6.0).sin() * 0.8, ground_colour(v), mix(ground_colour(v), DARK, 0.25));
            wagon(&mut c, (fw * 0.45) as i32, horizon as i32 + 2);
        }
        "Ash Hollow" => {
            c.land(|x| if x < fw * 0.4 { horizon - fh * 0.35 + x / fw * fh * 0.85 } else { fh }, STRAW, mix(STRAW, DARK, 0.3));
            for i in 0..6 {
                let x = fw * 0.45 + i as f64 * fw * 0.08;
                blob(&mut c, x, horizon - 2.0, 3.0, 2.5, (66, 118, 52));
                c.rect(x as i32, (horizon - 1.0) as i32, x as i32 + 1, horizon as i32 + 2, WOOD);
            }
            river(&mut c, fh - 4.0, 2.0, salt);
        }
        "Soda Springs" => {
            for i in 0..4 {
                let x = (fw * (0.25 + 0.17 * i as f64)) as i32;
                let y = horizon as i32 + 2 + (i % 2);
                c.rect(x - 2, y, x + 3, y + 1, (200, 200, 190));
                for k in 0..3 {
                    c.set(x + (k % 2), y - 2 - k * 2, SNOW);
                }
            }
        }
        "Parting of the Ways" => {
            for y in horizon as i32 + 1..ground + 2 {
                let t = (y as f64 - horizon) / (fh - horizon);
                let mid = fw * 0.5;
                for dx in 0..2 {
                    c.set((mid - t * fw * 0.35) as i32 + dx, y, (196, 170, 120));
                    c.set((mid + t * fw * 0.15) as i32 + dx, y, (196, 170, 120));
                }
            }
        }
        "Fort Kearny" => {
            c.rect((fw * 0.2) as i32, horizon as i32 - 1, (fw * 0.55) as i32, horizon as i32 + 2, (110, 90, 60));
            for i in 0..4 {
                c.peak(fw * 0.62 + i as f64 * 5.0, horizon - 3.0, 1.0, horizon + 2.0, CANVAS_WHITE, None);
            }
        }
        "Laurel Hill" => {
            // the slope falls away westward, to the left
            let slope = |x: f64| horizon + fh * 0.4 - x / fw * fh * 0.7;
            c.land(slope, (60, 120, 60), PINE);
            for i in 0..5 {
                let x = fw * (0.05 + i as f64 * 0.09);
                let y = slope(x);
                c.peak(x, y - 6.0, 0.5, y + 1.0, PINE, None);
            }
            let wx = fw * 0.62;
            wagon(&mut c, wx as i32, slope(wx + 7.0) as i32);
            // the snubbing tree and its rope
            c.peak(fw * 0.85, slope(fw * 0.85) - 7.0, 0.5, slope(fw * 0.85) + 1.0, PINE, None);
            for x in (wx as i32 + 14)..(fw * 0.85) as i32 {
                c.set(x, slope(x as f64) as i32 - 2, (200, 180, 140));
            }
        }
        "Grande Ronde Valley" => {
            c.land(|x| horizon + 1.0 + (x / fw * 3.0).sin() * 0.5, (84, 160, 70), (60, 130, 56));
        }
        "Oregon City" => {
            river(&mut c, horizon + 1.0, 3.0, salt);
            // the falls: a ledge of white water across the river
            for x in (fw * 0.28) as i32..(fw * 0.46) as i32 {
                let col = if x % 2 == 0 { (225, 235, 245) } else { (170, 200, 230) };
                c.rect(x, horizon as i32 + 1, x + 1, horizon as i32 + 4, col);
            }
            for i in 0..4 {
                let x = (fw * 0.6) as i32 + i * 6;
                c.rect(x, horizon as i32 - 3, x + 4, horizon as i32 + 1, LOG);
            }
            c.land(|x| fh - 3.0 + (x / fw * 5.0).sin(), (90, 170, 70), (60, 140, 56));
        }
        _ => match kind {
            Kind::River { .. } => {
                river(&mut c, horizon + 2.0, (fh - horizon - 4.0).max(2.0), salt);
                if name == "Three Island Crossing" {
                    for i in 0..3 {
                        let x = (fw * (0.2 + 0.25 * i as f64)) as i32;
                        c.rect(x, horizon as i32 + 4, x + 7, horizon as i32 + 6, SAND);
                    }
                }
            }
            Kind::Fort(_) => {
                let (wall, flag) = match name {
                    "Fort Bridger" => (LOG, false),
                    "Fort Vancouver" => (LOG, true),
                    "Fort Hall" => ((226, 218, 200), true),
                    _ => (ADOBE, true),
                };
                fort(&mut c, (fw * 0.3) as i32, horizon as i32 + 2, wall, flag);
                if name == "Fort Vancouver" {
                    river(&mut c, fh - 4.0, 3.0, salt);
                }
            }
            Kind::Fork => {
                // The Dalles: the Columbia under basalt cliffs
                // the rimrock: dark basalt in columns along the far bank
                c.land(|x| horizon - 1.0 - (x / fw * 13.0).sin().abs() * 2.0, (84, 74, 66), (60, 52, 48));
                for x in (0..w).step_by(3) {
                    for y in (horizon - 3.0) as i32..(horizon + 2.0) as i32 {
                        if c.get(x, y.max(0) as usize) == (84, 74, 66) {
                            c.set(x as i32, y, (60, 52, 48));
                        }
                    }
                }
                river(&mut c, horizon + 2.0, (fh - horizon - 3.0).max(2.0), salt);
            }
            _ => wagon(&mut c, (fw * 0.45) as i32, ground),
        },
    }
    weather(&mut c, v);
    c
}

/// The road: the country the wagon is crossing, with the wagon in it,
/// `along` from 0 (just left the last landmark) to 1 (at the next).
pub fn road(v: View, w: usize, rows: usize, along: f64, salt: u64) -> Canvas {
    let mut c = Canvas::new(w, rows * 2, SKY_LOW);
    let horizon = backdrop(&mut c, v, salt);
    // the wagon goes west, right to left
    let x = ((1.0 - along.clamp(0.0, 1.0)) * (w as f64 - 16.0)) as i32 + 1;
    let ground = (horizon + 2.0).min(c.h as f64 - 1.0) as i32;
    wagon(&mut c, x, ground);
    weather(&mut c, v);
    c
}

/// The title picture: a wagon and its team on the prairie at evening.
pub fn title(w: usize, rows: usize) -> Canvas {
    let mut c = Canvas::new(w, rows * 2, SKY_LOW);
    c.sky((60, 60, 120), (240, 160, 90));
    let (fw, fh) = (w as f64, (rows * 2) as f64);
    // the sun going down in the west
    let (sx, sy) = (fw * 0.2, fh * 0.55);
    for y in 0..c.h {
        for x in 0..c.w {
            let d = ((x as f64 - sx) / 2.0).powi(2) + (y as f64 - sy).powi(2);
            if d < 9.0 {
                c.set(x as i32, y as i32, (255, 220, 140));
            }
        }
    }
    let horizon = fh * 0.62;
    c.land(|x| horizon + (x / fw * 5.0).sin() * 0.8, (64, 96, 52), (40, 60, 34));
    if rows >= 6 {
        big_wagon(&mut c, (fw * 0.5) as i32, (fh - 2.0) as i32);
    } else {
        wagon(&mut c, (fw * 0.55) as i32, (fh - 2.0) as i32);
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::ROUTE;

    #[test]
    fn every_landmark_has_a_picture_of_its_size() {
        for l in &ROUTE {
            for sky in [Sky::Fair, Sky::Rain, Sky::Snow] {
                let c = landmark(l.name, &l.kind, View { region: l.region, sky, month: 7 }, 70, 6);
                assert_eq!(c.lines().len(), 6, "{}", l.name);
                assert!(c.lines().iter().all(|line| line.spans.len() == 70));
            }
        }
        let r = road(View { region: Region::Mountains, sky: Sky::Fair, month: 7 }, 40, 3, 0.5, 1);
        assert_eq!(r.lines().len(), 3);
        // tiny sizes do not panic
        landmark("Chimney Rock", &ROUTE[6].kind, View { region: Region::HighPlains, sky: Sky::Fair, month: 5 }, 4, 1);
        road(View { region: Region::Columbia, sky: Sky::Blizzard, month: 11 }, 3, 1, 1.0, 0);
        title(5, 1);
    }
}
