//! A day's hunting: what is about in this country and season, what it costs
//! in powder and shot, and what one person can carry back to the wagon.

use crate::date;
use crate::rng::Rng;
use crate::trail::Region;

/// The most meat a hunter can carry back to camp in a day.
pub const CARRY: u32 = 100;
/// Rounds a hunt needs at the least.
pub const SHOT_NEEDED: u32 = 10;

struct Game {
    name: &'static str,
    /// pounds of meat from one animal
    meat: u32,
    /// the chance of bringing one down
    hit: f64,
}

const BUFFALO: Game = Game { name: "a buffalo", meat: 450, hit: 0.55 };
const ANTELOPE: Game = Game { name: "an antelope", meat: 45, hit: 0.35 };
const DEER: Game = Game { name: "a deer", meat: 70, hit: 0.5 };
const ELK: Game = Game { name: "an elk", meat: 250, hit: 0.4 };
const SAGE_HENS: Game = Game { name: "a brace of sage hens", meat: 6, hit: 0.8 };
const RABBITS: Game = Game { name: "three rabbits", meat: 6, hit: 0.8 };
const TURKEY: Game = Game { name: "a wild turkey", meat: 12, hit: 0.6 };

fn about(region: Region, day: u32) -> Vec<(&'static Game, f64)> {
    let m = date::month(day);
    let herds = (5..=8).contains(&m);
    match region {
        Region::Prairie => vec![(&DEER, 2.0), (&TURKEY, 2.0), (&RABBITS, 2.0), (&BUFFALO, if herds { 0.5 } else { 0.0 })],
        Region::HighPlains => vec![(&BUFFALO, if herds { 4.0 } else { 1.0 }), (&ANTELOPE, 3.0), (&RABBITS, 1.0)],
        Region::Mountains => vec![(&ELK, 1.5), (&DEER, 2.0), (&ANTELOPE, 2.0), (&SAGE_HENS, 2.0)],
        Region::SnakeDesert => vec![(&RABBITS, 3.0), (&SAGE_HENS, 3.0), (&ANTELOPE, 0.5)],
        Region::BlueMountains => vec![(&ELK, 2.0), (&DEER, 3.0), (&SAGE_HENS, 0.5)],
        Region::Columbia => vec![(&DEER, 2.0), (&RABBITS, 1.0)],
    }
}

pub struct Hunt {
    pub text: String,
    /// pounds brought back
    pub meat: u32,
    pub shot: u32,
}

/// Go hunting. `hunted` is how many days the party has already hunted on this
/// stretch of road: the game learns to keep away.
pub fn hunt(rng: &mut Rng, region: Region, day: u32, hunted: u32, shot: u32) -> Hunt {
    let used = (rng.range(10, 30) as u32).min(shot);
    let wary = 0.8f64.powi(hunted as i32);
    let found = about(region, day);
    let weights: Vec<(usize, f64)> = found.iter().enumerate().map(|(i, g)| (i, g.1)).collect();
    if !rng.chance(0.85 * wary) {
        let text = *rng.pick(&[
            "You walked the creek bottoms all day and saw nothing worth a shot.",
            "Tracks everywhere, and not one animal. The game has moved off.",
            "A long day in the hills. Whatever lives here saw you first.",
        ]);
        return Hunt { text: text.into(), meat: 0, shot: used / 2 };
    }
    let g = found[rng.weighted(&weights).unwrap_or(0)].0;
    if !rng.chance(g.hit) {
        return Hunt { text: format!("You got close to {} and missed. It was gone before you could load again.", g.name), meat: 0, shot: used };
    }
    let meat = g.meat.min(CARRY);
    let text = if g.meat > CARRY {
        format!("You brought down {}: {} lb of meat, but you could carry only {CARRY} lb back to the wagon.", g.name, g.meat)
    } else {
        format!("You brought down {}: {meat} lb of meat.", g.name)
    };
    Hunt { text, meat, shot: used }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plains_feed_hunters_in_summer_and_the_desert_does_not() {
        let total = |region: Region| -> u32 { (0..500).map(|s| hunt(&mut Rng::new(s), region, date::first_of(6), 0, 100).meat).sum() };
        let plains = total(Region::HighPlains);
        let desert = total(Region::SnakeDesert);
        assert!(plains > desert * 4, "{plains} {desert}");
        let h = hunt(&mut Rng::new(1), Region::HighPlains, date::first_of(6), 0, 100);
        assert!(h.meat <= CARRY && h.shot <= 30);
    }
}
