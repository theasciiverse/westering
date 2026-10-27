//! The game's state and its answer to each key. Screens follow one another
//! like the pages of a diary: the title, outfitting, the trail, and the end.

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
}

#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    Title,
    Quit,
}

pub struct Game {
    pub rng: Rng,
    pub screen: Screen,
    /// the menu row under the cursor, on screens with a menu
    pub cursor: usize,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        info!("new game, seed {seed}");
        Game { rng: Rng::new(seed), screen: Screen::Title, cursor: 0 }
    }

    pub fn key(&mut self, key: Key) {
        debug!("key {key:?} on {:?}", self.screen);
        match self.screen {
            Screen::Title => match key {
                Key::Esc | Key::Char('q') => self.screen = Screen::Quit,
                _ => {}
            },
            Screen::Quit => {}
        }
    }

    pub fn done(&self) -> bool {
        self.screen == Screen::Quit
    }
}
