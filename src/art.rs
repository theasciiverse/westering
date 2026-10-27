//! Small pictures in plain characters, teletype style: a scene for some
//! landmarks, and the map of the road so far.

use crate::route::{Kind, Landmark};

/// A few lines of scenery for a landmark, if it has one.
pub fn scene(l: &Landmark) -> Option<&'static [&'static str]> {
    Some(match l.name {
        "Chimney Rock" => &[
            r"                    |",
            r"                    |",
            r"                   / \",
            r"                  /   \",
            r"            _____/     \_____",
            r"  _________/                 \___________",
        ],
        "Independence Rock" => &[
            r"                 __________",
            r"            ____/  names   \______",
            r"         __/   carved here      \__",
            r"  ~~~~~~/__________________________\~~~~~ Sweetwater",
        ],
        "South Pass" => &[
            r"      /\        /\    the divide     /\",
            r"     /  \  /\  /  \_________________/  \  /\",
            r"    /    \/  \/                         \/  \",
            r"   <-- to the Atlantic        to the Pacific -->",
        ],
        "Scotts Bluff" => &[
            r"            ___      _____",
            r"       ____|   |____|     |___",
            r"      |                       |____",
            r"  ____|                            |______",
        ],
        "Laurel Hill" => &[
            r"   \",
            r"    \___  wagon",
            r"        \___  chained to a tree",
            r"            \___",
            r"                \_______________",
        ],
        "Oregon City" => &[
            r"     _/\_      _/\_          ||  ||",
            r"    /    \    /    \    _____||__||_____",
            r"   ~~~~~~~~ falls of the Willamette ~~~~~~",
            r"  ,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,,",
        ],
        _ => return match l.kind {
            Kind::River { .. } => Some(&[
                r"  ___                                    ___",
                r"     \___   ~  ~   ~   ~  ~   ~   ~  ___/",
                r"         \__~___~___~___~___~___~___/",
            ]),
            Kind::Fort(_) => Some(&[
                r"         |>",
                r"   ______|______________",
                r"  |  _   _     _   _    |",
                r"  | |_| |_|   |#| |_|   |",
                r"  |_____________________|",
            ]),
            _ => None,
        },
    })
}

/// One line of the map: the road from Independence to Oregon City, with
/// the landmarks passed marked and the wagon where it is.
pub fn map_line(width: usize, travelled: f64, total: f64, marks: &[(f64, bool)]) -> (String, String) {
    let w = width.max(10);
    let at = |miles: f64| ((miles / total) * (w - 1) as f64).round().clamp(0.0, (w - 1) as f64) as usize;
    let mut road: Vec<char> = vec!['-'; w];
    let mut wagon: Vec<char> = vec![' '; w];
    for &(m, passed) in marks {
        road[at(m)] = if passed { '+' } else { 'o' };
    }
    road[0] = '[';
    road[w - 1] = ']';
    wagon[at(travelled)] = 'V';
    (wagon.into_iter().collect(), road.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::ROUTE;

    #[test]
    fn scenes_fit_the_page_and_the_map_marks_the_wagon() {
        for l in &ROUTE {
            if let Some(lines) = scene(l) {
                assert!(lines.iter().all(|s| s.chars().count() <= 70), "{}", l.name);
            }
        }
        let (wagon, road) = map_line(50, 500.0, 2000.0, &[(1000.0, false)]);
        assert_eq!(wagon.chars().position(|c| c == 'V'), Some(12));
        assert_eq!(road.chars().nth(25), Some('o'));
    }
}
