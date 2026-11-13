//! The map of the road so far, in plain characters.

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

    #[test]
    fn the_map_marks_the_wagon() {
        let (wagon, road) = map_line(50, 500.0, 2000.0, &[(1000.0, false)]);
        assert_eq!(wagon.chars().position(|c| c == 'V'), Some(12));
        assert_eq!(road.chars().nth(25), Some('o'));
    }
}
