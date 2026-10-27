//! Drawing: green text on black with amber accents, the Asciiverse palette,
//! laid out like a teletype page in a column about 72 wide.

use crate::date;
use crate::game::{self, Game, PARTY, Screen, TRAIL_MENU};
use crate::party::Item;
use crate::route::ROUTE;
use crate::trail;
use crate::party::{self, ITEMS};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};

pub const GREEN: Color = Color::Rgb(80, 255, 120);
pub const DARK: Color = Color::Rgb(40, 150, 70);
pub const AMBER: Color = Color::Rgb(255, 176, 0);
pub const RED: Color = Color::Rgb(255, 90, 80);
pub const DIM: Color = Color::Rgb(120, 120, 130);
pub const BLACK: Color = Color::Rgb(0, 0, 0);

/// The page width the text is set in, like a teletype's roll.
pub const PAGE: u16 = 72;

pub fn draw(f: &mut Frame, game: &Game) {
    let area = f.area();
    f.render_widget(Block::new().style(Style::new().bg(BLACK).fg(GREEN)), area);
    let page = column(area);
    match &game.screen {
        Screen::Title => title(f, page),
        Screen::Names(i) => names(f, page, game, *i),
        Screen::Month => month(f, page, game),
        Screen::Store => store(f, page, game),
        Screen::Trail => trail(f, page, game),
        Screen::Landmark => landmark(f, page, game),
        Screen::River => river_page(f, page, game),
        Screen::Trade(offer) => trade(f, page, game, *offer),
        Screen::Epitaph(i) => epitaph(f, page, game, *i),
        Screen::End => end(f, page, game),
        Screen::Quit => {}
    }
}

/// The centred column the page is set in.
pub fn column(area: Rect) -> Rect {
    let w = PAGE.min(area.width);
    Rect { x: area.x + (area.width - w) / 2, y: area.y, width: w, height: area.height }
}

pub fn amber(s: impl Into<String>) -> Span<'static> {
    Span::styled(s.into(), Style::new().fg(AMBER))
}

pub fn dim(s: impl Into<String>) -> Span<'static> {
    Span::styled(s.into(), Style::new().fg(DIM))
}

pub fn bold(s: impl Into<String>) -> Span<'static> {
    Span::styled(s.into(), Style::new().fg(GREEN).add_modifier(Modifier::BOLD))
}

fn title(f: &mut Frame, page: Rect) {
    let art = [
        r"__        __        _            _             ",
        r"\ \      / /__  ___| |_ ___ _ __(_)_ __   __ _ ",
        r" \ \ /\ / / _ \/ __| __/ _ \ '__| | '_ \ / _` |",
        r"  \ V  V /  __/\__ \ ||  __/ |  | | | | | (_| |",
        r"   \_/\_/ \___||___/\__\___|_|  |_|_| |_|\__, |",
        r"                                         |___/ ",
    ];
    let mut lines: Vec<Line> = vec![Line::default(); 3];
    lines.extend(art.iter().map(|l| Line::from(amber(*l)).centered()));
    lines.push(Line::default());
    lines.push(Line::from("Independence, Missouri. Spring, 1848.").centered());
    lines.push(Line::default());
    lines.push(Line::from("Two thousand miles of prairie, desert and mountain lie between you").centered());
    lines.push(Line::from("and the Willamette Valley. Every wagon that sets out is a wager.").centered());
    lines.push(Line::default());
    lines.push(Line::default());
    lines.push(Line::from(vec![amber("Enter"), Span::raw(" set out    "), amber("Esc"), Span::raw(" quit")]).centered());
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), page);
}

