//! Drawing: green text on black with amber accents, the Asciiverse palette,
//! laid out like a teletype page in a column about 72 wide.

use crate::date;
use crate::game::{Game, PARTY, Screen};
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
        Line::from(vec![Span::raw("Matthiessen's General Store, Independence. "), dim(date::format(game.day))]),
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
            Span::raw(format!("{:>9}", party::money(g.price))),
        ]));
    }
    body.push(Line::default());
    body.push(Line::from(vec![pointer(game.cursor == ITEMS.len()), bold("Leave Independence")]));
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

fn trail(f: &mut Frame, page: Rect, game: &Game) {
    let names: Vec<&str> = game.party.iter().map(|p| p.name.as_str()).collect();
    let body = vec![Line::from(format!("{} leave Independence on {}.", names.join(", "), date::format(game.day)))];
    page_with(f, page, "The trail", body, "", keys(&[("Ctrl-C", "quit")]));
}
