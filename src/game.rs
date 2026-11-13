//! The game's state and its answer to each key. Screens follow one another
//! like the pages of a diary: the title, the party, the store, the trail, and
//! the end.

use crate::date;
use crate::events;
use crate::hunt;
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
    /// a fifth of a second passing, for the hunting scene
    Tick,
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
    /// how to hunt: stalk by hand, or send the hunters out
    HuntChoice,
    /// stalking game on foot (the scene is in `Game::stalk`)
    Hunt,
    /// writing the words for the grave of the dead person at this index
    Epitaph(usize),
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
pub const TRAIL_MENU: [&str; 6] = ["Continue on", "Rest a day", "Go hunting", "Change the pace", "Change the rations", "Look for a trade"];

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
    /// the epitaph being written
    pub epitaph_text: String,
    /// the words are still the ones offered: the first letter typed replaces them
    pub epitaph_offered: bool,
    /// days hunted on this stretch of road
    pub hunted: u32,
    /// the wagon has been left behind
    pub on_foot: bool,
    /// miles from Independence of each landmark reached
    pub passed: Vec<f64>,
    /// Esc on the trail page: asking whether to leave the journey
    pub leaving: bool,
    /// where in the diary the road from the last landmark began, so a
    /// landmark page can show what happened on the way to it
    pub stretch: usize,
    /// a journey is kept from an earlier session, to take up from the title
    pub can_resume: bool,
    /// the hunting scene under way
    pub stalk: Option<crate::stalk::Stalk>,
    /// the journeys made before this one, from the register
    pub records: Vec<crate::save::Record>,
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
            epitaph_text: String::new(),
            epitaph_offered: false,
            hunted: 0,
            on_foot: false,
            passed: Vec::new(),
            leaving: false,
            stretch: 0,
            can_resume: false,
            stalk: None,
            records: Vec::new(),
        }
    }

    fn go(&mut self, screen: Screen) {
        info!("screen {:?} -> {screen:?}", self.screen);
        self.screen = screen;
        self.cursor = 0;
        self.note.clear();
        self.leaving = false;
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
            Screen::HuntChoice => match key {
                Key::Char('1') => self.start_stalk(),
                Key::Char('2') => {
                    self.go(Screen::Trail);
                    self.go_hunting();
                }
                Key::Up | Key::Down => self.cursor = 1 - self.cursor.min(1),
                Key::Enter if self.cursor == 0 => self.start_stalk(),
                Key::Enter => {
                    self.go(Screen::Trail);
                    self.go_hunting();
                }
                Key::Esc => self.go(Screen::Trail),
                _ => {}
            },
            Screen::Hunt => self.hunt_key(key),
            Screen::Epitaph(i) => self.epitaph_key(i, key),
            Screen::End => match key {
                Key::Enter | Key::Char('y') => {
                    let seed = self.rng.next();
                    let mut records = std::mem::take(&mut self.records);
                    records.push(crate::save::Record::of(self));
                    *self = Game::new(seed);
                    self.records = records;
                }
                Key::Esc | Key::Char('q') | Key::Char('n') => self.go(Screen::Quit),
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
        if self.screen == Screen::Store && (self.cursor != row || matches!(key, Key::Up | Key::Down)) {
            self.note = match ITEMS.get(self.cursor) {
                Some(i) => i.good().advice.into(),
                None => "When you are ready, the trail is waiting.".into(),
            };
        }
    }

    pub fn price(&self, item: Item) -> u32 {
        (item.good().price as f64 * self.prices).round() as u32
    }

    pub(crate) fn buy(&mut self, item: Item) {
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
        self.diary.push(format!("{}: Left Independence with {} oxen and {} lb of provisions.", date::short(self.day), self.supplies.get(Item::Oxen), self.supplies.get(Item::Food)));
        self.set_out(route::START);
    }

    /// Head for the landmark after `from` (one with a single road on).
    fn set_out(&mut self, from: usize) {
        if let Next::On(to, miles) = ROUTE[from].next {
            self.head_for(to, miles);
        }
    }

    fn head_for(&mut self, to: usize, miles: f64) {
        self.hunted = 0;
        self.stretch = self.diary.len();
        self.to = to;
        self.left = miles;
        self.journey.region = ROUTE[to].region;
        self.go(Screen::Trail);
    }

    pub fn alive(&self) -> usize {
        self.party.iter().filter(|p| p.alive()).count()
    }

    /// Esc asked whether to leave for now; the answer. True when the key
    /// was taken by the question.
    fn leaving_key(&mut self, key: Key) -> bool {
        if !self.leaving {
            return false;
        }
        match key {
            Key::Char('y') => {
                info!("left the trail on {} at mile {:.0}", date::format(self.day), self.journey.miles);
                self.go(Screen::Quit);
            }
            Key::Char('n') | Key::Esc => self.leaving = false,
            _ => {}
        }
        true
    }

    fn trail_key(&mut self, key: Key) {
        if self.leaving_key(key) {
            return;
        }
        match key {
            Key::Esc => self.leaving = true,
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

    pub(crate) fn trail_choice(&mut self, i: usize) {
        match i {
            0 => self.travel(),
            1 => {
                let from = self.day;
                let d = self.live_day(true);
                self.diary.push(format!("{}: Rested a day.", date::short(from)));
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
                if self.supplies.get(Item::Shot) < hunt::SHOT_NEEDED {
                    self.note = "You have too little powder and shot to hunt.".into();
                } else {
                    self.go(Screen::HuntChoice);
                }
            }
            3 => {
                self.journey.pace = self.journey.pace.next();
                self.note = format!("The pace is now {}.", self.journey.pace.name());
            }
            4 => {
                self.journey.rations = self.journey.rations.next();
                self.note = format!("Rations are now {}.", self.journey.rations.name());
            }
            _ => self.look_for_trade(),
        }
    }

    /// With no team left the wagon stays behind, and the party walks on with
    /// what it can carry on its backs and the last ox, if there is one.
    fn abandon_wagon(&mut self) {
        self.on_foot = true;
        let alive = self.alive() as u32;
        let s = &mut self.supplies;
        let food = s.get(Item::Food).min(40 * alive + 60 * s.get(Item::Oxen));
        s.set(Item::Food, food);
        for part in [Item::Wheel, Item::Axle, Item::Tongue] {
            s.set(part, 0);
        }
        info!("wagon abandoned, walking on with {food} lb");
        self.diary.push(format!("{}: With no team to pull it, you leave the wagon by the road and walk on with {food} lb of provisions on your backs.", date::short(self.day)));
    }

    /// The Sublette Cutoff: fifty miles without water, driven through the
    /// night. It tells on the oxen and the people, and may cost an ox.
    fn dry_drive(&mut self) {
        let j = &mut self.journey;
        j.oxen_health = (j.oxen_health - 20.0).max(0.0);
        for p in self.party.iter_mut().filter(|p| p.alive()) {
            p.health = (p.health - 5).max(1);
        }
        let mut line = "You fill every keg and drive through the night across the dry stretch. The oxen go two days without water.".to_string();
        if self.supplies.get(Item::Oxen) > 0 && self.rng.chance(0.3) {
            self.supplies.take(Item::Oxen, 1);
            line.push_str(" One drops in the yoke and is left behind.");
        }
        self.diary.push(line);
    }

    fn hands(&self) -> u32 {
        self.party.iter().filter(|p| p.alive() && p.illness.is_none()).count() as u32
    }

    /// Out on foot with the rifle: the hunting scene.
    fn start_stalk(&mut self) {
        let seed = self.rng.next();
        let rounds = self.supplies.get(Item::Shot);
        let carry = hunt::CARRY_EACH * self.hands().max(1);
        let s = crate::stalk::Stalk::new(seed, self.journey.region, self.day, self.hunted, rounds, carry);
        info!("stalking: {} animals, {rounds} rounds, carry {carry}", s.animals.len());
        self.stalk = Some(s);
        self.go(Screen::Hunt);
    }

    fn hunt_key(&mut self, key: Key) {
        let Some(s) = self.stalk.as_mut() else {
            self.go(Screen::Trail);
            return;
        };
        if s.over {
            if matches!(key, Key::Enter | Key::Esc | Key::Char(' ')) {
                self.end_stalk();
            }
            return;
        }
        match key {
            Key::Up => s.step(0, -1),
            Key::Down => s.step(0, 1),
            Key::Left => s.step(-1, 0),
            Key::Right => s.step(1, 0),
            Key::Char('q') => s.step(-1, -1),
            Key::Char('e') => s.step(1, -1),
            Key::Char('z') => s.step(-1, 1),
            Key::Char('c') => s.step(1, 1),
            Key::Char(' ') | Key::Char('f') => s.fire(),
            Key::Enter | Key::Esc => {
                s.quit();
                s.message = "You turn back for the wagons.".into();
            }
            Key::Tick => s.tick(),
            _ => {}
        }
    }

    /// Back at the wagon with what the party could carry.
    fn end_stalk(&mut self) {
        let Some(s) = self.stalk.take() else { return };
        let from = self.day;
        let (down, carried) = s.meat();
        self.hunted += 1;
        self.supplies.take(Item::Shot, s.fired);
        self.supplies.add(Item::Food, carried);
        let rounds = match s.fired {
            0 => "You never fired.".to_string(),
            1 => "One round fired.".to_string(),
            n => format!("{n} rounds fired."),
        };
        let text = match s.bag() {
            Some(bag) if carried < down => format!("You brought down {bag}: {down} lb of meat, and between you carried {carried} lb back to the wagon. {rounds}"),
            Some(bag) => format!("You brought down {bag}: {down} lb of meat. {rounds}"),
            None => format!("You came back with nothing. {rounds}"),
        };
        info!("stalked: {down} lb down, {carried} carried, {} rounds", s.fired);
        self.go(Screen::Trail);
        let d = self.live_day(true);
        self.diary.push(format!("{}: Stalking. {text}", date::short(from)));
        for name in &d.died {
            let o = self.obituary(name);
            self.diary.push(o);
        }
        self.after_days();
    }

    /// A day's hunting by the party, if there is shot for it: the quick way.
    pub(crate) fn go_hunting(&mut self) {
        let shot = self.supplies.get(Item::Shot);
        if shot < hunt::SHOT_NEEDED {
            self.note = "You have too little powder and shot to hunt.".into();
            return;
        }
        let from = self.day;
        let hands = self.hands();
        let h = hunt::hunt(&mut self.rng, self.journey.region, self.day, self.hunted, shot, hands);
        self.hunted += 1;
        self.supplies.take(Item::Shot, h.shot);
        self.supplies.add(Item::Food, h.meat);
        info!("hunted: {} lb, {} rounds", h.meat, h.shot);
        let d = self.live_day(true);
        self.diary.push(format!("{}: Hunting. {}", date::short(from), h.text));
        for name in &d.died {
            let o = self.obituary(name);
            self.diary.push(o);
        }
        self.after_days();
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
        self.bury(&d.died, self.day - 1);
        d
    }

    /// Mark the graves of those who died today.
    fn bury(&mut self, died: &[String], day: u32) {
        let place = if self.left <= 0.0 { format!("at {}", ROUTE[self.at].name) } else { format!("on the way to {}", ROUTE[self.to].name) };
        let when = date::format(day);
        for p in self.party.iter_mut().filter(|p| !p.alive() && p.grave.is_none()) {
            if died.contains(&p.name) || p.cause.as_deref() == Some("drowning") {
                p.grave = Some(format!("{when}, {place}"));
            }
        }
    }

    /// "Ada has died of cholera."
    fn obituary(&self, name: &str) -> String {
        let cause = self.party.iter().find(|p| p.name == name).and_then(|p| p.cause.clone()).unwrap_or_else(|| "exhaustion".into());
        format!("{name} has died of {cause}.")
    }

    /// Travel on for up to a week, stopping at the next landmark or when
    /// something happens that the party must deal with.
    fn travel(&mut self) {
        if self.supplies.get(Item::Oxen) < 2 && !self.on_foot {
            self.abandon_wagon();
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
                happened.push(format!("{}: {name} is well again after {what}.", date::short(self.day - 1)));
            }
            if self.left <= 0.0 {
                // the last day's overshoot is the landmark's own ground
                miles += self.left;
                self.journey.miles += self.left;
                self.left = 0.0;
                arrived = true;
                break;
            }
            let snow = if self.alive() > 0 && !self.on_raft() { events::snow(&mut self.rng, &mut self.journey, self.day - 1, &mut self.party, &mut self.supplies) } else { None };
            if let Some(h) = snow.or_else(|| if self.alive() > 0 { events::roll(&mut self.rng, &mut self.journey, self.day - 1, &mut self.party, &mut self.supplies, self.started) } else { None }) {
                info!("event: {}", h.text);
                happened.push(format!("{}: {}", date::short(self.day - 1), h.text));
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
        let when = if self.day - 1 > from { format!("{} to {}", date::short(from), date::short(self.day - 1)) } else { date::short(from) };
        let miles = miles.round() as u32;
        let mut line = format!("{when}: {miles} {}, {}.", if miles == 1 { "mile" } else { "miles" }, self.journey.weather.describe());
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
            self.finish();
        }
    }

    fn arrive(&mut self) {
        self.at = self.to;
        self.passed.push(self.journey.miles);
        let l = &ROUTE[self.at];
        info!("reached {} on {}, {:.0} miles", l.name, date::format(self.day), self.journey.miles);
        self.diary.push(format!("{}: Reached {}.", date::short(self.day), l.name));
        if let Kind::River { depth, .. } = l.kind {
            self.depth = river::depth_on(&mut self.rng, depth, self.day);
        }
        if l.name == "Fort Vancouver" {
            let lines = river::rapids(&mut self.rng, &mut self.party, &mut self.supplies);
            self.diary.extend(lines);
            self.bury(&[], self.day);
            if self.alive() == 0 {
                self.finish();
                return;
            }
        }
        if l.kind == Kind::End {
            self.finish();
        } else {
            self.go(Screen::Landmark);
        }
    }

    /// The journey is over: the graves first, then the reckoning.
    fn finish(&mut self) {
        info!("journey over on {}: {} of {} alive, score {}", date::format(self.day), self.alive(), PARTY, self.score().iter().map(|s| s.1).sum::<i64>());
        self.next_grave(0);
    }

    fn next_grave(&mut self, from: usize) {
        match (from..self.party.len()).find(|&i| !self.party[i].alive() && self.party[i].epitaph.is_none()) {
            Some(i) => {
                self.go(Screen::Epitaph(i));
                self.epitaph_text = default_epitaph(&self.party[i], &mut self.rng);
                self.epitaph_offered = true;
            }
            None => self.go(Screen::End),
        }
    }

    fn epitaph_key(&mut self, i: usize, key: Key) {
        match key {
            Key::Char(c) if !c.is_control() => {
                if std::mem::take(&mut self.epitaph_offered) {
                    self.epitaph_text.clear();
                }
                if self.epitaph_text.chars().count() < 60 {
                    self.epitaph_text.push(c);
                }
            }
            Key::Backspace => {
                self.epitaph_offered = false;
                self.epitaph_text.pop();
            }
            Key::Enter => {
                let words = self.epitaph_text.trim().to_string();
                self.party[i].epitaph = Some(words);
                self.next_grave(i + 1);
            }
            _ => {}
        }
    }

    pub fn arrived(&self) -> bool {
        ROUTE[self.at].kind == Kind::End && self.alive() > 0
    }

    /// The score, line by line: what it is for and how much.
    pub fn score(&self) -> Vec<(String, i64)> {
        if !self.arrived() {
            return Vec::new();
        }
        let mut lines = Vec::new();
        for p in self.party.iter().filter(|p| p.alive()) {
            lines.push((format!("{}, in {} health", p.name, trail::health_of(p)), 300 + 3 * p.health as i64));
        }
        let s = &self.supplies;
        lines.push((format!("{} oxen", s.get(Item::Oxen)), 25 * s.get(Item::Oxen) as i64));
        lines.push((format!("{} lb of provisions", s.get(Item::Food)), s.get(Item::Food) as i64 / 5));
        lines.push((format!("{} sets of clothing", s.get(Item::Clothing)), 5 * s.get(Item::Clothing) as i64));
        let spares = s.get(Item::Wheel) + s.get(Item::Axle) + s.get(Item::Tongue);
        lines.push((format!("{spares} spare {}", if spares == 1 { "part" } else { "parts" }), 10 * spares as i64));
        lines.push((format!("{} in your purse", party::money(self.money)), self.money as i64 / 100));
        // ahead of the autumn rains: October 1 is the mark
        let early = date::first_of(10) as i64 - self.day as i64;
        if early > 0 {
            lines.push((format!("{early} days before October"), 5 * early));
        }
        lines
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
        if self.leaving_key(key) {
            return;
        }
        let n = self.landmark_choices().len();
        match key {
            Key::Esc => self.leaving = true,
            Key::Up => self.cursor = self.cursor.saturating_sub(1),
            Key::Down => self.cursor = (self.cursor + 1).min(n - 1),
            Key::Enter => self.landmark_choice(self.cursor),
            Key::Char(c @ '1'..='9') if (c as usize - '1' as usize) < n => {
                self.cursor = c as usize - '1' as usize;
                self.landmark_choice(self.cursor);
            }
            _ => {}
        }
    }

    pub(crate) fn landmark_choice(&mut self, i: usize) {
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
                self.diary.push(format!("{}: {}.", date::short(self.day), label));
                if label.contains("Sublette") {
                    self.dry_drive();
                }
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
            Key::Char(c @ '1'..='9') if (c as usize - '1' as usize) < choices.len() => {
                self.cursor = c as usize - '1' as usize;
                self.river_choice(choices[self.cursor].0);
            }
            _ => {}
        }
    }

    pub(crate) fn river_choice(&mut self, way: Way) {
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
        self.diary.push(format!("{}: Crossed the {}.", date::short(self.day), river::river_name(name)));
        self.diary.extend(c.lines);
        self.bury(&[], self.day);
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
            self.diary.push(format!("{}: Nobody on the road had anything to trade.", date::short(self.day - 1)));
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
                self.diary.push(format!("{}: Traded {} for {}.", date::short(self.day - 1), amount_of(offer.give), amount_of(offer.get)));
                self.go(Screen::Trail);
            }
            Key::Char('n') | Key::Enter | Key::Esc => self.go(Screen::Trail),
            Key::Up | Key::Down => self.cursor = 1 - self.cursor.min(1),
            _ => {}
        }
    }

    /// The landmarks for the map, at their miles from Independence and
    /// whether they are behind the wagon, and the road's whole length. The
    /// road ahead follows the first choice at each fork.
    pub fn map_marks(&self) -> (Vec<(f64, bool)>, f64) {
        let mut marks: Vec<(f64, bool)> = self.passed.iter().map(|&m| (m, true)).collect();
        let mut miles = self.journey.miles + self.left.max(0.0);
        let mut i = self.to;
        if self.left > 0.0 || i != self.at {
            marks.push((miles, false));
        }
        loop {
            let (next, m) = match ROUTE[i].next {
                Next::On(n, m) => (n, m),
                Next::Fork(roads) => (roads[0].1, roads[0].2),
                Next::Done => break,
            };
            miles += m;
            marks.push((miles, false));
            i = next;
        }
        (marks, miles)
    }

    /// Floating down the Columbia from The Dalles, where no snow closes the road.
    fn on_raft(&self) -> bool {
        ROUTE[self.to].name == "Fort Vancouver"
    }

    /// How far along the road to the next landmark the wagon is, 0 to 1.
    pub fn along(&self) -> f64 {
        let total = match ROUTE[self.at].next {
            Next::On(n, m) if n == self.to => m,
            Next::Fork(roads) => roads.iter().find(|r| r.1 == self.to).map_or(self.left, |r| r.2),
            _ => self.left,
        };
        if total <= 0.0 { 1.0 } else { (1.0 - self.left / total).clamp(0.0, 1.0) }
    }

    pub fn next_landmark(&self) -> &'static str {
        ROUTE[self.to].name
    }

    pub fn done(&self) -> bool {
        self.screen == Screen::Quit
    }
}

