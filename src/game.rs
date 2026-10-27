//! The game's state and its answer to each key. Screens follow one another
//! like the pages of a diary: the title, the party, the store, the trail, and
//! the end.

use crate::date;
use crate::events;
use crate::river::{self, Way};
use crate::party::{self, ITEMS, Item, Person, Supplies};
use crate::rng::Rng;
use crate::route::{self, Kind, Next, ROUTE};
use crate::trail::{self, Journey};
use log::{debug, info};

/// The keys the game knows, the same from the terminal and from a script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    Title,
    /// naming the party; the field being edited, 0 the leader
    Names(usize),
    Month,
    Store,
    /// on the road between landmarks, with the trail menu
    Trail,
    /// arrived at a landmark: its note, and what can be done there
    Landmark,
    /// at a river, deciding how to cross
    River,
    /// a trade offered by a passing party
    Trade(Offer),
    /// the journey is over, arrived or not
    End,
    Quit,
}

/// Give `give` of one thing for `get` of another.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Offer {
    pub give: (Item, u32),
    pub get: (Item, u32),
}

/// The trail menu, in order.
pub const TRAIL_MENU: [&str; 5] = ["Continue on", "Rest a day", "Change the pace", "Change the rations", "Look for a trade"];

pub const PARTY: usize = 5;

