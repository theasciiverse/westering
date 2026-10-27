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
    let sensible = strategy == Strategy::Sensible;
    let mut g = Game::new(seed);
    for _ in 0..6 {
        g.key(Key::Enter);
    }
    // the month: April for the sensible, June for the careless
    let month = if sensible { 1 } else { 3 };
    while g.cursor < month {
        g.key(Key::Down);
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
        buy_until(&mut g, Item::Oxen, 4);
        buy_until(&mut g, Item::Food, 500);
        buy_until(&mut g, Item::Shot, 100);
    }
    g.cursor = ITEMS.len();
    g.key(Key::Enter);
    if !sensible {
        g.journey.pace = Pace::Grueling;
    }
    let mut waited = 0;
    for _ in 0..5000 {
        match g.screen {
            Screen::Trail => {
                if !sensible {
                    g.trail_choice(0);
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
                    Kind::Fort(_) => g.landmark_choice(n - 1),
                    // the sensible keep to the road and pay the toll; the careless cut corners
                    Kind::Fork => g.landmark_choice(if sensible && g.money >= 500 { 1 } else { 0 }.min(n - 1)),
                    _ if l.name == "The Dalles" => g.landmark_choice(if sensible && g.money >= 500 { 0 } else { 1 }),
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
pub fn report(n: u64, first_seed: u64) -> String {
    let mut out = String::new();
    for strategy in [Strategy::Sensible, Strategy::Careless] {
        let (mut arrived, mut alive, mut days, mut score) = (0, 0, 0, 0i64);
        for seed in first_seed..first_seed + n {
            let g = play(seed, strategy);
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
    }
    out
}
