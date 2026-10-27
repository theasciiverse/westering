//! One day on the trail at a time: the miles, the food, the weather, and
//! what they do to the people and the oxen.

use crate::date;
use crate::party::{Item, Person, Supplies};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    Steady,
    Hard,
    Grueling,
}

impl Pace {
    pub fn name(self) -> &'static str {
        match self {
            Pace::Steady => "steady",
            Pace::Hard => "hard",
            Pace::Grueling => "grueling",
        }
    }

    pub fn next(self) -> Pace {
        match self {
            Pace::Steady => Pace::Hard,
            Pace::Hard => Pace::Grueling,
            Pace::Grueling => Pace::Steady,
        }
    }

    /// Miles a day with a strong team on good ground.
    fn miles(self) -> f64 {
        match self {
            Pace::Steady => 13.0,
            Pace::Hard => 17.0,
            Pace::Grueling => 21.0,
        }
    }

    /// What the pace costs a walker's health each day.
    fn toll(self) -> i32 {
        match self {
            Pace::Steady => 0,
            Pace::Hard => 1,
            Pace::Grueling => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rations {
    Filling,
    Meager,
    Bare,
}

impl Rations {
    pub fn name(self) -> &'static str {
        match self {
            Rations::Filling => "filling",
            Rations::Meager => "meager",
            Rations::Bare => "bare bones",
        }
    }

    pub fn next(self) -> Rations {
        match self {
            Rations::Filling => Rations::Meager,
            Rations::Meager => Rations::Bare,
            Rations::Bare => Rations::Filling,
        }
    }

    /// Pounds a person eats in a day.
    pub fn pounds(self) -> u32 {
        match self {
            Rations::Filling => 3,
            Rations::Meager => 2,
            Rations::Bare => 1,
        }
    }

    fn effect(self) -> i32 {
        match self {
            Rations::Filling => 2,
            Rations::Meager => 0,
            Rations::Bare => -2,
        }
    }
}

/// The country the trail runs through, which sets the weather and the grass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Prairie,
    HighPlains,
    Mountains,
    SnakeDesert,
    BlueMountains,
    Columbia,
}

impl Region {
    /// Degrees colder than the prairie, for the height of the country.
    fn chill(self) -> f64 {
        match self {
            Region::Prairie => 0.0,
            Region::HighPlains => 5.0,
            Region::Mountains => 16.0,
            Region::SnakeDesert => 2.0,
            Region::BlueMountains => 14.0,
            Region::Columbia => 4.0,
        }
    }

    fn wet(self) -> f64 {
        match self {
            Region::Prairie => 1.0,
            Region::HighPlains => 0.6,
            Region::Mountains => 0.7,
            Region::SnakeDesert => 0.3,
            Region::BlueMountains => 0.9,
            Region::Columbia => 1.1,
        }
    }