pub struct Game {
    pub rng: Rng,
    pub screen: Screen,
    /// the menu row under the cursor, on screens with a menu
    pub cursor: usize,
    pub party: Vec<Person>,
    pub supplies: Supplies,
    /// cents
    pub money: u32,
    /// day of 1848, counted from January 1
    pub day: u32,
    /// one line under the menu: the store's advice, a warning
    pub note: String,
    pub journey: Journey,
    /// the landmark last reached, and the one the wagon is heading for
    pub at: usize,
    pub to: usize,
    /// miles left to `to`
    pub left: f64,
    /// what has happened, a line or two per stretch, newest last
    pub diary: Vec<String>,
    /// the store's prices against Independence
    pub prices: f64,
    /// bought at this store, so it can be put back
    pub bought_here: Supplies,
    /// the day the party left Independence
    pub started: u32,
    /// the depth of the river being crossed today, in feet
    pub depth: f64,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        info!("new game, seed {seed}");
        let mut rng = Rng::new(seed);
        let leader = *rng.pick(&party::LEADER_NAMES);
        let mut people = vec![Person::new(leader)];
        while people.len() < PARTY {
            let name = *rng.pick(&party::COMPANION_NAMES);
            if people.iter().all(|p| p.name != name) {
                people.push(Person::new(name));
            }
        }
        Game {
            rng,
            screen: Screen::Title,
            cursor: 0,
            party: people,
            supplies: Supplies::default(),
            money: party::PURSE,
            day: date::first_of(4),
            note: String::new(),
            journey: Journey::new(),
            at: route::START,
            to: route::START,
            left: 0.0,
            diary: Vec::new(),
            prices: 1.0,
            bought_here: Supplies::default(),
            started: 0,
            depth: 0.0,
        }
    }

    fn go(&mut self, screen: Screen) {
        info!("screen {:?} -> {screen:?}", self.screen);
        self.screen = screen;
        self.cursor = 0;
        self.note.clear();
    }

    pub fn key(&mut self, key: Key) {
        debug!("key {key:?} on {:?}", self.screen);
        match self.screen.clone() {
            Screen::Title => match key {
                Key::Esc | Key::Char('q') => self.go(Screen::Quit),
                Key::Enter => self.go(Screen::Names(0)),
                _ => {}
            },
            Screen::Names(i) => self.names_key(i, key),
            Screen::Month => match key {
                Key::Up => self.cursor = self.cursor.saturating_sub(1),
                Key::Down => self.cursor = (self.cursor + 1).min(party::MONTHS.len() - 1),
                Key::Esc => self.go(Screen::Names(PARTY - 1)),
                Key::Enter => {
                    self.day = date::first_of(3 + self.cursor as u32);
                    info!("leaving in {}", party::MONTHS[self.cursor].0);
                    self.go(Screen::Store);
                    self.note = Item::Oxen.good().advice.into();
                }
                _ => {}
            },
            Screen::Store => self.store_key(key),
            Screen::Trail => self.trail_key(key),
            Screen::Landmark => self.landmark_key(key),
            Screen::River => self.river_key(key),
            Screen::Trade(offer) => self.trade_key(offer, key),
            Screen::End => match key {
                Key::Enter | Key::Esc | Key::Char('q') => self.go(Screen::Quit),
                _ => {}
            },
            Screen::Quit => {}
        }
    }

    fn names_key(&mut self, i: usize, key: Key) {
        let name = &mut self.party[i].name;
        match key {
            Key::Char(c) if name.chars().count() < 16 && (c.is_alphanumeric() || c == ' ' || c == '\'' || c == '-' || c == '.') => name.push(c),
            Key::Backspace => {
                name.pop();
            }
            Key::Up => self.screen = Screen::Names(i.saturating_sub(1)),
            Key::Down => self.screen = Screen::Names((i + 1).min(PARTY - 1)),
            Key::Enter if name.trim().is_empty() => self.note = "Everyone needs a name.".into(),
            Key::Enter if i + 1 < PARTY => {
                self.screen = Screen::Names(i + 1);
                self.note.clear();
            }
            Key::Enter => {
                for p in &mut self.party {
                    p.name = p.name.trim().to_string();
                }
                self.go(Screen::Month);
                self.cursor = 1;
            }
            Key::Esc => self.go(Screen::Title),
            _ => {}
        }
    }

    /// Rows of the store: one per item, then "leave".
    pub const STORE_ROWS: usize = ITEMS.len() + 1;

    fn store_key(&mut self, key: Key) {
        let row = self.cursor;
        match key {
            Key::Up => self.cursor = row.saturating_sub(1),
            Key::Down => self.cursor = (row + 1).min(Self::STORE_ROWS - 1),
            Key::Right | Key::Char('+') | Key::Char('=') if row < ITEMS.len() => self.buy(ITEMS[row]),
            Key::Left | Key::Char('-') if row < ITEMS.len() => self.sell_back(ITEMS[row]),
            Key::Enter if row < ITEMS.len() => self.buy(ITEMS[row]),
            Key::Enter if self.at == route::START => self.leave_town(),
            Key::Enter => self.go(Screen::Landmark),
            Key::Esc if self.at == route::START => self.go(Screen::Month),
            Key::Esc => self.go(Screen::Landmark),
            _ => {}
        }
        if self.cursor != row || matches!(key, Key::Up | Key::Down) {
            self.note = match ITEMS.get(self.cursor) {
                Some(i) => i.good().advice.into(),
                None => "When you are ready, the trail is waiting.".into(),
            };
        }
    }

    pub fn price(&self, item: Item) -> u32 {
        (item.good().price as f64 * self.prices).round() as u32
    }

    fn buy(&mut self, item: Item) {
        let g = item.good();
        let price = self.price(item);
        if self.money < price {
            self.note = "You cannot afford it.".into();
        } else if self.supplies.load() + g.per as f64 * g.weight > party::CAPACITY {
            self.note = "The wagon will not hold any more.".into();
        } else {
            self.money -= price;
            self.supplies.add(item, g.per);
            self.bought_here.add(item, g.per);
            self.note = g.advice.into();
        }
    }

    fn sell_back(&mut self, item: Item) {
        let g = item.good();
        if self.bought_here.get(item) >= g.per {
            self.bought_here.take(item, g.per);
            self.supplies.take(item, g.per);
            self.money += self.price(item);
        }
    }

    fn leave_town(&mut self) {
        if self.supplies.get(Item::Oxen) == 0 {
            self.note = "A wagon will not go far without oxen.".into();
            return;
        }
        if self.supplies.get(Item::Food) == 0 {
            self.note = "You will want something to eat on the way.".into();
            return;
        }
        info!("leaving Independence on {} with {:?}, {} left", date::format(self.day), self.supplies, party::money(self.money));
        self.started = self.day;
        self.diary.push(format!("{}: Left Independence with {} oxen and {} lb of provisions.", date::format(self.day), self.supplies.get(Item::Oxen), self.supplies.get(Item::Food)));
        self.set_out(route::START);
    }

    /// Head for the landmark after `from` (one with a single road on).
    fn set_out(&mut self, from: usize) {
        if let Next::On(to, miles) = ROUTE[from].next {
            self.head_for(to, miles);
        }
    }

    fn head_for(&mut self, to: usize, miles: f64) {
        self.to = to;
        self.left = miles;
        self.journey.region = ROUTE[to].region;
        self.go(Screen::Trail);
    }

    pub fn alive(&self) -> usize {
        self.party.iter().filter(|p| p.alive()).count()
    }

    fn trail_key(&mut self, key: Key) {
        match key {
            Key::Up => self.cursor = self.cursor.saturating_sub(1),
            Key::Down => self.cursor = (self.cursor + 1).min(TRAIL_MENU.len() - 1),
            Key::Char(c @ '1'..='9') => {
                let i = c as usize - '1' as usize;
                if i < TRAIL_MENU.len() {
                    self.cursor = i;
                    self.trail_choice(i);
                }
            }
            Key::Enter => self.trail_choice(self.cursor),
            _ => {}
        }
    }

    fn trail_choice(&mut self, i: usize) {
        match i {
            0 => self.travel(),
            1 => {
                let from = self.day;
                let d = self.live_day(true);
                self.diary.push(format!("{}: Rested a day.", date::format(from)));
                for (name, what) in d.recovered {
                    self.diary.push(format!("{name} is well again after {what}."));
                }
                for name in &d.died {
                    let o = self.obituary(name);
                    self.diary.push(o);
                }
                self.after_days();
            }
            2 => {
                self.journey.pace = self.journey.pace.next();
                self.note = format!("The pace is now {}.", self.journey.pace.name());
            }
            3 => {
                self.journey.rations = self.journey.rations.next();
                self.note = format!("Rations are now {}.", self.journey.rations.name());
            }
            _ => self.look_for_trade(),
        }
    }

    /// One day lived, travelling or resting; returns the day's miles.
    fn live_day(&mut self, resting: bool) -> trail::Day {
        let d = trail::day(&mut self.rng, &mut self.journey, self.day, &mut self.party, &mut self.supplies, resting);
        self.day += 1;
        for name in &d.died {
            info!("{name} died on {}", date::format(self.day));
        }
        for (name, what) in &d.recovered {
            info!("{name} got over {what}");
        }
        d
    }

    /// "Ada has died of cholera."
    fn obituary(&self, name: &str) -> String {
        let cause = self.party.iter().find(|p| p.name == name).and_then(|p| p.cause.clone()).unwrap_or_else(|| "exhaustion".into());
        format!("{name} has died of {cause}.")
    }

    /// Travel on for up to a week, stopping at the next landmark or when
    /// something happens that the party must deal with.
    fn travel(&mut self) {
        if self.supplies.get(Item::Oxen) < 2 {
            self.note = "You have no team to pull the wagon.".into();
            return;
        }
        let from = self.day;
        let (mut miles, mut hungry, mut cold, mut died) = (0.0, false, false, Vec::new());
        let mut arrived = false;
        let mut happened = Vec::new();
        for _ in 0..7 {
            let d = self.live_day(false);
            miles += d.miles;
            self.left -= d.miles;
            hungry |= d.hungry;
            cold |= d.cold;
            let mut stop = !d.died.is_empty();
            died.extend(d.died);
            for (name, what) in d.recovered {
                happened.push(format!("{}: {name} is well again after {what}.", date::format(self.day - 1)));
            }
            if self.left <= 0.0 {
                // the last day's overshoot is the landmark's own ground
                miles += self.left;
                self.journey.miles += self.left;
                self.left = 0.0;
                arrived = true;
                break;
            }
            if self.alive() > 0
                && let Some(h) = events::roll(&mut self.rng, &mut self.journey, self.day - 1, &mut self.party, &mut self.supplies, self.started)
            {
                info!("event: {}", h.text);
                happened.push(format!("{}: {}", date::format(self.day - 1), h.text));
                for _ in 0..h.days_lost {
                    let d = self.live_day(true);
                    died.extend(d.died);
                }
                stop |= h.stop || h.days_lost > 0;
            }
            if stop || self.alive() == 0 {
                break;
            }
        }
        let mut line = format!("{} to {}: {:.0} miles, {}.", date::short(from), date::short(self.day - 1), miles, self.journey.weather.describe());
        if hungry {
            line.push_str(" Not enough to eat.");
        }
        if cold {
            line.push_str(" Cold without enough clothing.");
        }
        self.diary.push(line);
        self.diary.extend(happened);
        for name in &died {
            let o = self.obituary(name);
            self.diary.push(o);
        }
        if arrived && self.alive() > 0 {
            self.arrive();
        } else {
            self.after_days();
        }
    }

    /// After days on the road: the end, if nobody is left.
    fn after_days(&mut self) {
        if self.alive() == 0 {
            self.diary.push("There is no one left to go on.".into());
            info!("all dead on {}", date::format(self.day));
            self.go(Screen::End);
        }
    }

    fn arrive(&mut self) {
        self.at = self.to;
        let l = &ROUTE[self.at];
        info!("reached {} on {}, {:.0} miles", l.name, date::format(self.day), self.journey.miles);
        self.diary.push(format!("{}: Reached {}.", date::format(self.day), l.name));
        if let Kind::River { depth, .. } = l.kind {
            self.depth = river::depth_on(&mut self.rng, depth, self.day);
        }
        if l.name == "Fort Vancouver" {
            let lines = river::rapids(&mut self.rng, &mut self.party, &mut self.supplies);
            self.diary.extend(lines);
            if self.alive() == 0 {
                self.go(Screen::End);
                return;
            }
        }
        if l.kind == Kind::End {
            self.go(Screen::End);
        } else {
            self.go(Screen::Landmark);
        }
    }

    /// The choices at the landmark just reached.
    pub fn landmark_choices(&self) -> Vec<String> {
        let l = &ROUTE[self.at];
        match (l.kind, l.next) {
            (_, Next::Fork(roads)) => roads.iter().map(|r| r.0.to_string()).collect(),
            (Kind::Fort(_), _) => vec!["Visit the trade room".into(), "Go on".into()],
            (Kind::River { .. }, _) => vec!["Cross the river".into()],
            _ => vec!["Go on".into()],
        }
    }

    fn landmark_key(&mut self, key: Key) {
        let n = self.landmark_choices().len();
        match key {
            Key::Up => self.cursor = self.cursor.saturating_sub(1),
            Key::Down => self.cursor = (self.cursor + 1).min(n - 1),
            Key::Enter => self.landmark_choice(self.cursor),
            _ => {}
        }
    }

    fn landmark_choice(&mut self, i: usize) {
        let l = &ROUTE[self.at];
        match (l.kind, l.next) {
            (_, Next::Fork(roads)) => {
                let (label, to, miles) = roads[i];
                if label.contains("toll") {
                    if self.money < 500 {
                        self.note = "You cannot pay the toll.".into();
                        return;
                    }
                    self.money -= 500;
                }
                self.diary.push(format!("{}: {}.", date::format(self.day), label));
                if label.contains("Raft") {
                    for _ in 0..3 {
                        self.live_day(true);
                    }
                    self.diary.push("Three days felling pines and lashing them into a raft. The wagon goes on it in pieces.".into());
                }
                self.head_for(to, miles);
            }
            (Kind::Fort(f), _) if i == 0 => {
                self.prices = f;
                self.bought_here = Supplies::default();
                self.go(Screen::Store);
                self.note = format!("Prices here are {:.1} times those in Independence.", f);
            }
            (Kind::River { .. }, _) => self.go(Screen::River),
            _ => self.set_out(self.at),
        }
    }

    /// The ways over the river here, with what each says.
    pub fn river_choices(&self) -> Vec<(Way, String)> {
        let Kind::River { ferry, .. } = ROUTE[self.at].kind else { return Vec::new() };
        let mut v = vec![(Way::Drive, "Drive the team straight across".to_string()), (Way::Float, "Seal the wagon box and float it over (a day)".to_string())];
        if let Some(cents) = ferry {
            v.push((Way::Ferry, format!("Pay the ferry ({})", party::money(cents))));
        }
        v.push((Way::Wait, "Camp a day and see if the water falls".into()));
        v
    }

    fn river_key(&mut self, key: Key) {
        let choices = self.river_choices();
        match key {
            Key::Up => self.cursor = self.cursor.saturating_sub(1),
            Key::Down => self.cursor = (self.cursor + 1).min(choices.len() - 1),
            Key::Esc => self.go(Screen::Landmark),
            Key::Enter => self.river_choice(choices[self.cursor].0),
            _ => {}
        }
    }

    fn river_choice(&mut self, way: Way) {
        let Kind::River { width, ferry, .. } = ROUTE[self.at].kind else { return };
        let name = ROUTE[self.at].name;
        match way {
            Way::Wait => {
                self.live_day(true);
                self.depth = river::wait(&mut self.rng, self.depth);
                self.note = format!("A day in camp. {}", river::describe(self.depth, width));
                self.after_days();
                return;
            }
            Way::Ferry => {
                let cents = ferry.unwrap_or(0);
                if self.money < cents {
                    self.note = "You cannot pay the ferry.".into();
                    return;
                }
                self.money -= cents;
            }
            Way::Float => {
                self.live_day(true);
            }
            Way::Drive => {}
        }
        let quicksand = name.contains("South Platte");
        let c = river::cross(&mut self.rng, way, self.depth, width, quicksand, &mut self.party, &mut self.supplies);
        info!("crossed {name} by {way:?} at {:.1} ft: {:?}", self.depth, c.lines);
        self.diary.push(format!("{}: Crossed the {}.", date::format(self.day), river::river_name(name)));
        self.diary.extend(c.lines);
        if self.alive() == 0 {
            self.after_days();
            return;
        }
        self.set_out(self.at);
    }

    /// A day spent asking the other trains what they would trade.
    fn look_for_trade(&mut self) {
        self.live_day(true);
        let wants = [Item::Oxen, Item::Food, Item::Clothing, Item::Shot, Item::Wheel, Item::Axle, Item::Tongue];
        let give = *self.rng.pick(&wants);
        let mut get = *self.rng.pick(&wants);
        while get == give {
            get = *self.rng.pick(&wants);
        }
        let amount = |item: Item, rng: &mut Rng| -> u32 {
            match item {
                Item::Oxen => 1,
                Item::Food => rng.range(3, 10) as u32 * 10,
                Item::Clothing => rng.range(1, 3) as u32,
                Item::Shot => rng.range(1, 4) as u32 * 20,
                _ => 1,
            }
        };
        let offer = Offer { give: (give, amount(give, &mut self.rng)), get: (get, amount(get, &mut self.rng)) };
        if self.rng.chance(0.25) {
            self.diary.push(format!("{}: Nobody on the road had anything to trade.", date::format(self.day - 1)));
            self.after_days();
            return;
        }
        info!("trade offer {offer:?}");
        self.go(Screen::Trade(offer));
    }

    fn trade_key(&mut self, offer: Offer, key: Key) {
        match key {
            Key::Char('y') | Key::Enter if self.cursor == 0 || key == Key::Char('y') => {
                let (item, n) = offer.give;
                if self.supplies.get(item) < n {
                    self.note = "You do not have enough to trade.".into();
                    return;
                }
                self.supplies.take(item, n);
                self.supplies.add(offer.get.0, offer.get.1);
                self.diary.push(format!("{}: Traded {} for {}.", date::format(self.day - 1), amount_of(offer.give), amount_of(offer.get)));
                self.go(Screen::Trail);
            }
            Key::Char('n') | Key::Enter | Key::Esc => self.go(Screen::Trail),
            Key::Up | Key::Down => self.cursor = 1 - self.cursor.min(1),
            _ => {}
        }
    }

    pub fn next_landmark(&self) -> &'static str {
        ROUTE[self.to].name
    }

    pub fn done(&self) -> bool {
        self.screen == Screen::Quit
    }
}

