//! What happens on the road: sickness, accidents, breakdowns, weather,
//! luck good and bad. One roll a travelling day; what can happen depends on
//! the country, the season and how the party is travelling.

use crate::date;
use crate::party::{Illness, Item, Person, Supplies};
use crate::rng::Rng;
use crate::trail::{Journey, Pace, Rations, Region};

/// The chance of something happening on any one day of travel.
const DAILY: f64 = 0.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    Cholera,
    MountainFever,
    Measles,
    Typhoid,
    Scurvy,
    BrokenBone,
    RunOver,
    Snakebite,
    Breakdown,
    OxStrays,
    OxDies,
    Hail,
    PoorGrass,
    BadWater,
    LostTrail,
    Abandoned,
    GoodCamp,
    Kindness,
    Fire,
    Theft,
    Mud,
}

/// What a day's event came to.
#[derive(Debug, Default)]
pub struct Happening {
    pub text: String,
    /// days the party loses to it, spent in camp
    pub days_lost: u32,
    /// worth stopping the week's travel for, so the player can respond
    pub stop: bool,
}

fn weights(j: &Journey, day: u32, supplies: &Supplies, started: u32) -> Vec<(Event, f64)> {
    use Event::*;
    use Region::*;
    let r = j.region;
    let m = date::month(day);
    let plains = matches!(r, Prairie | HighPlains);
    let summer = (5..=8).contains(&m);
    let hard = match j.pace {
        Pace::Steady => 1.0,
        Pace::Hard => 1.3,
        Pace::Grueling => 1.8,
    };
    let rough = if matches!(r, Mountains | BlueMountains) { 2.0 } else { 1.0 };
    // scurvy comes after two months or more without fresh food
    let long_out = (day.saturating_sub(started) as f64 / 60.0 - 0.75).max(0.0);
    vec![
        // 1848 came just before the great cholera years on the trail
        (Cholera, if plains && summer { 0.5 } else { 0.0 }),
        (MountainFever, if r == Mountains && summer { 2.0 } else { 0.0 }),
        (Measles, if plains { 0.8 } else { 0.2 }),
        (Typhoid, 0.8 * hard),
        (Scurvy, if j.rations != Rations::Filling { 0.8 * long_out } else { 0.15 * long_out }),
        (BrokenBone, 0.7 * hard * rough),
        (RunOver, 0.35 * hard),
        (Snakebite, if matches!(r, HighPlains | SnakeDesert) { 0.6 } else { 0.1 }),
        (Breakdown, 1.4 * hard * rough),
        (OxStrays, if supplies.get(Item::Oxen) > 0 { 0.9 } else { 0.0 }),
        (OxDies, if supplies.get(Item::Oxen) > 0 { (100.0 - j.oxen_health) / 25.0 } else { 0.0 }),
        (Hail, if plains && (4..=7).contains(&m) { 0.5 } else { 0.1 }),
        (PoorGrass, if r == SnakeDesert { 1.5 } else if m <= 4 || m >= 9 { 0.8 } else { 0.3 }),
        (BadWater, if matches!(r, HighPlains | SnakeDesert) { 1.0 } else { 0.3 }),
        (LostTrail, if plains { 0.15 } else { 0.6 }),
        (Abandoned, if plains { 0.5 } else { 1.2 }),
        (GoodCamp, 0.9),
        (Kindness, if r == Prairie { 0.2 } else { 0.7 }),
        (Fire, 0.3),
        (Theft, 0.4),
        (Mud, if r == Prairie && m <= 6 { 1.0 } else { 0.2 }),
    ]
}