    fn grass(self) -> f64 {
        match self {
            Region::Prairie => 1.0,
            Region::HighPlains => 0.8,
            Region::Mountains => 0.7,
            Region::SnakeDesert => 0.35,
            Region::BlueMountains => 0.8,
            Region::Columbia => 0.6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weather {
    /// degrees Fahrenheit, the midday temperature
    pub temp: f64,
    pub sky: Sky,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sky {
    Fair,
    Rain,
    Downpour,
    Snow,
    Blizzard,
}

impl Weather {
    pub fn today(rng: &mut Rng, day: u32, region: Region) -> Weather {
        // midday at Independence, by month, January first
        const NORMAL: [f64; 12] = [36.0, 42.0, 52.0, 64.0, 73.0, 82.0, 88.0, 86.0, 78.0, 66.0, 52.0, 40.0];
        // chance of rain or snow on a day, by month
        const WET: [f64; 12] = [0.15, 0.18, 0.25, 0.3, 0.33, 0.3, 0.22, 0.2, 0.2, 0.2, 0.18, 0.15];
        let m = (date::month(day) - 1) as usize;
        let temp = NORMAL[m] - region.chill() + rng.range(-12, 12) as f64;
        let wet = rng.chance(WET[m] * region.wet());
        let sky = match (wet, temp < 34.0) {
            (false, _) => Sky::Fair,
            (true, true) if rng.chance(0.25) => Sky::Blizzard,
            (true, true) => Sky::Snow,
            (true, false) if rng.chance(0.25) => Sky::Downpour,
            (true, false) => Sky::Rain,
        };
        Weather { temp, sky }
    }

    pub fn describe(self) -> String {
        let t = match self.temp {
            t if t >= 90.0 => "very hot",
            t if t >= 78.0 => "hot",
            t if t >= 60.0 => "warm",
            t if t >= 45.0 => "cool",
            t if t >= 30.0 => "cold",
            _ => "bitter cold",
        };
        match self.sky {
            Sky::Fair => t.to_string(),
            Sky::Rain => format!("{t}, rain"),
            Sky::Downpour => format!("{t}, a downpour"),
            Sky::Snow => format!("{t}, snow"),
            Sky::Blizzard => format!("{t}, a blizzard"),
        }
    }

    /// What the weather leaves of a day's miles.
    fn going(self) -> f64 {
        match self.sky {
            Sky::Fair => 1.0,
            Sky::Rain => 0.85,
            Sky::Downpour => 0.55,
            Sky::Snow => 0.6,
            Sky::Blizzard => 0.15,
        }
    }
}

/// The state of a journey under way.
#[derive(Clone, Debug)]
pub struct Journey {
    pub miles: f64,
    pub pace: Pace,
    pub rations: Rations,
    /// the team's strength, 0 to 100
    pub oxen_health: f64,
    pub weather: Weather,
    /// the country the wagon is in now
    pub region: Region,
}

impl Journey {
    pub fn new() -> Journey {
        Journey { miles: 0.0, pace: Pace::Steady, rations: Rations::Filling, oxen_health: 90.0, weather: Weather { temp: 60.0, sky: Sky::Fair }, region: Region::Prairie }
    }

    pub fn region(&self) -> Region {
        self.region
    }
}

/// What a day came to, for the diary.
#[derive(Debug, Default)]
pub struct Day {
    pub miles: f64,
    pub hungry: bool,
    pub cold: bool,
    /// names of those who died today
    pub died: Vec<String>,
}

/// How much grass the oxen find, by month (January first).
fn grass(day: u32, region: Region) -> f64 {
    const BY_MONTH: [f64; 12] = [0.1, 0.1, 0.3, 0.7, 1.0, 1.0, 0.95, 0.8, 0.65, 0.45, 0.25, 0.1];
    BY_MONTH[(date::month(day) - 1) as usize] * region.grass()
}

/// Miles the team pulls in a day at a pace, before the weather.
pub fn team_miles(j: &Journey, oxen: u32) -> f64 {
    if oxen < 2 {
        return 0.0;
    }
    let team = (oxen as f64 / 6.0).min(1.15).sqrt();
    j.pace.miles() * team * (0.45 + 0.55 * j.oxen_health / 100.0)
}

/// Live one day: travel (unless resting), eat, and take the weather.
pub fn day(rng: &mut Rng, j: &mut Journey, day: u32, party: &mut [Person], supplies: &mut Supplies, resting: bool) -> Day {
    let mut out = Day::default();
    j.weather = Weather::today(rng, day, j.region());
    let alive = party.iter().filter(|p| p.alive()).count() as u32;

    // eating
    let want = alive * j.rations.pounds();
    let ate = supplies.take(Item::Food, want);
    out.hungry = ate < want;
    let fed = if want == 0 { 1.0 } else { ate as f64 / want as f64 };

    // the cold, against the clothing
    let clothed = supplies.get(Item::Clothing) as f64 / (alive.max(1) as f64 * 2.0);
    let cold = match j.weather.temp {
        t if t < 20.0 => 6.0,
        t if t < 34.0 => 3.0,
        t if t < 45.0 => 1.0,
        _ => 0.0,
    } * (1.0 - clothed.min(1.0) * 0.8);
    out.cold = cold >= 2.0;
    let heat = if j.weather.temp >= 92.0 { 1 } else { 0 };

    for p in party.iter_mut().filter(|p| p.alive()) {
        let mut change = if fed >= 1.0 { j.rations.effect() } else { -2 - (8.0 * (1.0 - fed)) as i32 };
        if resting {
            change += 4;
        } else {
            change -= j.pace.toll() + heat;
        }
        change -= cold.round() as i32;
        if matches!(j.weather.sky, Sky::Downpour | Sky::Blizzard) && !resting {
            change -= 1;
        }
        // the hale mend a little on their own; the frail slip
        if p.illness.is_none() && p.health > 70 {
            change += 1;
        }
        p.health = (p.health + change).clamp(0, 100);
        if !p.alive() {
            out.died.push(p.name.clone());
        }
    }

    // the oxen
    let g = grass(day, j.region());
    let strain = if resting { -6.0 } else { [1.0, 2.5, 4.5][j.pace as usize] };
    j.oxen_health = (j.oxen_health + g * 4.0 - strain - if g < 0.3 { 1.5 } else { 0.0 }).clamp(0.0, 100.0);

    if !resting {
        out.miles = team_miles(j, supplies.get(Item::Oxen)) * j.weather.going() * (0.9 + rng.unit() * 0.2);
        j.miles += out.miles;
    }
    out
}

/// The party's health in a word, from the average of the living.
pub fn health_word(party: &[Person]) -> &'static str {
    let alive: Vec<i32> = party.iter().filter(|p| p.alive()).map(|p| p.health).collect();
    if alive.is_empty() {
        return "gone";
    }
    match alive.iter().sum::<i32>() / alive.len() as i32 {
        h if h >= 80 => "good",
        h if h >= 60 => "fair",
        h if h >= 35 => "poor",
        _ => "very poor",
    }
}

pub fn oxen_word(health: f64) -> &'static str {
    match health {
        h if h >= 75.0 => "strong",
        h if h >= 50.0 => "tiring",
        h if h >= 25.0 => "worn",
        _ => "failing",
    }
}

pub fn health_of(p: &Person) -> &'static str {
    match p.health {
        0 => "dead",
        h if h >= 80 => "good",
        h if h >= 60 => "fair",
        h if h >= 35 => "poor",
        _ => "very poor",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outfit() -> (Vec<Person>, Supplies) {
        let party: Vec<Person> = ["A", "B", "C", "D", "E"].iter().map(|n| Person::new(n)).collect();
        let mut s = Supplies::default();
        s.set(Item::Oxen, 6);
        s.set(Item::Food, 1000);
        s.set(Item::Clothing, 10);
        (party, s)
    }

    #[test]
    fn a_steady_spring_month_covers_ground_and_keeps_folk_well() {
        let (mut party, mut s) = outfit();
        let mut rng = Rng::new(5);
        let mut j = Journey::new();
        let start = date::first_of(4) + 20;
        for d in 0..30 {
            day(&mut rng, &mut j, start + d, &mut party, &mut s, false);
        }
        assert!((280.0..420.0).contains(&j.miles), "{} miles", j.miles);
        assert_eq!(s.get(Item::Food), 1000 - 30 * 5 * 3);
        assert!(party.iter().all(|p| p.health > 80));
    }

    #[test]
    fn starving_kills() {
        let (mut party, mut s) = outfit();
        s.set(Item::Food, 0);
        let mut rng = Rng::new(9);
        let mut j = Journey::new();
        let mut dead = 0;
        for d in 0..60 {
            dead += day(&mut rng, &mut j, date::first_of(5) + d, &mut party, &mut s, false).died.len();
        }
        assert_eq!(dead, 5);
        assert_eq!(health_word(&party), "gone");
    }
}
