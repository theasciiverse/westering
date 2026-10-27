//! Westering: take a wagon party two thousand miles from Independence,
//! Missouri, to the Willamette Valley in 1848. Buy your outfit, set the pace
//! and the rations, cross the rivers and get your people there alive. An
//! Asciiverse journey game in the line of The Oregon Trail (1971), with our
//! own code, writing and history.
//!
//! Run: `cargo run --release [-- --seed N]` (`RUST_LOG=debug cargo run
//! --release 2>westering.log` for a session log). Without a terminal:
//! `cargo run -- --script KEYS [--seed N] [--size 80x30]` plays the keys and
//! prints the screen as text. In a script `>` is Enter, `<` Backspace, `!`
//! Esc, `^` and `_` the up and down arrows; every other character is itself.

mod date;
mod game;
mod party;
mod rng;
mod route;
mod trail;
mod ui;

use game::{Game, Key};
use log::info;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::io;
use std::time::Duration;

fn main() -> io::Result<()> {
    env_logger::Builder::from_default_env().format_timestamp_millis().init();
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str);
    let seed = arg("--seed").and_then(|s| s.parse().ok()).unwrap_or_else(seed);
    if let Some(keys) = arg("--script") {
        let size = arg("--size").and_then(|s| s.split_once('x')).and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)));
        print!("{}", script(seed, keys, size.unwrap_or((80, 30))));
        return Ok(());
    }
    run(seed)
}

fn seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64)
}

fn run(seed: u64) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut game = Game::new(seed);
    let result = (|| -> io::Result<()> {
        while !game.done() {
            terminal.draw(|f| ui::draw(f, &game))?;
            if !event::poll(Duration::from_millis(250))? {
                continue;
            }
            if let Event::Key(k) = event::read()?
                && k.kind != KeyEventKind::Release
            {
                if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                let key = match k.code {
                    KeyCode::Char(c) => Key::Char(c),
                    KeyCode::Enter => Key::Enter,
                    KeyCode::Esc => Key::Esc,
                    KeyCode::Backspace => Key::Backspace,
                    KeyCode::Up => Key::Up,
                    KeyCode::Down => Key::Down,
                    KeyCode::Left => Key::Left,
                    KeyCode::Right => Key::Right,
                    _ => continue,
                };
                game.key(key);
            }
        }
        Ok(())
    })();
    ratatui::restore();
    info!("quit");
    result
}

/// The keys of a script, with `>` `<` `!` `^` `_` `{` `}` standing for
/// Enter, Backspace, Esc and the up, down, left and right arrows.
pub fn script_keys(keys: &str) -> impl Iterator<Item = Key> + '_ {
    keys.chars().map(|c| match c {
        '>' => Key::Enter,
        '<' => Key::Backspace,
        '!' => Key::Esc,
        '^' => Key::Up,
        '_' => Key::Down,
        '{' => Key::Left,
        '}' => Key::Right,
        c => Key::Char(c),
    })
}

/// Play the keys without a terminal and return the screen as text.
fn script(seed: u64, keys: &str, (w, h): (u16, u16)) -> String {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    let mut game = Game::new(seed);
    for k in script_keys(keys) {
        game.key(k);
    }
    let mut terminal = Terminal::new(TestBackend::new(w, h)).expect("test backend");
    terminal.draw(|f| ui::draw(f, &game)).expect("draw");
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..h {
        let line: String = (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect();
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}
