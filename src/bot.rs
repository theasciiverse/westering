//! A player made of rules, for testing and balancing: it plays whole
//! journeys headless, sensibly or carelessly. `westering --bot N` runs N of
//! each and prints how they fared.

use crate::date;
use crate::game::{Game, Key, Screen};
use crate::party::{ITEMS, Item};
use crate::river::Way;
use crate::route::{Kind, ROUTE};
use crate::trail::{Pace, Rations};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    Sensible,
    Careless,
}

fn buy_until(g: &mut Game, item: Item, want: u32) {
    let row = ITEMS.iter().position(|&i| i == item).unwrap_or(0);
    g.cursor = row;
    let mut guard = 0;
    while g.supplies.get(item) < want && guard < 200 {
        let before = g.supplies.get(item);
        g.buy(item);
        if g.supplies.get(item) == before {
            break;
        }
        guard += 1;
    }
}

/// Outfit, then play until the journey ends.
pub fn play(seed: u64, strategy: Strategy) -> Game {
    play_from(seed, strategy, None)
}

/// Play, leaving in the given month (3 March to 7 July) instead of the
/// strategy's own.
pub fn play_from(seed: u64, strategy: Strategy, month: Option<u32>) -> Game {
    play_steps(seed, strategy, month, 5000)
}

/// Play only so many decisions into the journey.
#[cfg(test)]
pub fn play_until(seed: u64, strategy: Strategy, steps: usize) -> Game {
    play_steps(seed, strategy, None, steps)
}

fn play_steps(seed: u64, strategy: Strategy, month: Option<u32>, steps: usize) -> Game {
    let sensible = strategy == Strategy::Sensible;
    let mut g = Game::new(seed);
    for _ in 0..6 {
        g.key(Key::Enter);
    }
    // the month: April for the sensible, May for the careless
    let month = month.map_or(if sensible { 1 } else { 2 }, |m| m.clamp(3, 7) as usize - 3);
    while g.cursor < month {
        g.key(Key::Down);
    }
    while g.cursor > month {
        g.key(Key::Up);
    }
    g.key(Key::Enter);
    if sensible {
        buy_until(&mut g, Item::Oxen, 8);
        buy_until(&mut g, Item::Food, 1200);
        buy_until(&mut g, Item::Clothing, 10);
        buy_until(&mut g, Item::Shot, 200);
        for part in [Item::Wheel, Item::Axle, Item::Tongue] {
            buy_until(&mut g, part, 1);
        }
    } else {
        // enough to start, nothing to spare
        buy_until(&mut g, Item::Oxen, 4);
        buy_until(&mut g, Item::Food, 900);
        buy_until(&mut g, Item::Clothing, 3);
        buy_until(&mut g, Item::Shot, 100);
    }
    g.cursor = ITEMS.len();
    g.key(Key::Enter);
    if !sensible {
        g.journey.pace = Pace::Grueling;
    }
    let mut waited = 0;
    for _ in 0..steps {
        match g.screen {
            Screen::Trail => {
                if !sensible {
                    // hunt only once the flour has run out
                    if g.supplies.get(Item::Food) < 15 && g.hunted < 3 && g.supplies.get(Item::Shot) >= crate::hunt::SHOT_NEEDED {
                        g.trail_choice(2);
                    } else {
                        g.trail_choice(0);
                    }
                    continue;
                }
                let alive = g.alive() as u32;
                let food_days = g.supplies.get(Item::Food) / alive.max(1);
                g.journey.rations = match food_days {
                    d if d >= 3 * 12 => Rations::Filling,
                    d if d >= 2 * 6 => Rations::Meager,
                    _ => Rations::Bare,
                };
                let late = g.day >= date::first_of(8) + 15;
                g.journey.pace = if late { Pace::Hard } else { Pace::Steady };
                let weak = g.party.iter().filter(|p| p.alive()).any(|p| p.health < 45);
                if food_days < 20 && g.hunted < 3 && g.supplies.get(Item::Shot) >= crate::hunt::SHOT_NEEDED {
                    g.trail_choice(2);
                } else if weak && food_days >= 4 {
                    g.trail_choice(1);
                } else {
                    g.trail_choice(0);
                }
            }
            Screen::Landmark => {
                let l = &ROUTE[g.at];
                let n = g.landmark_choices().len();
                match l.kind {
                    Kind::Fort(_) if sensible => {
                        g.landmark_choice(0);
                        let alive = g.alive() as u32;
                        buy_until(&mut g, Item::Food, alive * 3 * 30);
                        buy_until(&mut g, Item::Oxen, 6);
                        buy_until(&mut g, Item::Clothing, alive * 2);
                        g.cursor = ITEMS.len();
                        g.key(Key::Enter);
                        g.landmark_choice(1);
                    }
                    Kind::Fort(_) => {
                        // tops up the flour, no more
                        g.landmark_choice(0);
                        buy_until(&mut g, Item::Food, 700);
                        g.cursor = ITEMS.len();
                        g.key(Key::Enter);
                        g.landmark_choice(n - 1);
                    }
                    // the sensible keep to the road and pay the toll; the careless cut corners
                    _ if l.name == "The Dalles" => g.landmark_choice(if sensible && g.money >= 500 { 0 } else { 1 }),
                    Kind::Fork => g.landmark_choice(if sensible { 1 } else { 0 }.min(n - 1)),
                    _ => g.landmark_choice(0),
                }
            }
            Screen::River => {
                let choices = g.river_choices();
                let has = |w: Way| choices.iter().any(|c| c.0 == w);
                let way = if !sensible || g.depth < 2.5 {
                    Way::Drive
                } else if has(Way::Ferry) && g.money >= 500 {
                    Way::Ferry
                } else if g.depth > 4.0 && waited < 3 {
                    waited += 1;
                    Way::Wait
                } else {
                    Way::Float
                };
                if way != Way::Wait {
                    waited = 0;
                }
                g.river_choice(way);
            }
            Screen::Trade(_) => g.key(Key::Char('n')),
            Screen::Epitaph(_) => g.key(Key::Enter),
            Screen::End | Screen::Quit => break,
            ref s => panic!("bot stuck on {s:?}"),
        }
    }
    g
}