/// A page: a heading, the body lines, then the note and the key help at
/// the foot.
fn page_with(f: &mut Frame, page: Rect, heading: &str, mut body: Vec<Line<'static>>, note: &str, help: Line<'static>) {
    let mut lines = vec![Line::default(), Line::from(amber(heading.to_uppercase())), Line::default()];
    lines.append(&mut body);
    let foot = 4;
    let room = page.height.saturating_sub(foot) as usize;
    lines.truncate(room);
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), Rect { height: page.height.saturating_sub(foot), ..page });
    let foot_area = Rect { y: page.y + page.height.saturating_sub(foot), height: foot.min(page.height), ..page };
    let foot_lines = vec![Line::from(note.to_string()).style(Style::new().fg(DARK)), Line::default(), help];
    f.render_widget(Paragraph::new(foot_lines).wrap(Wrap { trim: false }), foot_area);
}

fn keys(pairs: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (k, what)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        spans.push(amber(*k));
        spans.push(dim(format!(" {what}")));
    }
    Line::from(spans)
}

fn pointer(on: bool) -> Span<'static> {
    if on { amber("> ") } else { Span::raw("  ") }
}

fn names(f: &mut Frame, page: Rect, game: &Game, field: usize) {
    let mut body = vec![Line::from("Five of you will make the journey. Who are they?"), Line::default()];
    for i in 0..PARTY {
        let label = if i == 0 { "Leader of the party" } else { "Companion" };
        let mut name = vec![pointer(i == field), dim(format!("{label:<22}")), bold(game.party[i].name.clone())];
        if i == field {
            name.push(Span::styled("_", Style::new().fg(AMBER).add_modifier(Modifier::SLOW_BLINK)));
        }
        body.push(Line::from(name));
    }
    page_with(f, page, "Your party", body, &game.note, keys(&[("type", "a name"), ("Enter", "next"), ("Up Down", "move"), ("Esc", "back")]));
}

fn month(f: &mut Frame, page: Rect, game: &Game) {
    let mut body = vec![Line::from("When will you leave Independence?"), Line::default()];
    for (i, (m, _)) in party::MONTHS.iter().enumerate() {
        body.push(Line::from(vec![pointer(i == game.cursor), bold(*m)]));
    }
    body.push(Line::default());
    page_with(f, page, "Setting out", body, party::MONTHS[game.cursor].1, keys(&[("Up Down", "choose"), ("Enter", "decide"), ("Esc", "back")]));
}

fn store(f: &mut Frame, page: Rect, game: &Game) {
    let mut body = vec![
        Line::from(vec![
            Span::raw(if game.at == 0 { "Matthiessen's General Store, Independence. ".to_string() } else { format!("The trade room at {}. ", ROUTE[game.at].name) }),
            dim(date::format(game.day)),
        ]),
        Line::default(),
        Line::from(dim(format!("  {:<18}{:>9}  {:<18}{:>9}", "", "you have", "sold by the", "price"))),
    ];
    for (i, &item) in ITEMS.iter().enumerate() {
        let g = item.good();
        body.push(Line::from(vec![
            pointer(i == game.cursor),
            bold(format!("{:<18}", g.name)),
            Span::raw(format!("{:>9}", game.supplies.get(item))),
            dim(format!("  {:<18}", g.unit)),
            Span::raw(format!("{:>9}", party::money(game.price(item)))),
        ]));
    }
    body.push(Line::default());
    body.push(Line::from(vec![pointer(game.cursor == ITEMS.len()), bold(if game.at == 0 { "Leave Independence" } else { "Done trading" })]));
    body.push(Line::default());
    let load = game.supplies.load();
    body.push(Line::from(vec![
        dim("Money left "),
        amber(party::money(game.money)),
        dim("    Load "),
        Span::styled(format!("{:.0} of {:.0} lb", load, party::CAPACITY), Style::new().fg(if load > party::CAPACITY * 0.9 { AMBER } else { GREEN })),
    ]));
    page_with(f, page, "Outfitting", body, &game.note, keys(&[("Right +", "buy"), ("Left -", "put back"), ("Up Down", "choose"), ("Esc", "back")]));
}

