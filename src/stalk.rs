//! Stalking game on foot: the hunting scene played by hand. A stretch of
//! country seen from above, game grazing in it, the wind, and the day's
//! light running out. Creep up from downwind and through cover, fire when
//! close enough, and carry back what the party can.
//!
//! The scene runs in ticks (five a second in the terminal); the animals
//! graze, catch the hunter's scent or sight, and run. Buffalo, as the
//! emigrants wrote, often stood while their neighbours fell; antelope have
//! the sharpest eyes on the plains.

use crate::date;
use crate::rng::Rng;
use crate::trail::Region;

/// The field, in cells.
pub const W: i32 = 64;
pub const H: i32 = 12;
/// Ticks in a day's hunting: about a minute and a quarter in the terminal.
pub const DAY: u32 = 380;
/// Ticks a second, for the terminal loop.
pub const TICKS_PER_SECOND: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ground {
    Grass,
    /// brush, sage or tall grass: hides a crawling hunter
    Brush,
    Rock,
    Tree,
    Water,
}

impl Ground {
    pub fn cover(self) -> bool {
        matches!(self, Ground::Brush | Ground::Rock | Ground::Tree)
    }
}

#[derive(Debug, PartialEq)]
pub struct Kind {
    pub name: &'static str,
    /// the name for more than one
    pub plural: &'static str,
    pub glyph: char,
    /// pounds of meat
    pub meat: u32,
    /// the chance a shot at close range brings one down
    pub hit: f64,
    /// cells it sees a moving hunter in the open at
    pub sight: f64,
    /// how much a shot nearby alarms it (buffalo often stood)
    pub startle: f64,
    /// ticks between steps when running
    pub run: u32,
    /// how many go together
    pub herd: (i64, i64),
}

pub const BUFFALO: Kind = Kind { name: "buffalo", plural: "buffalo", glyph: 'B', meat: 450, hit: 0.6, sight: 5.0, startle: 4.0, run: 2, herd: (4, 8) };
pub const ANTELOPE: Kind = Kind { name: "antelope", plural: "antelope", glyph: 'a', meat: 45, hit: 0.45, sight: 10.0, startle: 20.0, run: 1, herd: (3, 6) };
pub const DEER: Kind = Kind { name: "deer", plural: "deer", glyph: 'd', meat: 70, hit: 0.55, sight: 7.0, startle: 20.0, run: 1, herd: (1, 3) };
pub const ELK: Kind = Kind { name: "elk", plural: "elk", glyph: 'E', meat: 250, hit: 0.5, sight: 7.0, startle: 12.0, run: 2, herd: (2, 5) };
pub const TURKEY: Kind = Kind { name: "wild turkey", plural: "wild turkeys", glyph: 't', meat: 12, hit: 0.65, sight: 6.0, startle: 20.0, run: 2, herd: (2, 4) };
pub const RABBIT: Kind = Kind { name: "rabbit", plural: "rabbits", glyph: 'r', meat: 2, hit: 0.7, sight: 4.0, startle: 20.0, run: 1, herd: (2, 4) };
pub const SAGE_HEN: Kind = Kind { name: "sage hen", plural: "sage hens", glyph: 'h', meat: 3, hit: 0.75, sight: 4.0, startle: 20.0, run: 1, herd: (2, 5) };