/// The chance of snow closing the road on a day, for a party still in the
/// high country late in the year: the Rockies from mid-September, the Blue
/// Mountains from October, the Cascades on the Barlow Road from late October.
fn snow_chance(r: Region, day: u32) -> f64 {
    // half-months since the first of September
    let half = day.saturating_sub(date::first_of(9)) / 15;
    let by_half: [f64; 8] = match r {
        Region::Mountains => [0.0, 0.03, 0.07, 0.12, 0.18, 0.25, 0.3, 0.35],
        Region::BlueMountains => [0.0, 0.0, 0.04, 0.09, 0.15, 0.22, 0.3, 0.35],
        Region::Columbia => [0.0, 0.0, 0.0, 0.04, 0.1, 0.18, 0.25, 0.3],
        _ => return 0.0,
    };
    by_half[(half as usize).min(7)]
}

/// Snow on the heights, if it comes today: days lost breaking a road, the
/// oxen and the people worn by it.
pub fn snow(rng: &mut Rng, j: &mut Journey, day: u32, party: &mut [Person], supplies: &mut Supplies) -> Option<Happening> {
    if !rng.chance(snow_chance(j.region, day)) {
        return None;
    }
    let days = rng.range(2, 5) as u32;
    j.oxen_health = (j.oxen_health - 15.0).max(0.0);
    for p in party.iter_mut().filter(|p| p.alive()) {
        p.health = (p.health - 10).max(1);
    }
    let mut text = format!("Snow on the heights, a foot of it by morning. The oxen paw for grass and you lose {days} days breaking a road through the drifts.");
    if supplies.get(Item::Oxen) > 0 && rng.chance(0.3) {
        supplies.take(Item::Oxen, 1);
        text.push_str(" One ox does not live through the cold.");
    }
    Some(Happening { text, days_lost: days, stop: true })
}

/// The people still alive, by index.
fn living(party: &[Person]) -> Vec<usize> {
    (0..party.len()).filter(|&i| party[i].alive()).collect()
}

/// Fall ill or hurt: an initial blow, then a daily drain whose strength
/// varies from case to case, mild to grave.
fn fall_ill(rng: &mut Rng, p: &mut Person, name: &'static str, hit: i32, drain: i32, days: u32) {
    let severity = 0.5 + rng.unit() * rng.unit() * 2.0;
    p.health = (p.health - hit).max(1);
    p.illness = Some(Illness { name, drain: ((drain as f64 * severity).round() as i32).max(1), days });
}