/// The wagon's state, the head of every trail page.
fn status(game: &Game) -> Vec<Line<'static>> {
    let j = &game.journey;
    let row = |a: String, b: String| Line::from(vec![Span::raw(format!("{a:<36}")), Span::raw(b)]);
    vec![
        Line::from(vec![bold(format!("{:<36}", date::format(game.day))), Span::raw(format!("Weather: {}", j.weather.describe()))]),
        row(format!("Health: {}", trail::health_word(&game.party)), format!("Pace: {}   Rations: {}", j.pace.name(), j.rations.name())),
        row(format!("Provisions: {} lb", game.supplies.get(Item::Food)), if game.on_foot { "On foot, the wagon left behind".to_string() } else { format!("Oxen: {}, {}", game.supplies.get(Item::Oxen), trail::oxen_word(j.oxen_health)) }),
        row(format!("Next: {}, {:.0} mi", game.next_landmark(), game.left.max(0.0)), format!("Travelled: {:.0} miles", j.miles)),
        Line::from(people(game)),
    ]
}

/// Each person and how they are: "Ada good, Tom poor (measles)", the dead dimmed.
fn people(game: &Game) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, p) in game.party.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        if !p.alive() {
            spans.push(dim(format!("{} (dead)", p.name)));
            continue;
        }
        let color = match p.health {
            h if h >= 60 => GREEN,
            h if h >= 35 => AMBER,
            _ => RED,
        };
        let mut text = format!("{} {}", p.name, trail::health_of(p));
        if let Some(ill) = &p.illness {
            text.push_str(&format!(" ({})", ill.name));
        }
        spans.push(Span::styled(text, Style::new().fg(color)));
    }
    spans
}

/// The newest diary lines that fit in `rows`, wrapped to the page.
fn diary(game: &Game, rows: usize, width: usize) -> Vec<Line<'static>> {
    let mut lines: Vec<Line> = Vec::new();
    for entry in game.diary.iter().rev() {
        let wrapped = wrap(entry, width);
        if lines.len() + wrapped.len() > rows {
            break;
        }
        for l in wrapped.into_iter().rev() {
            lines.insert(0, Line::from(l).style(Style::new().fg(DARK)));
        }
    }
    lines
}

/// Plain word wrap.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn trail(f: &mut Frame, page: Rect, game: &Game) {
    let mut body = status(game);
    body.push(Line::default());
    let menu: Vec<Line> = TRAIL_MENU
        .iter()
        .enumerate()
        .map(|(i, m)| Line::from(vec![pointer(i == game.cursor), amber(format!("{} ", i + 1)), bold(*m)]))
        .collect();
    let rows = (page.height as usize).saturating_sub(3 + body.len() + 1 + menu.len() + 4);
    body.extend(diary(game, rows, page.width as usize));
    body.push(Line::default());
    body.extend(menu);
    page_with(f, page, "On the trail", body, &game.note, keys(&[("Up Down", "choose"), ("Enter", "do it"), ("1-6", "at once")]));
}

fn landmark(f: &mut Frame, page: Rect, game: &Game) {
    let l = &ROUTE[game.at];
    let mut body = status(game);
    body.push(Line::default());
    body.push(Line::from(bold(l.name)));
    body.push(Line::default());
    for w in wrap(l.note, page.width as usize) {
        body.push(Line::from(w));
    }
    body.push(Line::default());
    for (i, c) in game.landmark_choices().iter().enumerate() {
        body.push(Line::from(vec![pointer(i == game.cursor), bold(c.clone())]));
    }
    page_with(f, page, l.name, body, &game.note, keys(&[("Up Down", "choose"), ("Enter", "do it")]));
}

fn river_page(f: &mut Frame, page: Rect, game: &Game) {
    let l = &ROUTE[game.at];
    let crate::route::Kind::River { width, .. } = l.kind else { return };
    let mut body = status(game);
    body.push(Line::default());
    for w in wrap(&crate::river::describe(game.depth, width), page.width as usize) {
        body.push(Line::from(w));
    }
    body.push(Line::default());
    for (i, (_, c)) in game.river_choices().iter().enumerate() {
        body.push(Line::from(vec![pointer(i == game.cursor), bold(c.clone())]));
    }
    page_with(f, page, &format!("Crossing the {}", crate::river::river_name(l.name)), body, &game.note, keys(&[("Up Down", "choose"), ("Enter", "do it"), ("Esc", "back")]));
}

