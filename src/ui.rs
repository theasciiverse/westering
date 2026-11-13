//! Drawing: green text on black with amber accents, the Asciiverse palette,
//! laid out like a teletype page in a column about 72 wide.

use crate::date;
use crate::game::{self, Game, PARTY, Screen, TRAIL_MENU};
use crate::party::Item;
use crate::route::ROUTE;
use crate::pix;
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
        Screen::Title => title(f, page, game),
        Screen::Names(i) => names(f, page, game, *i),
        Screen::Month => month(f, page, game),
        Screen::Store => store(f, page, game),
        Screen::Trail => trail(f, page, game),
        Screen::Landmark => landmark(f, page, game),
        Screen::River => river_page(f, page, game),
        Screen::Trade(offer) => trade(f, page, game, *offer),
        Screen::HuntChoice => hunt_choice(f, page, game),
        Screen::Hunt => hunt_page(f, page, game),
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

fn title(f: &mut Frame, page: Rect, game: &Game) {
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
    let register = register_line(game);
    let pic = (page.height as usize).saturating_sub(lines.len() + 4 + if register.is_some() { 2 } else { 0 }).min(8);
    if pic >= 4 {
        lines.extend(pix::title(page.width as usize, pic).lines());
    }
    lines.push(Line::default());
    if game.can_resume {
        lines.push(Line::from(vec![amber("c"), Span::raw(" take up your journey    "), amber("Enter"), Span::raw(" set out anew    "), amber("Esc"), Span::raw(" quit")]).centered());
    } else {
        lines.push(Line::from(vec![amber("Enter"), Span::raw(" set out    "), amber("Esc"), Span::raw(" quit")]).centered());
    }
    if let Some(r) = register {
        lines.push(Line::default());
        lines.push(Line::from(dim(r)).centered());
    }
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), page);
}

/// The register of journeys in a line: how many, and the best.
fn register_line(game: &Game) -> Option<String> {
    let n = game.records.len();
    let arrived = game.records.iter().filter(|r| r.arrived).count();
    let best = game.records.iter().filter(|r| r.arrived).max_by_key(|r| r.score);
    let made = format!("{n} {} made, {arrived} to Oregon", if n == 1 { "journey" } else { "journeys" });
    match best {
        _ if n == 0 => None,
        Some(b) => Some(format!("{made}. The best: {}'s party, {} of {}, score {}.", b.leader, b.alive, PARTY, b.score)),
        None => Some(format!("{made}.")),
    }
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
            Span::raw(if game.at == 0 { "Ebbert and Sons, Outfitters, Independence. ".to_string() } else { format!("The trade room at {}. ", ROUTE[game.at].name) }),
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
fn status(game: &Game, width: usize) -> Vec<Line<'static>> {
    let j = &game.journey;
    let row = |a: String, b: String| Line::from(vec![Span::raw(format!("{a:<36}")), Span::raw(b)]);
    let mut lines = vec![
        Line::from(vec![bold(format!("{:<36}", date::format(game.day))), Span::raw(format!("Weather: {}", j.weather.describe()))]),
        row(format!("Health: {}", trail::health_word(&game.party)), format!("Pace: {}   Rations: {}", j.pace.name(), j.rations.name())),
        row(format!("Provisions: {} lb", game.supplies.get(Item::Food)), if game.on_foot { "On foot, the wagon left behind".to_string() } else { format!("Oxen: {}, {}", game.supplies.get(Item::Oxen), trail::oxen_word(j.oxen_health)) }),
        row(if game.left <= 0.0 && game.at == game.to { format!("At {}", ROUTE[game.at].name) } else { format!("Next: {}, {:.0} mi", game.next_landmark(), game.left.max(0.0)) }, format!("Travelled: {:.0} miles", j.miles)),
    ];
    // the party, on as many lines as it takes
    let mut line: Vec<Span> = Vec::new();
    let mut used = 0;
    for span in people(game) {
        let n = span.content.chars().count();
        if used > 0 && used + 2 + n > width {
            lines.push(Line::from(std::mem::take(&mut line)));
            used = 0;
        }
        if used > 0 {
            line.push(Span::raw("  "));
            used += 2;
        }
        used += n;
        line.push(span);
    }
    lines.push(Line::from(line));
    lines
}