/// What is about, by country and season, with weights.
fn about(region: Region, day: u32) -> Vec<(&'static Kind, f64)> {
    let m = date::month(day);
    let herds = (5..=8).contains(&m);
    match region {
        Region::Prairie => vec![(&DEER, 2.0), (&TURKEY, 2.0), (&RABBIT, 1.5), (&BUFFALO, if herds { 0.4 } else { 0.0 })],
        Region::HighPlains => vec![(&BUFFALO, if herds { 3.0 } else { 0.8 }), (&ANTELOPE, 3.0), (&RABBIT, 1.0)],
        Region::Mountains => vec![(&ELK, 1.5), (&DEER, 2.0), (&ANTELOPE, 2.0), (&SAGE_HEN, 1.5)],
        Region::SnakeDesert => vec![(&RABBIT, 3.0), (&SAGE_HEN, 3.0), (&ANTELOPE, 0.7)],
        Region::BlueMountains => vec![(&ELK, 2.0), (&DEER, 3.0), (&SAGE_HEN, 0.5)],
        Region::Columbia => vec![(&DEER, 2.5), (&RABBIT, 1.0), (&ELK, 0.5)],
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Grazing,
    /// head up, staring about; settles again if nothing more happens
    Alert,
    /// running, this way
    Fleeing(i32, i32),
    Down,
    /// out of the field, gone
    Gone,
}

#[derive(Debug)]
pub struct Animal {
    pub kind: &'static Kind,
    pub x: i32,
    pub y: i32,
    pub state: State,
    /// fear, 0 calm; alert at 4, off at 10
    pub alarm: f64,
    /// the herd it belongs to
    pub herd: usize,
}

pub struct Stalk {
    rng: Rng,
    pub ground: Vec<Ground>,
    pub hunter: (i32, i32),
    pub animals: Vec<Animal>,
    /// the way the wind blows, toward: one of eight directions
    pub wind: (i32, i32),
    pub tick: u32,
    /// rounds carried out, and fired
    pub rounds: u32,
    pub fired: u32,
    /// pounds the party can carry back
    pub carry: u32,
    /// ticks since the hunter last moved, for being seen
    pub still: u32,
    /// the latest thing that happened, for the line under the field
    pub message: String,
    pub over: bool,
}

pub const DIRS: [(i32, i32); 8] = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)];

impl Stalk {
    /// A day's stalking in this country and season. `wary` is how many days
    /// the party has already hunted this stretch: fewer animals each time.
    pub fn new(seed: u64, region: Region, day: u32, wary: u32, rounds: u32, carry: u32) -> Stalk {
        let mut rng = Rng::new(seed);
        let mut ground = vec![Ground::Grass; (W * H) as usize];
        let (brush, rock, tree) = match region {
            Region::Prairie => (0.10, 0.0, 0.03),
            Region::HighPlains => (0.06, 0.01, 0.0),
            Region::Mountains => (0.08, 0.06, 0.04),
            Region::SnakeDesert => (0.12, 0.05, 0.0),
            Region::BlueMountains => (0.06, 0.02, 0.12),
            Region::Columbia => (0.07, 0.02, 0.1),
        };
        // patches of cover, grown from seeds so they clump
        let cells = (W * H) as f64;
        for (what, share) in [(Ground::Brush, brush), (Ground::Rock, rock), (Ground::Tree, tree)] {
            let patches = (cells * share / 5.0).round() as usize;
            for _ in 0..patches {
                let (mut x, mut y) = (rng.range(0, W as i64 - 1) as i32, rng.range(0, H as i64 - 1) as i32);
                for _ in 0..5 {
                    ground[(y * W + x) as usize] = what;
                    let d = *rng.pick(&DIRS);
                    x = (x + d.0).clamp(0, W - 1);
                    y = (y + d.1).clamp(0, H - 1);
                }
            }
        }
        // a creek across some country
        if matches!(region, Region::Prairie | Region::Mountains | Region::Columbia | Region::BlueMountains) {
            let mut y = rng.range(3, H as i64 - 5) as i32;
            let fords = rng.range(0, 9) as i32;
            for x in 0..W {
                // shallow fords every so often, where a hunter can wade
                if (x + fords) % 10 >= 2 {
                    ground[(y * W + x) as usize] = Ground::Water;
                }
                if x % 6 == 0 {
                    y = (y + rng.range(-1, 1) as i32).clamp(2, H - 4);
                }
            }
        }
        let hunter = (W / 2, H - 1);
        ground[(hunter.1 * W + hunter.0) as usize] = Ground::Grass;
        let mut animals = Vec::new();
        let kinds = about(region, day);
        let groups = (rng.range(2, 4) as f64 * 0.75f64.powi(wary as i32)).round().max(1.0) as usize;
        for herd in 0..groups {
            let Some(kind) = rng.weighted(&kinds) else { break };
            let n = rng.range(kind.herd.0, kind.herd.1);
            let (cx, cy) = (rng.range(4, W as i64 - 5) as i32, rng.range(1, (H / 2) as i64) as i32);
            for _ in 0..n {
                let (x, y) = ((cx + rng.range(-3, 3) as i32).clamp(0, W - 1), (cy + rng.range(-2, 2) as i32).clamp(0, H - 1));
                if ground[(y * W + x) as usize] != Ground::Water {
                    animals.push(Animal { kind, x, y, state: State::Grazing, alarm: 0.0, herd });
                }
            }
        }
        let wind = *rng.pick(&DIRS);
        let mut seen: Vec<&str> = Vec::new();
        for a in &animals {
            if !seen.contains(&a.kind.plural) {
                seen.push(a.kind.plural);
            }
        }
        let message = match seen.len() {
            0 => "You slip away from the wagons with your rifle. Nothing stirs.".to_string(),
            1 => format!("You slip away from the wagons with your rifle. There are {} about.", seen[0]),
            n => format!("You slip away from the wagons with your rifle. There are {} and {} about.", seen[..n - 1].join(", "), seen[n - 1]),
        };
        Stalk { rng, ground, hunter, animals, wind, tick: 0, rounds, fired: 0, carry, still: 20, message, over: false }
    }

