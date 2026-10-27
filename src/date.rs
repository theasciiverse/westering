//! Dates in 1848 (a leap year), counted in days from the first of January.

const MONTH_DAYS: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTH_NAMES: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// The day of the year a month (1 to 12) begins, counting from 0.
pub fn first_of(month: u32) -> u32 {
    MONTH_DAYS[..(month - 1) as usize].iter().sum()
}

/// The month (1 to 12) a day falls in; past the year's end, December.
pub fn month(day: u32) -> u32 {
    let mut d = day;
    for (i, &n) in MONTH_DAYS.iter().enumerate() {
        if d < n {
            return i as u32 + 1;
        }
        d -= n;
    }
    12
}

pub fn format(day: u32) -> String {
    let m = month(day);
    let d = day.saturating_sub(first_of(m)) + 1;
    format!("{} {}, 1848", MONTH_NAMES[(m - 1) as usize], d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_and_months() {
        assert_eq!(format(0), "January 1, 1848");
        assert_eq!(format(first_of(3)), "March 1, 1848");
        assert_eq!(format(first_of(3) - 1), "February 29, 1848");
        assert_eq!(month(first_of(4) + 10), 4);
    }
}