/// "3 oxen", "40 lb of provisions", in words.
pub fn amount_of((item, n): (Item, u32)) -> String {
    match item {
        Item::Oxen => format!("{n} {}", if n == 1 { "ox" } else { "oxen" }),
        Item::Food => format!("{n} lb of provisions"),
        Item::Clothing => format!("{n} {}", if n == 1 { "set of clothing" } else { "sets of clothing" }),
        Item::Shot => format!("{n} rounds of shot"),
        Item::Wheel => format!("{n} spare {}", if n == 1 { "wheel" } else { "wheels" }),
        Item::Axle => format!("{n} spare {}", if n == 1 { "axle" } else { "axles" }),
        Item::Tongue => format!("{n} spare {}", if n == 1 { "tongue" } else { "tongues" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script_keys;

    pub fn play(seed: u64, keys: &str) -> Game {
        let mut g = Game::new(seed);
        for k in script_keys(keys) {
            g.key(k);
        }
        g
    }

    #[test]
    fn naming_choosing_a_month_and_shopping() {
        // rename the leader, keep the rest, leave in May
        let g = play(1, ">");
        assert_eq!(g.screen, Screen::Names(0));
        let keys = format!(">{}Ada>>>>>_>", "<".repeat(20));
        let mut g = play(1, &keys);
        assert_eq!(g.party[0].name, "Ada");
        assert_eq!(g.screen, Screen::Store);
        assert_eq!(date::format(g.day), "May 1, 1848");
        g.key(Key::Enter);
        g.key(Key::Enter);
        assert_eq!(g.supplies.get(Item::Oxen), 4);
        assert_eq!(g.money, party::PURSE - 10000);
        g.key(Key::Left);
        assert_eq!(g.supplies.get(Item::Oxen), 2);
        // no food yet: the store will not let you go
        for _ in 0..ITEMS.len() {
            g.key(Key::Down);
        }
        g.key(Key::Enter);
        assert_eq!(g.screen, Screen::Store);
        g.cursor = 1;
        g.key(Key::Right);
        g.cursor = ITEMS.len();
        g.key(Key::Enter);
        assert_eq!(g.screen, Screen::Trail);
    }

    #[test]
    fn the_purse_and_the_wagon_have_limits() {
        let mut g = play(2, ">>>>>>>");
        assert_eq!(g.screen, Screen::Store);
        g.cursor = 1;
        for _ in 0..100 {
            g.key(Key::Right);
        }
        assert!(g.supplies.load() <= party::CAPACITY);
        g.cursor = 0;
        for _ in 0..100 {
            g.key(Key::Right);
        }
        assert!(g.money < Item::Oxen.good().price);
    }

    /// Outfit sensibly, then always go on; the journey must end either way.
    pub fn bot(seed: u64) -> Game {
        let mut g = play(seed, ">>>>>>>");
        for (row, n) in [(0, 3), (1, 20), (2, 10), (3, 5), (4, 1), (5, 1), (6, 1)] {
            g.cursor = row;
            for _ in 0..n {
                g.key(Key::Right);
            }
        }
        g.cursor = ITEMS.len();
        g.key(Key::Enter);
        for _ in 0..2000 {
            match g.screen {
                Screen::Trail => g.trail_choice(0),
                Screen::Landmark => {
                    let n = g.landmark_choices().len();
                    let pick = if matches!(ROUTE[g.at].kind, Kind::Fort(_)) { n - 1 } else { 0 };
                    g.landmark_choice(pick);
                }
                Screen::River => {
                    let choices = g.river_choices();
                    let has = |w: Way| choices.iter().any(|c| c.0 == w);
                    let way = if g.depth < 3.0 { Way::Drive } else if has(Way::Ferry) && g.money >= 500 { Way::Ferry } else { Way::Float };
                    g.river_choice(way);
                }
                Screen::Trade(_) => g.key(Key::Char('n')),
                Screen::End => break,
                ref s => panic!("bot stuck on {s:?}"),
            }
        }
        g
    }

    #[test]
    fn a_journey_ends() {
        for seed in 0..20 {
            let g = bot(seed);
            assert_eq!(g.screen, Screen::End, "seed {seed}");
            eprintln!("seed {seed}: {} alive, {:.0} miles, {}", g.alive(), g.journey.miles, date::format(g.day));
        }
    }
}