/// Roll for the day. None when the day passes quietly.
pub fn roll(rng: &mut Rng, j: &mut Journey, day: u32, party: &mut [Person], supplies: &mut Supplies, started: u32) -> Option<Happening> {
    if !rng.chance(DAILY) {
        return None;
    }
    let event = rng.weighted(&weights(j, day, supplies, started))?;
    let who = living(party);
    let mut h = Happening::default();
    // someone not already sick, for a sickness or an accident
    let well: Vec<usize> = who.iter().copied().filter(|&i| party[i].illness.is_none()).collect();
    let pick_well = |rng: &mut Rng| -> Option<usize> { if well.is_empty() { None } else { Some(*rng.pick(&well)) } };
    use Event::*;
    match event {
        Cholera | MountainFever | Measles | Typhoid | Scurvy | BrokenBone | RunOver | Snakebite => {
            let i = pick_well(rng)?;
            let p = &mut party[i];
            let n = p.name.clone();
            h.stop = true;
            h.text = match event {
                Cholera => {
                    fall_ill(rng, p, "cholera", 25, 14, 4);
                    format!("{n} is doubled over with cramps and cannot keep water down. Cholera.")
                }
                MountainFever => {
                    fall_ill(rng, p, "mountain fever", 10, 5, 8);
                    format!("{n} has a fever, aching joints and a headache that will not lift. Mountain fever, the old hands call it.")
                }
                Measles => {
                    fall_ill(rng, p, "measles", 10, 4, 7);
                    format!("{n} is covered in a red rash and burning up. Measles.")
                }
                Typhoid => {
                    fall_ill(rng, p, "typhoid fever", 10, 9, 10);
                    format!("{n} has a fever that climbs a little higher each day. Typhoid.")
                }
                Scurvy => {
                    fall_ill(rng, p, "scurvy", 5, 3, 15);
                    format!("{n}'s gums are swollen and bleeding, and old cuts have opened again. Scurvy, from too long without fresh food.")
                }
                BrokenBone => {
                    let what = *rng.pick(&["an arm", "a leg", "a collarbone"]);
                    fall_ill(rng, p, "a broken bone", 20, 2, 12);
                    format!("{n} fell under the wagon and broke {what}. You splint it as best you can.")
                }
                RunOver => {
                    let hit = rng.range(30, 75) as i32;
                    fall_ill(rng, p, "injuries", hit, 4, 6);
                    format!("{n} slipped from the wagon tongue and a wheel went over them. They are badly hurt.")
                }
                _ => {
                    fall_ill(rng, p, "a snakebite", 30, 3, 4);
                    format!("A rattlesnake bit {n} on the ankle. The leg swells black and blue.")
                }
            };
        }
        Breakdown => {
            let (part, item) = *rng.pick(&[("A wheel", Item::Wheel), ("An axle", Item::Axle), ("The tongue", Item::Tongue)]);
            if supplies.take(item, 1) == 1 {
                h.text = format!("{part} broke. You fit the spare and lose half a day.");
            } else {
                h.days_lost = rng.range(2, 4) as u32;
                h.stop = true;
                h.text = format!("{part} broke and you have no spare. It takes {} days to cut timber and make one that will hold.", h.days_lost);
            }
        }
        OxStrays => {
            if rng.chance(0.5) {
                h.days_lost = 1;
                h.text = "An ox strayed in the night. You spend a day finding it.".into();
            } else {
                supplies.take(Item::Oxen, 1);
                h.stop = true;
                h.text = "An ox strayed in the night and could not be found.".into();
            }
        }
        OxDies => {
            supplies.take(Item::Oxen, 1);
            h.stop = true;
            h.text = "One of the oxen lay down in the traces and did not get up.".into();
        }
        Hail => {
            let lost = supplies.take(Item::Food, rng.range(10, 60) as u32);
            h.text = format!("A hailstorm comes up out of nowhere. Stones the size of eggs tear the wagon cover and spoil {lost} lb of provisions.");
            if let Some(i) = pick_well(rng)
                && rng.chance(0.3)
            {
                party[i].health = (party[i].health - 10).max(1);
                h.text.push_str(&format!(" {} is badly bruised.", party[i].name));
            }
        }
        PoorGrass => {
            j.oxen_health = (j.oxen_health - 12.0).max(0.0);
            h.text = "The grass is grazed bare for miles. The oxen go hungry.".into();
        }
        BadWater => {
            if let Some(i) = pick_well(rng)
                && rng.chance(0.5)
            {
                fall_ill(rng, &mut party[i], "bad water", 8, 4, 4);
                h.text = format!("The only water is brackish and foul. {} drank it and is sick.", party[i].name);
                h.stop = true;
            } else {
                j.oxen_health = (j.oxen_health - 10.0).max(0.0);
                h.text = "The water is alkaline. The oxen drink it anyway and sicken.".into();
            }
        }
        LostTrail => {
            h.days_lost = rng.range(1, 2) as u32;
            let plains = matches!(j.region, Region::Prairie | Region::HighPlains);
            let how = if plains { "You followed a stock trail off the road" } else { "You lost the trail in broken country" };
            h.text = format!("{how} and spent {} finding it again.", if h.days_lost == 1 { "a day" } else { "two days" });
        }
        Abandoned => {
            let found = *rng.pick(&[Item::Food, Item::Food, Item::Clothing, Item::Shot, Item::Wheel, Item::Axle, Item::Tongue]);
            let n = match found {
                Item::Food => rng.range(2, 8) as u32 * 10,
                Item::Clothing => rng.range(1, 3) as u32,
                Item::Shot => rng.range(1, 3) as u32 * 20,
                _ => 1,
            };
            supplies.add(found, n);
            h.text = format!("By the road, a wagon someone gave up on. You take {}.", crate::game::amount_of((found, n)));
        }
        GoodCamp => {
            for &i in &who {
                party[i].health = (party[i].health + 6).min(100);
            }
            h.text = (*rng.pick(&[
                "A camp with sweet water, firewood and wild currants. Everyone eats well and sleeps well.",
                "A cold clear spring and shade. The whole party feels better for it.",
                "Another train shares fresh milk and news from home. Spirits lift.",
            ]))
            .into();
        }
        Kindness => {
            // the peoples whose country the road crossed in 1848
            let people = match j.region {
                Region::Prairie => "A Kaw family",
                Region::HighPlains => *rng.pick(&["A Lakota family", "A Cheyenne family"]),
                Region::Mountains | Region::SnakeDesert => "A Shoshone family",
                Region::BlueMountains => "A Nez Perce family",
                Region::Columbia => "A Wasco family",
            };
            if j.region == Region::SnakeDesert || j.region == Region::Columbia {
                supplies.add(Item::Food, 30);
                h.text = format!("{people} trades you dried salmon for a few needles and some thread. 30 lb of provisions.");
            } else {
                j.oxen_health = (j.oxen_health + 15.0).min(100.0);
                h.text = format!("{people} shows you a valley off the road with good grass and water. The oxen feed well.");
            }
        }
        Fire => {
            let food = supplies.get(Item::Food) / 10;
            supplies.take(Item::Food, food);
            let clothes = supplies.take(Item::Clothing, 1);
            h.text = format!("A spark from the cook fire caught the wagon cover. You lost {food} lb of provisions{}.", if clothes > 0 { " and a set of clothing" } else { "" });
        }
        Theft => {
            let food = supplies.take(Item::Food, rng.range(10, 40) as u32);
            h.text = format!("In the morning, {food} lb of provisions are gone. Nobody saw who took them.");
        }
        Mud => {
            h.days_lost = 1;
            h.text = "Rain has turned the road to deep mud. The wagon sinks to its hubs and you lose a day digging it out.".into();
        }
    }
    Some(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_event_can_happen_and_leaves_the_party_sound() {
        let mut seen = std::collections::HashSet::new();
        for region in [Region::Prairie, Region::HighPlains, Region::Mountains, Region::SnakeDesert, Region::BlueMountains, Region::Columbia] {
            for seed in 0..400 {
                let mut rng = Rng::new(seed);
                let mut j = Journey::new();
                j.region = region;
                j.oxen_health = 40.0;
                j.rations = Rations::Meager;
                let mut party: Vec<Person> = ["A", "B", "C"].iter().map(|n| Person::new(n)).collect();
                let mut s = Supplies::default();
                s.set(Item::Oxen, 6);
                s.set(Item::Food, 500);
                let day = date::first_of(6) + 10;
                for _ in 0..20 {
                    if let Some(h) = roll(&mut rng, &mut j, day, &mut party, &mut s, date::first_of(4)) {
                        assert!(!h.text.is_empty());
                        assert!(!h.text.contains('\u{2014}') && !h.text.contains('\u{2013}'));
                        seen.insert(h.text.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
                    }
                }
                assert!(party.iter().all(|p| p.health >= 0 && p.health <= 100));
            }
        }
        assert!(seen.len() > 20, "{}", seen.len());
    }
}
#[cfg(test)]
mod count {
    use super::*;

    #[test]
    fn something_happens_about_one_day_in_ten() {
        let mut rng = Rng::new(11);
        let mut j = Journey::new();
        let mut party: Vec<Person> = ["A", "B", "C"].iter().map(|n| Person::new(n)).collect();
        let mut s = Supplies::default();
        s.set(Item::Oxen, 6);
        s.set(Item::Food, 500);
        let mut n = 0;
        for d in 0..1000 {
            if roll(&mut rng, &mut j, date::first_of(5) + d % 30, &mut party, &mut s, date::first_of(4)).is_some() {
                n += 1;
            }
        }
        assert!((50..130).contains(&n), "{n} in 1000 days");
    }
}
