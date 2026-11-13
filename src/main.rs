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
//! Esc, `^` `_` `{` `}` the up, down, left and right arrows, `~` a tick of
//! the hunting scene's clock; every other character is itself. `--bot N
//! [--seed S]` plays N journeys with a sensible and a careless rule player and prints how they fared (`--month
//! M`, 3 to 7, sends both off in that month);
//! `--bot-show SEED [--careless]` prints the last screen of one and its diary;
//! `--pictures` prints every landmark's picture in colour.

mod art;
mod bot;
mod date;
mod events;
mod game;
mod hunt;
mod party;
mod pix;
mod river;
mod rng;
mod route;
mod save;
mod stalk;
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
    if let Some(n) = arg("--bot") {
        print!("{}", bot::report(n.parse().unwrap_or(100), seed.min(1 << 40), arg("--month").and_then(|m| m.parse().ok())));
        return Ok(());
    }
    if let Some(s) = arg("--bot-show") {
        let g = bot::play(s.parse().unwrap_or(0), if args.iter().any(|a| a == "--careless") { bot::Strategy::Careless } else { bot::Strategy::Sensible });
        print!("{}", render(&g, (80, 50)));
        println!("--- the diary ---");
        for line in &g.diary {
            println!("{line}");
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--pictures") {
        print!("{}", pictures());
        return Ok(());
    }
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
    game.can_resume = save::exists();
    game.records = save::records();
    let result = (|| -> io::Result<()> {
        let tick = Duration::from_millis(1000 / stalk::TICKS_PER_SECOND as u64);
        let mut last_tick = std::time::Instant::now();
        while !game.done() {
            terminal.draw(|f| ui::draw(f, &game))?;
            // the hunting scene runs on a clock; every other page waits for keys
            let hunting = game.screen == game::Screen::Hunt;
            if hunting && last_tick.elapsed() >= tick {
                last_tick = std::time::Instant::now();
                game.key(Key::Tick);
                continue;
            }
            let wait = if hunting { tick.saturating_sub(last_tick.elapsed()) } else { Duration::from_millis(250) };
            if !event::poll(wait)? {
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
                if game.screen == game::Screen::Title && game.can_resume && key == Key::Char('c') {
                    if let Some(mut g) = save::load() {
                        g.records = std::mem::take(&mut game.records);
                        game = g;
                    }
                    continue;
                }
                let was_over = matches!(game.screen, game::Screen::Epitaph(_) | game::Screen::End);
                game.key(key);
                match game.screen {
                    game::Screen::Epitaph(_) | game::Screen::End if !was_over => {
                        save::forget();
                        save::record(&save::Record::of(&game));
                    }
                    _ => save::store(&game),
                }
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
        '~' => Key::Tick,
        c => Key::Char(c),
    })
}

/// Play the keys without a terminal and return the screen as text.
fn script(seed: u64, keys: &str, (w, h): (u16, u16)) -> String {
    let mut game = Game::new(seed);
    for k in script_keys(keys) {
        game.key(k);
    }
    render(&game, (w, h))
}

/// The screen as text.
fn render(game: &Game, (w, h): (u16, u16)) -> String {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    let mut terminal = Terminal::new(TestBackend::new(w, h)).expect("test backend");
    terminal.draw(|f| ui::draw(f, game)).expect("draw");
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..h {
        let line: String = (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect();
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// Every landmark's picture in colour, for working on the art.
fn pictures() -> String {
    use ratatui::style::Color;
    let mut out = String::new();
    let rgb = |c: Option<Color>| match c {
        Some(Color::Rgb(r, g, b)) => format!("{r};{g};{b}"),
        _ => "0;0;0".into(),
    };
    for (i, l) in route::ROUTE.iter().enumerate() {
        let month = 5 + (i as u32 * 5 / route::ROUTE.len() as u32);
        let view = pix::View { region: l.region, sky: trail::Sky::Fair, month };
        out.push_str(&format!("{}\n", l.name));
        for line in pix::landmark(l.name, &l.kind, view, 72, 8).lines() {
            for span in &line.spans {
                out.push_str(&format!("\x1b[38;2;{}m\x1b[48;2;{}m{}", rgb(span.style.fg), rgb(span.style.bg), span.content));
            }
            out.push_str("\x1b[0m\n");
        }
    }
    out
}
