//! A small seeded random number generator (SplitMix64), so a seed always
//! plays the same journey: the headless runs and the balance tests rely on it.

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A float in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// True with probability p.
    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    /// An integer in lo..=hi.
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next() % (hi - lo + 1) as u64) as i64
    }

    /// One of the items, chosen by weight; None when all weights are zero.
    pub fn weighted<T: Copy>(&mut self, items: &[(T, f64)]) -> Option<T> {
        let total: f64 = items.iter().map(|(_, w)| w.max(0.0)).sum();
        if total <= 0.0 {
            return None;
        }
        let mut pick = self.unit() * total;
        for &(t, w) in items {
            let w = w.max(0.0);
            if pick < w {
                return Some(t);
            }
            pick -= w;
        }
        items.last().map(|&(t, _)| t)
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.next() as usize % items.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_seed_repeats_and_ranges_hold() {
        let (mut a, mut b) = (Rng::new(7), Rng::new(7));
        for _ in 0..1000 {
            assert_eq!(a.next(), b.next());
            let r = a.range(3, 9);
            assert!((3..=9).contains(&r));
            let u = b.unit();
            assert!((0.0..1.0).contains(&u));
        }
        assert_eq!(a.weighted(&[('x', 0.0), ('y', 1.0)]), Some('y'));
        assert_eq!(a.weighted::<char>(&[('x', 0.0)]), None);
    }
}
