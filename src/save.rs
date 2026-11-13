//! The journey kept between sessions: written to the state folder
//! (`$XDG_STATE_HOME/westering/journey.txt`) whenever the wagon is on the
//! trail or at a landmark, taken up again from the title page, and removed
//! when the journey ends. A plain text file, one field to a line.

use crate::game::{Game, Screen};
use crate::party::{ITEMS, Illness, Person};
use crate::rng::Rng;
use crate::trail::{Pace, Rations, Region, Sky, Weather};
use log::{info, warn};
use std::path::PathBuf;

const MAGIC: &str = "westering journey 1";

/// The folder Westering keeps its state in; none under test, so tests never
/// touch a player's journey.
pub fn dir() -> Option<PathBuf> {
    if cfg!(test) {
        return None;
    }
    let var = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = var("XDG_STATE_HOME").or_else(|| if cfg!(windows) { var("LOCALAPPDATA") } else { None }).or_else(|| var("HOME").map(|h| h.join(".local").join("state")))?;
    Some(base.join("westering"))
}

fn journey_file() -> Option<PathBuf> {
    dir().map(|d| d.join("journey.txt"))
}

/// Keep the journey, if it is at a page it can be taken up from.
pub fn store(game: &Game) {
    let Some(path) = journey_file() else { return };
    if !matches!(game.screen, Screen::Trail | Screen::Landmark) {
        return;
    }
    let result = std::fs::create_dir_all(path.parent().unwrap_or(&path)).and_then(|_| std::fs::write(&path, to_text(game)));
    if let Err(e) = result {
        warn!("could not keep the journey in {}: {e}", path.display());
    }
}

/// Forget the kept journey: it is over.
pub fn forget() {
    if let Some(path) = journey_file()
        && path.exists()
    {
        info!("journey over, removing {}", path.display());
        let _ = std::fs::remove_file(path);
    }
}

/// Is there a journey to take up?
pub fn exists() -> bool {
    journey_file().is_some_and(|p| p.exists())
}

/// The kept journey, if there is one and it reads.
pub fn load() -> Option<Game> {
    let path = journey_file()?;
    let text = std::fs::read_to_string(&path).ok()?;
    let game = from_text(&text);
    match &game {
        Some(_) => info!("took up the journey from {}", path.display()),
        None => warn!("could not read the journey in {}", path.display()),
    }
    game
}

fn opt(s: &Option<String>) -> &str {
    s.as_deref().unwrap_or("-")
}

pub fn to_text(g: &Game) -> String {
    let j = &g.journey;
    let mut out = vec![
        MAGIC.to_string(),
        format!("rng {}", g.rng.state()),
        format!("screen {}", if g.screen == Screen::Landmark { "landmark" } else { "trail" }),
        format!("money {}", g.money),
        format!("day {}", g.day),
        format!("started {}", g.started),
        format!("at {}", g.at),
        format!("to {}", g.to),
        format!("left {}", g.left),
        format!("prices {}", g.prices),
        format!("depth {}", g.depth),
        format!("hunted {}", g.hunted),
        format!("on_foot {}", g.on_foot as u8),
        format!("stretch {}", g.stretch),
        format!("supplies {}", ITEMS.iter().map(|&i| g.supplies.get(i).to_string()).collect::<Vec<_>>().join(" ")),
        format!("journey {} {} {} {} {} {} {}", j.miles, j.pace as u8, j.rations as u8, j.oxen_health, j.weather.temp, j.weather.sky as u8, j.region as u8),
        format!("passed {}", g.passed.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(" ")),
    ];
    for p in &g.party {
        let (ill, drain, days) = p.illness.as_ref().map_or(("-", 0, 0), |i| (i.name, i.drain, i.days));
        out.push(format!("person {}\t{ill}\t{drain}\t{days}\t{}\t{}\t{}\t{}", p.health, opt(&p.cause), opt(&p.grave), opt(&p.epitaph), p.name));
    }
    for d in &g.diary {
        out.push(format!("diary {d}"));
    }
    out.join("\n") + "\n"
}

/// The illnesses by name, for reading them back.
const ILLNESSES: [&str; 10] = ["cholera", "mountain fever", "measles", "typhoid fever", "scurvy", "a broken bone", "injuries", "a snakebite", "bad water", "a wound"];

