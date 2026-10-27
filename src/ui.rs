//! Drawing: green text on black with amber accents, the Asciiverse palette,
//! laid out like a teletype page in a column about 72 wide.

use crate::game::{Game, Screen};
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