/// Play `n` journeys of each kind and report.
pub fn report(n: u64, first_seed: u64, month: Option<u32>) -> String {
    let mut out = String::new();
    for strategy in [Strategy::Sensible, Strategy::Careless] {
        let (mut arrived, mut alive, mut days, mut score) = (0, 0, 0, 0i64);
        let mut whole = 0;
        let mut causes: std::collections::BTreeMap<String, u32> = Default::default();
        let mut lost_at: std::collections::BTreeMap<&str, u32> = Default::default();
        for seed in first_seed..first_seed + n {
            let g = play_from(seed, strategy, month);
            for p in g.party.iter().filter(|p| !p.alive()) {
                *causes.entry(p.cause.clone().unwrap_or_default()).or_default() += 1;
            }
            if g.alive() == 5 && g.arrived() {
                whole += 1;
            }
            if !g.arrived() {
                *lost_at.entry(ROUTE[g.to].name).or_default() += 1;
            }
            if g.arrived() {
                arrived += 1;
                alive += g.alive();
                days += g.day - g.started;
                score += g.score().iter().map(|s| s.1).sum::<i64>();
            }
        }
        let per = |x: f64| if arrived > 0 { x / arrived as f64 } else { 0.0 };
        out.push_str(&format!(
            "{strategy:?}: {arrived} of {n} arrived ({:.0}%), {:.1} of 5 alive on arrival, {:.0} days, score {:.0}\n",
            100.0 * arrived as f64 / n as f64,
            per(alive as f64),
            per(days as f64),
            per(score as f64)
        ));
        out.push_str(&format!("  all five arrived: {whole}\n  deaths: {causes:?}\n  lost on the way to: {lost_at:?}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The balance the game is tuned to (PLAN.md, M9): a sensible party
    /// usually gets everyone through, a careless one almost never arrives.
    #[test]
    fn sensible_parties_mostly_arrive_whole_and_careless_ones_rarely_arrive() {
        let n = 200;
        let whole = (0..n).filter(|&s| {
            let g = play(s, Strategy::Sensible);
            g.arrived() && g.alive() == 5
        });
        let whole = whole.count();
        let careless = (0..n).filter(|&s| play(s, Strategy::Careless).arrived()).count();
        assert!((130..=165).contains(&whole), "sensible, whole party: {whole} of {n}");
        assert!(careless <= 20, "careless arrived: {careless} of {n}");
    }
}
