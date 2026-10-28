//! Getting the wagon over a river: how high the water is today, and what
//! each way across risks.

use crate::date;
use crate::party::{Item, Person, Supplies};
use crate::rng::Rng;

/// How a party crosses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    Drive,
    Float,
    Ferry,
    Wait,
}

/// Snowmelt raises the rivers into June and lets them fall after (January first).
const SEASON: [f64; 12] = [0.8, 0.8, 1.0, 1.3, 1.5, 1.6, 1.15, 0.85, 0.7, 0.7, 0.75, 0.8];

/// The depth in feet on the day a party arrives.
pub fn depth_on(rng: &mut Rng, base: f64, day: u32) -> f64 {
    base * SEASON[(date::month(day) - 1) as usize] * (0.85 + rng.unit() * 0.3)
}

/// A day later: the water drifts, more often down than up.
pub fn wait(rng: &mut Rng, depth: f64) -> f64 {
    (depth * (0.88 + rng.unit() * 0.17)).max(0.5)
}

/// The river's own name, from its landmark's: "South Platte crossing" is the South Platte.
pub fn river_name(landmark: &str) -> String {
    match landmark {
        "Three Island Crossing" => "Snake".into(),
        l => l.trim_end_matches(" crossing").into(),
    }
}

pub fn describe(depth: f64, width: u32) -> String {
    let how = match depth {
        d if d < 2.0 => "low and lazy",
        d if d < 3.0 => "up to the wheel hubs",
        d if d < 4.5 => "deep enough to reach the wagon bed",
        d if d < 6.0 => "high and fast",
        _ => "in flood, brown and roaring",
    };
    format!("The river is {width} yards across and {depth:.1} feet deep: {how}.")
}

/// The chance something goes wrong, and the chance that if it does, it is the worst.
fn risk(way: Way, depth: f64, width: u32, quicksand: bool) -> (f64, f64) {
    match way {
        Way::Drive => {
            let mut p = ((depth - 2.0) * 0.2).clamp(0.03, 0.9);
            if quicksand {
                p += 0.1;
            }
            (p, if depth > 4.0 { 0.35 } else { 0.1 })
        }
        Way::Float => ((0.07 + width as f64 / 4000.0 + (depth - 6.0).max(0.0) * 0.04).min(0.6), 0.15),
        Way::Ferry => (0.02, 0.0),
        Way::Wait => (0.0, 0.0),
    }
}

/// What a crossing came to: the diary lines, and anyone drowned.
#[derive(Debug, Default)]
pub struct Crossing {
    pub lines: Vec<String>,
    pub drowned: Option<String>,
}

pub fn cross(rng: &mut Rng, way: Way, depth: f64, width: u32, quicksand: bool, party: &mut [Person], supplies: &mut Supplies) -> Crossing {
    let mut out = Crossing::default();
    let (p, worst) = risk(way, depth, width, quicksand);
    if !rng.chance(p) {
        out.lines.push(
            match way {
                Way::Drive if depth < 2.5 => "You drove straight across. The water barely wet the axles.",
                Way::Drive => "The oxen leaned into the current and dragged the wagon across. Everything came over dry enough.",
                Way::Float => "With the box sealed and the oxen swimming alongside, the wagon floated over like a clumsy boat.",
                _ => "The ferryman poled you over without trouble.",
            }
            .into(),
        );
        return out;
    }
    // something went wrong
    if quicksand && way == Way::Drive && rng.chance(0.5) {
        out.lines.push("The wheels sank in quicksand mid-river. You unloaded half the wagon by hand to free it, and lost some of what you carried.".into());
    } else {
        out.lines.push(
            (*rng.pick(&[
                "The current swung the wagon broadside and it tipped.",
                "The wagon struck a hidden bar and lurched over on its side.",
                "The oxen panicked in the deep water and turned downstream.",
            ]))
            .into(),
        );
    }
    let frac = 0.1 + rng.unit() * 0.3;
    let food = supplies.take(Item::Food, (supplies.get(Item::Food) as f64 * frac) as u32);
    let clothes = supplies.take(Item::Clothing, rng.range(0, 2) as u32);
    let shot = supplies.take(Item::Shot, rng.range(0, 2) as u32 * 20);
    let mut lost = Vec::new();
    if food > 0 {
        lost.push(format!("{food} lb of provisions"));
    }
    if clothes > 0 {
        lost.push(crate::game::amount_of((Item::Clothing, clothes)));
    }
    if shot > 0 {
        lost.push(format!("{shot} rounds of shot"));
    }
    if way != Way::Ferry && supplies.get(Item::Oxen) > 0 && rng.chance(0.35) {
        supplies.take(Item::Oxen, 1);
        lost.push("an ox, drowned".into());
    }
    if !lost.is_empty() {
        out.lines.push(format!("Lost: {}.", lost.join(", ")));
    }
    if rng.chance(worst) {
        let living: Vec<usize> = (0..party.len()).filter(|&i| party[i].alive()).collect();
        if !living.is_empty() {
            let i = *rng.pick(&living);
            party[i].health = 0;
            party[i].illness = None;
            party[i].cause = Some("drowning".into());
            out.lines.push(format!("{} was swept away and drowned.", party[i].name));
            out.drowned = Some(party[i].name.clone());
        }
    }
    out
}

/// Rafting the Columbia from The Dalles: the rapids at the Cascades.
pub fn rapids(rng: &mut Rng, party: &mut [Person], supplies: &mut Supplies) -> Vec<String> {
    let mut lines = Vec::new();
    let roll = rng.unit();
    if roll < 0.65 {
        lines.push("You carried the goods round the Cascades and lined the raft down through the rapids on ropes. It held.".into());
    } else if roll < 0.92 {
        let food = supplies.take(Item::Food, supplies.get(Item::Food) / 3);
        let clothes = supplies.take(Item::Clothing, 2);
        lines.push(format!("The raft broke up in the white water at the Cascades. You saved the people and lost {food} lb of provisions and {clothes} sets of clothing."));
    } else {
        let living: Vec<usize> = (0..party.len()).filter(|&i| party[i].alive()).collect();
        if let Some(&i) = living.get(rng.next() as usize % living.len().max(1)) {
            party[i].health = 0;
            party[i].illness = None;
            party[i].cause = Some("drowning".into());
            lines.push(format!("The raft overturned in the Cascades. {} was pulled under and did not come up.", party[i].name));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_fords_are_dangerous_and_ferries_safe() {
        let trials = 2000;
        let count = |way: Way, depth: f64| -> usize {
            let mut bad = 0;
            for seed in 0..trials {
                let mut rng = Rng::new(seed);
                let mut party = vec![Person::new("A"), Person::new("B")];
                let mut s = Supplies::default();
                s.set(Item::Food, 500);
                s.set(Item::Oxen, 6);
                let c = cross(&mut rng, way, depth, 200, false, &mut party, &mut s);
                if c.lines.len() > 1 {
                    bad += 1;
                }
            }
            bad
        };
        let shallow = count(Way::Drive, 1.5);
        let deep = count(Way::Drive, 5.0);
        let ferry = count(Way::Ferry, 5.0);
        assert!(shallow < 150, "{shallow}");
        assert!(deep > 1000, "{deep}");
        assert!(ferry < 80, "{ferry}");
    }
}