/// Each person and how they are: "Ada good, Tom poor (measles)", the dead dimmed.
fn people(game: &Game) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for p in &game.party {
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

/// The road as two lines: the wagon above, the landmarks on the road below.
fn map(game: &Game, width: usize) -> Vec<Line<'static>> {
    let (marks, total) = game.map_marks();
    let (wagon, road) = crate::art::map_line(width, game.journey.miles, total, &marks);
    vec![Line::from(amber(wagon)), Line::from(road).style(Style::new().fg(DARK))]
}

fn trail(f: &mut Frame, page: Rect, game: &Game) {
    let mut body = status(game, page.width as usize);
    let head = body.len();
    body.extend(map(game, page.width as usize));
    body.push(Line::default());
    let menu: Vec<Line> = TRAIL_MENU
        .iter()
        .enumerate()
        .map(|(i, m)| Line::from(vec![pointer(i == game.cursor), amber(format!("{} ", i + 1)), bold(*m)]))
        .collect();
    let rows = (page.height as usize).saturating_sub(3 + body.len() + 1 + menu.len() + 4);
    // the country the wagon is crossing, where it leaves the diary four rows
    let pic = rows.saturating_sub(4).min(5);
    if pic >= 3 {
        let view = pix::View { region: game.journey.region, sky: game.journey.weather.sky, month: date::month(game.day) };
        let lines = pix::road(view, page.width as usize, pic, game.along(), game.to as u64).lines();
        body.splice(head..head, lines);
    }
    let rows = if pic >= 3 { rows - pic } else { rows };
    body.extend(diary(game, rows, page.width as usize));
    body.push(Line::default());
    body.extend(menu);
    let help = if game.leaving { leave_question() } else { keys(&[("Up Down", "choose"), ("Enter", "do it"), ("1-6", "at once"), ("Esc", "leave")]) };
    page_with(f, page, "On the trail", body, &game.note, help);
}

/// Esc on the trail or at a landmark: leave for now?
fn leave_question() -> Line<'static> {
    let mut ask = keys(&[("y", "leave"), ("n", "stay")]);
    ask.spans.insert(0, Span::styled("Stop here for now? The journey waits for you.   ", Style::new().fg(AMBER)));
    ask
}

fn landmark(f: &mut Frame, page: Rect, game: &Game) {
    let l = &ROUTE[game.at];
    let mut body = status(game, page.width as usize);
    body.push(Line::default());
    let note: Vec<Line> = wrap(l.note, page.width as usize).into_iter().map(Line::from).collect();
    let choices = game.landmark_choices();
    // the picture, then what happened on the way here, each only where it
    // leaves room for the note and the choices
    let mut room = (page.height as usize).saturating_sub(3 + 4 + body.len() + note.len() + 1 + choices.len());
    let way: Vec<&String> = game.diary.get(game.stretch..).unwrap_or_default().iter().filter(|e| !e.contains(": Reached ")).collect();
    // whole entries, newest first, as many as could ever fit
    let mut on_the_way: Vec<Line> = Vec::new();
    for entry in way.iter().rev() {
        let lines: Vec<Line> = wrap(entry, page.width as usize).into_iter().map(|w| Line::from(w).style(Style::new().fg(DARK))).collect();
        if on_the_way.len() + lines.len() > room {
            break;
        }
        on_the_way.splice(0..0, lines);
    }
    let way_rows = if on_the_way.is_empty() { 0 } else { on_the_way.len().min(2) + 1 };
    let rows = room.saturating_sub(way_rows + 1).min(8);
    if rows >= 3 {
        body.extend(landmark_picture(game, page.width as usize, rows));
        body.push(Line::default());
        room -= rows + 1;
    }
    if !on_the_way.is_empty() {
        // the newest entries, when not all fit
        let mut skip = 0;
        while on_the_way.len() - skip > room.saturating_sub(1) {
            // drop the oldest entry: up to the next line that starts one
            skip += 1;
            while skip < on_the_way.len() && !line_starts_entry(&on_the_way[skip]) {
                skip += 1;
            }
        }
        let at = body.len() - if rows >= 3 { rows + 1 } else { 0 };
        body.splice(at..at, on_the_way.into_iter().skip(skip).chain(std::iter::once(Line::default())));
    }
    body.extend(note);
    body.push(Line::default());
    for (i, c) in choices.iter().enumerate() {
        body.push(Line::from(vec![pointer(i == game.cursor), amber(format!("{} ", i + 1)), bold(c.clone())]));
    }
    let help = if game.leaving { leave_question() } else { keys(&[("Up Down", "choose"), ("Enter", "do it"), ("Esc", "leave")]) };
    page_with(f, page, l.name, body, &game.note, help);
}