fn trade(f: &mut Frame, page: Rect, game: &Game, offer: game::Offer) {
    let mut body = status(game);
    body.push(Line::default());
    let text = format!(
        "A family from another train will give you {} for {}. You have {}.",
        game::amount_of(offer.get),
        game::amount_of(offer.give),
        game::amount_of((offer.give.0, game.supplies.get(offer.give.0)))
    );
    for w in wrap(&text, page.width as usize) {
        body.push(Line::from(w));
    }
    body.push(Line::default());
    body.push(Line::from(vec![pointer(game.cursor == 0), bold("Accept")]));
    body.push(Line::from(vec![pointer(game.cursor == 1), bold("Decline")]));
    page_with(f, page, "A trade", body, &game.note, keys(&[("y", "accept"), ("n", "decline")]));
}

fn epitaph(f: &mut Frame, page: Rect, game: &Game, i: usize) {
    let p = &game.party[i];
    let w = page.width as usize;
    let mut body = vec![Line::from(bold(format!("Here lies {}", p.name))), Line::default()];
    let died = format!("Died of {}, {}.", p.cause.as_deref().unwrap_or("the trail"), p.grave.as_deref().unwrap_or("on the trail"));
    body.extend(wrap(&died, w).into_iter().map(Line::from));
    body.push(Line::default());
    body.push(Line::from("You cut a board for the grave. What shall it say?"));
    body.push(Line::default());
    body.push(Line::from(vec![
        amber("> "),
        Span::raw(game.epitaph_text.clone()),
        Span::styled("_", Style::new().fg(AMBER).add_modifier(Modifier::SLOW_BLINK)),
    ]));
    page_with(f, page, "A grave by the trail", body, "", keys(&[("type", "the words"), ("Backspace", "erase"), ("Enter", "done")]));
}

fn end(f: &mut Frame, page: Rect, game: &Game) {
    let w = page.width as usize;
    let mut body = Vec::new();
    let days = game.day - game.started;
    let heading = if game.arrived() {
        let text = format!(
            "You reached Oregon City on {}, {} days and {:.0} miles after leaving Independence, with {} of the {} who set out.",
            date::format(game.day),
            days,
            game.journey.miles,
            game.alive(),
            PARTY
        );
        body.extend(wrap(&text, w).into_iter().map(Line::from));
        body.push(Line::default());
        let score = game.score();
        for (what, points) in &score {
            body.push(Line::from(vec![Span::raw(format!("  {what:<50}")), amber(format!("{points:>8}"))]));
        }
        body.push(Line::from(vec![bold(format!("  {:<50}", "Score")), amber(format!("{:>8}", score.iter().map(|s| s.1).sum::<i64>()))]));
        "The Willamette Valley"
    } else {
        let text = format!("The trail took every one of you. The last of the party died on {}, {:.0} miles from Independence.", date::format(game.day.saturating_sub(1)), game.journey.miles);
        body.extend(wrap(&text, w).into_iter().map(Line::from));
        "The end of the trail"
    };
    let graves: Vec<_> = game.party.iter().filter(|p| !p.alive()).collect();
    if !graves.is_empty() {
        body.push(Line::default());
        body.push(Line::from(amber("Graves along the trail")));
        for p in graves {
            let text = format!("{}, died of {}, {}.", p.name, p.cause.as_deref().unwrap_or("the trail"), p.grave.as_deref().unwrap_or("on the trail"));
            body.extend(wrap(&text, w).into_iter().map(|l| Line::from(format!("  {l}"))));
            if let Some(e) = p.epitaph.as_deref().filter(|e| !e.is_empty()) {
                body.push(Line::from(dim(format!("    \"{e}\""))));
            }
        }
    }
    page_with(f, page, heading, body, "", keys(&[("Enter", "play again"), ("Esc", "quit")]));
}
