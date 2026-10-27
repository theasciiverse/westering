//! The game's state and its answer to each key. Screens follow one another
//! like the pages of a diary: the title, the party, the store, the trail, and
//! the end.

use crate::date;
use crate::party::{self, ITEMS, Item, Person, Supplies};
use crate::rng::Rng;
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
    Trail,
    Quit,
}

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
            Screen::Trail | Screen::Quit => {}
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
            Key::Enter => self.leave_town(),
            Key::Esc => self.go(Screen::Month),
            _ => {}
        }
        if self.cursor != row || matches!(key, Key::Up | Key::Down) {
            self.note = match ITEMS.get(self.cursor) {
                Some(i) => i.good().advice.into(),
                None => "When you are ready, the trail is waiting.".into(),
            };
        }
    }

    fn buy(&mut self, item: Item) {
        let g = item.good();
        if self.money < g.price {
            self.note = "You cannot afford it.".into();
        } else if self.supplies.load() + g.per as f64 * g.weight > party::CAPACITY {
            self.note = "The wagon will not hold any more.".into();
        } else {
            self.money -= g.price;
            self.supplies.add(item, g.per);
            self.note = g.advice.into();
        }
    }

    fn sell_back(&mut self, item: Item) {
        let g = item.good();
        if self.supplies.get(item) >= g.per {
            self.supplies.take(item, g.per);
            self.money += g.price;
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
        self.go(Screen::Trail);
    }

    pub fn done(&self) -> bool {
        self.screen == Screen::Quit
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
}