/// Diary entries begin with a date ("May 3: ..."), their wrapped lines do not.
fn line_starts_entry(line: &Line) -> bool {
    let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
    text.split_once(": ").is_some_and(|(head, _)| head.split(' ').count() <= 4 && head.chars().any(|c| c.is_ascii_digit()) && date_like(head))
}

fn date_like(head: &str) -> bool {
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"].iter().any(|m| head.starts_with(m))
}

/// The landmark the wagon is at, in today's weather.
fn landmark_picture(game: &Game, width: usize, rows: usize) -> Vec<Line<'static>> {
    let l = &ROUTE[game.at];
    let view = pix::View { region: l.region, sky: game.journey.weather.sky, month: date::month(game.day) };
    pix::landmark(l.name, &l.kind, view, width, rows).lines()
}

fn river_page(f: &mut Frame, page: Rect, game: &Game) {
    let l = &ROUTE[game.at];
    let crate::route::Kind::River { width, .. } = l.kind else { return };
    let mut body = status(game, page.width as usize);
    let head = body.len();
    body.push(Line::default());
    for w in wrap(&crate::river::describe(game.depth, width), page.width as usize) {
        body.push(Line::from(w));
    }
    body.push(Line::default());
    for (i, (_, c)) in game.river_choices().iter().enumerate() {
        body.push(Line::from(vec![pointer(i == game.cursor), amber(format!("{} ", i + 1)), bold(c.clone())]));
    }
    let pic = (page.height as usize).saturating_sub(3 + 4 + body.len() + 1).min(8);
    if pic >= 3 {
        let lines = landmark_picture(game, page.width as usize, pic);
        body.splice(head + 1..head + 1, lines.into_iter().chain(std::iter::once(Line::default())));
    }
    page_with(f, page, &format!("Crossing the {}", crate::river::river_name(l.name)), body, &game.note, keys(&[("Up Down", "choose"), ("Enter", "do it"), ("Esc", "back")]));
}