pub fn from_text(text: &str) -> Option<Game> {
    let mut lines = text.lines();
    if lines.next()? != MAGIC {
        return None;
    }
    let mut g = Game::new(0);
    g.party.clear();
    for line in lines {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        let num = |s: &str| s.trim().parse::<f64>().ok();
        match key {
            "rng" => g.rng = Rng::new(rest.parse().ok()?),
            "screen" => g.screen = if rest == "landmark" { Screen::Landmark } else { Screen::Trail },
            "money" => g.money = rest.parse().ok()?,
            "day" => g.day = rest.parse().ok()?,
            "started" => g.started = rest.parse().ok()?,
            "at" => g.at = rest.parse().ok()?,
            "to" => g.to = rest.parse().ok()?,
            "left" => g.left = num(rest)?,
            "prices" => g.prices = num(rest)?,
            "depth" => g.depth = num(rest)?,
            "hunted" => g.hunted = rest.parse().ok()?,
            "on_foot" => g.on_foot = rest == "1",
            "stretch" => g.stretch = rest.parse().ok()?,
            "supplies" => {
                let v: Vec<u32> = rest.split_whitespace().filter_map(|n| n.parse().ok()).collect();
                if v.len() != ITEMS.len() {
                    return None;
                }
                for (&item, &n) in ITEMS.iter().zip(&v) {
                    g.supplies.set(item, n);
                }
            }
            "journey" => {
                let v: Vec<f64> = rest.split_whitespace().filter_map(|n| n.parse().ok()).collect();
                if v.len() != 7 {
                    return None;
                }
                let j = &mut g.journey;
                j.miles = v[0];
                j.pace = [Pace::Steady, Pace::Hard, Pace::Grueling][(v[1] as usize).min(2)];
                j.rations = [Rations::Filling, Rations::Meager, Rations::Bare][(v[2] as usize).min(2)];
                j.oxen_health = v[3];
                let sky = [Sky::Fair, Sky::Rain, Sky::Downpour, Sky::Snow, Sky::Blizzard][(v[5] as usize).min(4)];
                j.weather = Weather { temp: v[4], sky };
                j.region = [Region::Prairie, Region::HighPlains, Region::Mountains, Region::SnakeDesert, Region::BlueMountains, Region::Columbia][(v[6] as usize).min(5)];
            }
            "passed" => g.passed = rest.split_whitespace().filter_map(|n| n.parse().ok()).collect(),
            "person" => {
                let f: Vec<&str> = rest.split('\t').collect();
                if f.len() != 8 {
                    return None;
                }
                let some = |s: &str| if s == "-" { None } else { Some(s.to_string()) };
                let mut p = Person::new(f[7]);
                p.health = f[0].parse().ok()?;
                if let Some(&name) = ILLNESSES.iter().find(|&&n| n == f[1]) {
                    p.illness = Some(Illness { name, drain: f[2].parse().ok()?, days: f[3].parse().ok()? });
                }
                p.cause = some(f[4]);
                p.grave = some(f[5]);
                p.epitaph = some(f[6]);
                g.party.push(p);
            }
            "diary" => g.diary.push(rest.to_string()),
            _ => {}
        }
    }
    let route = crate::route::ROUTE.len();
    if g.party.len() != crate::game::PARTY || g.at >= route || g.to >= route {
        return None;
    }
    g.cursor = 0;
    g.note.clear();
    Some(g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bot::Strategy;

    #[test]
    fn a_journey_reads_back_as_it_was_and_plays_on_the_same() {
        // a journey stopped partway, kept, read back, and played on
        let mut g = crate::bot::play_until(31, Strategy::Sensible, 40);
        g.screen = Screen::Trail;
        let text = to_text(&g);
        let mut back = from_text(&text).expect("reads back");
        assert_eq!(to_text(&back), text);
        for _ in 0..3 {
            g.trail_choice(0);
            back.trail_choice(0);
        }
        assert_eq!(to_text(&back), to_text(&g));
        assert!(from_text("something else").is_none());
        assert!(dir().is_none(), "tests keep nothing on disk");
    }
}