    pub fn at(&self, x: i32, y: i32) -> Ground {
        if (0..W).contains(&x) && (0..H).contains(&y) { self.ground[(y * W + x) as usize] } else { Ground::Grass }
    }

    fn animal_at(&self, x: i32, y: i32) -> bool {
        self.animals.iter().any(|a| a.x == x && a.y == y && !matches!(a.state, State::Gone))
    }

    /// The hunter creeps a cell; water and animals are in the way.
    pub fn step(&mut self, dx: i32, dy: i32) {
        if self.over {
            return;
        }
        let (x, y) = (self.hunter.0 + dx, self.hunter.1 + dy);
        if !(0..W).contains(&x) || !(0..H).contains(&y) || self.at(x, y) == Ground::Water || self.animal_at(x, y) {
            return;
        }
        self.hunter = (x, y);
        self.still = 0;
    }

    /// The live animal nearest the hunter, and how far.
    fn nearest(&self) -> Option<(usize, f64)> {
        self.animals
            .iter()
            .enumerate()
            .filter(|(_, a)| matches!(a.state, State::Grazing | State::Alert | State::Fleeing(..)))
            .map(|(i, a)| (i, dist((a.x, a.y), self.hunter)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// Fire at the nearest animal in range.
    pub fn fire(&mut self) {
        if self.over {
            return;
        }
        if self.rounds == self.fired {
            self.message = "You are out of powder and shot.".into();
            return;
        }
        let Some((i, d)) = self.nearest().filter(|&(_, d)| d <= 14.0) else {
            self.message = "Nothing in range. Get closer before you waste a round.".into();
            return;
        };
        self.fired += 1;
        let kind = self.animals[i].kind;
        let chance = kind.hit * (1.35 - d / 12.0).clamp(0.1, 1.0);
        if self.rng.chance(chance) {
            self.animals[i].state = State::Down;
            self.message = format!("A hit at {} paces: the {} is down.", (d * 5.0).round() as u32, kind.name);
        } else {
            self.message = format!("You fired at {} paces and missed the {}.", (d * 5.0).round() as u32, kind.name);
        }
        // the shot startles everything that heard it
        let hunter = self.hunter;
        for a in &mut self.animals {
            if matches!(a.state, State::Grazing | State::Alert) {
                let near = dist((a.x, a.y), hunter);
                a.alarm += a.kind.startle * (1.0 - near / 60.0).max(0.2);
            }
        }
        self.still = 0;
    }

    /// Head back to the wagon now.
    pub fn quit(&mut self) {
        self.over = true;
    }

    /// Pounds brought down, and what the party carries back.
    pub fn meat(&self) -> (u32, u32) {
        let down: u32 = self.animals.iter().filter(|a| a.state == State::Down).map(|a| a.kind.meat).sum();
        (down, down.min(self.carry))
    }

    /// "a buffalo and two antelope", or None.
    pub fn bag(&self) -> Option<String> {
        let mut kinds: Vec<(&Kind, u32)> = Vec::new();
        for a in self.animals.iter().filter(|a| a.state == State::Down) {
            match kinds.iter_mut().find(|(k, _)| std::ptr::eq(*k, a.kind)) {
                Some(e) => e.1 += 1,
                None => kinds.push((a.kind, 1)),
            }
        }
        if kinds.is_empty() {
            return None;
        }
        let words: Vec<String> = kinds
            .iter()
            .map(|(k, n)| match n {
                1 => format!("{} {}", if "aeiou".contains(k.name.chars().next().unwrap_or('x')) { "an" } else { "a" }, k.name),
                n => format!("{} {}", number(*n), k.plural),
            })
            .collect();
        Some(match words.len() {
            1 => words[0].clone(),
            n => format!("{} and {}", words[..n - 1].join(", "), words[n - 1]),
        })
    }

    /// The day moves on by a tick.
    pub fn tick(&mut self) {
        if self.over {
            return;
        }
        self.tick += 1;
        self.still = self.still.saturating_add(1);
        if self.tick >= DAY {
            self.over = true;
            self.message = "The sun is down. Time to find the wagons.".into();
            return;
        }
        if self.tick.is_multiple_of(120) {
            // the wind backs or veers a point
            let i = DIRS.iter().position(|&d| d == self.wind).unwrap_or(0);
            let turn = if self.rng.chance(0.5) { 1 } else { 7 };
            self.wind = DIRS[(i + turn) % 8];
            self.message = format!("The wind shifts; it blows {} now.", wind_word(self.wind));
        }
        let hunter = self.hunter;
        let hidden = self.at(hunter.0, hunter.1).cover();
        let moving = self.still < 3;
        let wind = self.wind;
        for i in 0..self.animals.len() {
            let a = &self.animals[i];
            let (ax, ay) = (a.x, a.y);
            let d = dist((ax, ay), hunter);
            match a.state {
                State::Down | State::Gone => continue,
                State::Fleeing(dx, dy) => {
                    if self.tick.is_multiple_of(a.kind.run) {
                        let (nx, ny) = (ax + dx, ay + dy);
                        let a = &mut self.animals[i];
                        if !(0..W).contains(&nx) || !(0..H).contains(&ny) {
                            a.state = State::Gone;
                        } else {
                            a.x = nx;
                            a.y = ny;
                        }
                    }
                    continue;
                }
                State::Grazing | State::Alert => {}
            }
            // scent: the wind carries it from the hunter to the animal
            let (vx, vy) = ((ax - hunter.0) as f64, (ay - hunter.1) as f64);
            let along = (vx * wind.0 as f64 + vy * wind.1 as f64) / ((wind.0 * wind.0 + wind.1 * wind.1) as f64).sqrt();
            let mut fear = 0.0;
            if d < 16.0 && along > d * 0.75 {
                fear += 1.5;
            }
            // sight: a hunter moving in the open, or standing close
            let sight = if hidden { 2.0 } else { a.kind.sight };
            if d <= sight && (moving || d <= sight / 2.0) {
                fear += if moving { 1.2 } else { 0.5 };
            }
            if d < 1.5 {
                fear += 5.0;
            }
            let a = &mut self.animals[i];
            a.alarm = if fear > 0.0 { a.alarm + fear } else { (a.alarm - 0.08).max(0.0) };
            if a.alarm >= 10.0 {
                // run from the hunter, the whole herd with it
                let herd = a.herd;
                let away = (sign(ax - hunter.0), if ay <= hunter.1 { -1 } else { 1 });
                let away = if away == (0, 0) { (0, -1) } else { away };
                for b in self.animals.iter_mut().filter(|b| b.herd == herd && matches!(b.state, State::Grazing | State::Alert)) {
                    b.state = State::Fleeing(if away.0 == 0 { sign(b.x - hunter.0) } else { away.0 }, away.1);
                }
                let k = self.animals[i].kind;
                self.message = format!("The {} {} and run.", k.plural, if fear >= 1.5 && d > sight { "catch your scent" } else { "see you" });
                continue;
            }
            a.state = if a.alarm >= 4.0 { State::Alert } else { State::Grazing };
            // grazing: an unhurried step now and then, away from the water
            if a.state == State::Grazing && self.rng.chance(0.06) {
                let d = *self.rng.pick(&DIRS);
                let (nx, ny) = (ax + d.0, ay + d.1);
                if (0..W).contains(&nx) && (0..H - 2).contains(&ny) && self.at(nx, ny) != Ground::Water && (nx, ny) != hunter && !self.animal_at(nx, ny) {
                    let a = &mut self.animals[i];
                    a.x = nx;
                    a.y = ny;
                }
            }
        }
    }

    /// How far through the day, 0 to 1.
    pub fn daylight(&self) -> f64 {
        self.tick as f64 / DAY as f64
    }
}

fn dist(a: (i32, i32), b: (i32, i32)) -> f64 {
    // cells are twice as tall as wide: a row counts as two columns
    let (dx, dy) = ((a.0 - b.0) as f64, (a.1 - b.1) as f64 * 2.0);
    (dx * dx + dy * dy).sqrt() / 2.0
}

fn sign(v: i32) -> i32 {
    v.signum()
}

fn number(n: u32) -> String {
    ["no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten"].get(n as usize).map_or(n.to_string(), |s| s.to_string())
}

/// "from the west", the way the wind blows toward given.
pub fn wind_word(toward: (i32, i32)) -> &'static str {
    match toward {
        (1, 0) => "from the west",
        (1, 1) => "from the northwest",
        (0, 1) => "from the north",
        (-1, 1) => "from the northeast",
        (-1, 0) => "from the east",
        (-1, -1) => "from the southeast",
        (0, -1) => "from the south",
        _ => "from the southwest",
    }
}

/// An arrow for the way the wind blows.
pub fn wind_arrow(toward: (i32, i32)) -> char {
    match toward {
        (1, 0) => '\u{2192}',
        (1, 1) => '\u{2198}',
        (0, 1) => '\u{2193}',
        (-1, 1) => '\u{2199}',
        (-1, 0) => '\u{2190}',
        (-1, -1) => '\u{2196}',
        (0, -1) => '\u{2191}',
        _ => '\u{2197}',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plains() -> Stalk {
        Stalk::new(3, Region::HighPlains, date::first_of(6) + 10, 0, 40, 200)
    }

    #[test]
    fn a_day_runs_out_and_nothing_leaves_the_field() {
        for seed in 0..50 {
            let mut s = Stalk::new(seed, Region::Mountains, date::first_of(7), 0, 20, 160);
            assert!(!s.animals.is_empty(), "seed {seed}");
            for t in 0..DAY {
                if t % 3 == 0 {
                    s.step(0, -1);
                }
                s.tick();
                assert!((0..W).contains(&s.hunter.0) && (0..H).contains(&s.hunter.1));
            }
            assert!(s.over);
        }
    }

    #[test]
    fn walking_straight_at_game_in_the_open_sends_it_off() {
        let mut s = plains();
        s.ground.iter_mut().for_each(|g| *g = Ground::Grass);
        let target = (s.animals[0].x, s.animals[0].y);
        for _ in 0..200 {
            let (dx, dy) = (sign(target.0 - s.hunter.0), sign(target.1 - s.hunter.1));
            s.step(dx, dy);
            s.tick();
        }
        assert!(s.animals.iter().any(|a| matches!(a.state, State::Fleeing(..) | State::Gone)));
    }

    #[test]
    fn a_close_shot_usually_brings_game_down_and_the_carry_limits_the_meat() {
        let mut down = 0;
        for seed in 0..100 {
            let mut s = Stalk::new(seed, Region::HighPlains, date::first_of(6) + 10, 0, 40, 80);
            let a = &s.animals[0];
            s.hunter = (a.x, (a.y + 1).min(H - 1));
            if s.hunter == (a.x, a.y) {
                continue;
            }
            s.fire();
            assert_eq!(s.fired, 1);
            let (shot, carried) = s.meat();
            assert!(carried <= 80 && carried <= shot);
            if shot > 0 {
                down += 1;
                assert!(s.bag().is_some());
            }
        }
        assert!(down > 40, "{down}");
        // no rounds, no shot
        let mut s = Stalk::new(1, Region::HighPlains, date::first_of(6), 0, 0, 80);
        s.fire();
        assert_eq!(s.fired, 0);
    }
}