/// Words for a grave, offered for the player to keep or change.
fn default_epitaph(p: &Person, rng: &mut Rng) -> String {
    let lines = [
        "Gone ahead to the promised land.",
        "Rest easy. We carried you as far as we could.",
        "Never saw Oregon, but never stopped walking toward it.",
        "Beloved. Remembered at every campfire.",
        "The prairie keeps you now.",
        "Faithful to the last mile.",
    ];
    match p.cause.as_deref() {
        Some("drowning") => "Taken by the river. Rest where it is quiet.".into(),
        Some("hunger") => (*rng.pick(&["We had not enough. Forgive us.", "Gave the last of the bread to the others.", "Hungry no more."])).into(),
        _ => (*rng.pick(&lines)).into(),
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

    #[test]
    fn esc_on_the_trail_asks_first() {
        let mut g = play(3, ">>>>>>>");
        g.supplies.set(Item::Oxen, 6);
        g.supplies.set(Item::Food, 800);
        g.cursor = Game::STORE_ROWS - 1;
        g.key(Key::Enter);
        assert_eq!(g.screen, Screen::Trail);
        g.key(Key::Esc);
        assert!(g.leaving);
        g.key(Key::Enter);
        assert_eq!(g.screen, Screen::Trail, "Enter is not a yes");
        g.key(Key::Esc);
        assert!(!g.leaving, "a second Esc stays");
        g.key(Key::Esc);
        g.key(Key::Char('y'));
        assert!(g.done());
    }

    #[test]
    fn stalking_takes_a_day_and_brings_back_what_it_shot() {
        let mut g = play(4, ">>>>>>>");
        g.supplies.set(Item::Oxen, 6);
        g.supplies.set(Item::Food, 800);
        g.supplies.set(Item::Shot, 40);
        g.cursor = Game::STORE_ROWS - 1;
        g.key(Key::Enter);
        let day = g.day;
        for k in script_keys("31^^^ ^^ ") {
            g.key(k);
        }
        assert_eq!(g.screen, Screen::Hunt);
        for _ in 0..crate::stalk::DAY {
            g.key(Key::Tick);
        }
        g.key(Key::Enter);
        assert_eq!(g.screen, Screen::Trail);
        assert_eq!(g.day, day + 1);
        assert!(g.supplies.get(Item::Shot) <= 40);
        assert!(g.diary.last().is_some_and(|d| d.contains("Stalking.")));
        // the quick way still works, and Esc goes back
        for k in script_keys("3!") {
            g.key(k);
        }
        assert_eq!(g.screen, Screen::Trail);
        for k in script_keys("32") {
            g.key(k);
        }
        assert_eq!(g.day, day + 2);
    }

    #[test]
    fn a_journey_ends() {
        for seed in 0..20 {
            for strategy in [crate::bot::Strategy::Sensible, crate::bot::Strategy::Careless] {
                let g = crate::bot::play(seed, strategy);
                assert_eq!(g.screen, Screen::End, "seed {seed}");
            }
        }
    }
}