fn trade(f: &mut Frame, page: Rect, game: &Game, offer: game::Offer) {
    let mut body = status(game, page.width as usize);
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
    if game.epitaph_offered {
        body.push(Line::from(dim("Keep these words with Enter, or type your own.")));
    }
    body.push(Line::default());
    body.push(Line::from(vec![
        amber("> "),
        if game.epitaph_offered { dim(game.epitaph_text.clone()) } else { Span::raw(game.epitaph_text.clone()) },
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
            body.extend(wrap(&text, w.saturating_sub(4)).into_iter().enumerate().map(|(i, l)| Line::from(format!("{}{l}", if i == 0 { "  " } else { "    " }))));
            if let Some(e) = p.epitaph.as_deref().filter(|e| !e.is_empty()) {
                body.push(Line::from(dim(format!("    \"{e}\""))));
            }
        }
    }
    let best = game.records.iter().filter(|r| r.arrived).max_by_key(|r| r.score);
    let note = match best {
        Some(b) if game.arrived() && game.score().iter().map(|s| s.1).sum::<i64>() > b.score => "The best journey yet in the register.".to_string(),
        Some(b) => format!("The best before this one: {}'s party, {} of {}, score {}.", b.leader, b.alive, PARTY, b.score),
        None => String::new(),
    };
    page_with(f, page, heading, body, &note, keys(&[("Enter", "play again"), ("Esc", "quit")]));
}

fn hunt_choice(f: &mut Frame, page: Rect, game: &Game) {
    let mut body = status(game, page.width as usize);
    body.push(Line::default());
    body.push(Line::from(format!("You have {} rounds of powder and shot. How will you hunt?", game.supplies.get(Item::Shot))));
    body.push(Line::default());
    let choices = ["Stalk game yourself", "Send the hunters out for the day"];
    for (i, c) in choices.iter().enumerate() {
        body.push(Line::from(vec![pointer(i == game.cursor), amber(format!("{} ", i + 1)), bold(*c)]));
    }
    body.push(Line::default());
    body.push(Line::from(dim("Stalking: creep up from downwind and through cover, and fire when close.")));
    page_with(f, page, "Hunting", body, &game.note, keys(&[("1 2", "choose"), ("Enter", "do it"), ("Esc", "back")]));
}

/// The colours of the hunting ground by country.
fn field_colours(region: crate::trail::Region) -> (Color, Color) {
    use crate::trail::Region::*;
    match region {
        Prairie => (Color::Rgb(34, 66, 30), Color::Rgb(70, 120, 56)),
        HighPlains => (Color::Rgb(70, 62, 34), Color::Rgb(140, 124, 70)),
        Mountains => (Color::Rgb(52, 58, 44), Color::Rgb(116, 124, 92)),
        SnakeDesert => (Color::Rgb(78, 68, 46), Color::Rgb(150, 136, 98)),
        BlueMountains => (Color::Rgb(28, 54, 36), Color::Rgb(64, 110, 70)),
        Columbia => (Color::Rgb(26, 58, 34), Color::Rgb(62, 116, 66)),
    }
}

fn hunt_page(f: &mut Frame, page: Rect, game: &Game) {
    use crate::stalk::{self, Ground, State};
    let Some(s) = game.stalk.as_ref() else { return };
    let (bg, fg) = field_colours(game.journey.region);
    let mut body = Vec::new();
    // the sun across the sky, the wind, the rounds and the meat
    let bar = 20usize;
    let sun = ((s.daylight() * (bar - 1) as f64).round() as usize).min(bar - 1);
    let sky: String = (0..bar).map(|i| if i == sun { 'o' } else { '-' }).collect();
    let (down, carried) = s.meat();
    body.push(Line::from(vec![
        dim("Sun "),
        amber(sky),
        dim("   Wind "),
        bold(format!("{} {}", stalk::wind_arrow(s.wind), stalk::wind_word(s.wind))),
    ]));
    body.push(Line::from(vec![
        dim("Rounds "),
        bold(format!("{}", s.rounds - s.fired)),
        dim("   Meat down "),
        bold(format!("{down} lb")),
        dim(format!("   You can carry {} lb", s.carry)),
    ]));
    let _ = carried;
    body.push(Line::default());
    let pad = (page.width as i32 - stalk::W).max(0) / 2;
    for y in 0..stalk::H {
        let mut spans = vec![Span::raw(" ".repeat(pad as usize))];
        for x in 0..stalk::W {
            let base = Style::new().bg(bg);
            let cell = if s.hunter == (x, y) {
                Span::styled("@", base.fg(AMBER).add_modifier(Modifier::BOLD))
            } else if let Some(a) = s.animals.iter().find(|a| a.x == x && a.y == y && a.state != State::Gone) {
                let colour = match a.state {
                    State::Grazing => Color::Rgb(240, 236, 220),
                    State::Alert => AMBER,
                    State::Fleeing(..) => RED,
                    _ => Color::Rgb(150, 60, 50),
                };
                let glyph = if a.state == State::Down { 'x' } else { a.kind.glyph };
                Span::styled(glyph.to_string(), base.fg(colour).add_modifier(Modifier::BOLD))
            } else {
                match s.at(x, y) {
                    Ground::Grass => Span::styled(if crate::pix::hash(x as usize, y as usize, 5) < 0.2 { "," } else { " " }, base.fg(fg)),
                    Ground::Brush => Span::styled("\"", base.fg(Color::Rgb(120, 170, 80))),
                    Ground::Rock => Span::styled("o", base.fg(Color::Rgb(170, 165, 160))),
                    Ground::Tree => Span::styled("\u{2663}", base.fg(Color::Rgb(40, 140, 60))),
                    Ground::Water => Span::styled("~", Style::new().bg(Color::Rgb(40, 80, 140)).fg(Color::Rgb(140, 180, 220))),
                }
            };
            spans.push(cell);
        }
        body.push(Line::from(spans));
    }
    let help = if s.over {
        keys(&[("Enter", "back to the wagon")])
    } else {
        keys(&[("arrows", "creep"), ("q e z c", "diagonals"), ("Space", "fire"), ("Enter", "go back")])
    };
    page_with(f, page, "Stalking", body, &s.message, help);
}
