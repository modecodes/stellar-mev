use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Stellar SDEX Price fraction n/d (numerator / denominator)
#[derive(Debug, Clone, Copy, Eq, Serialize, Deserialize)]
pub struct Price {
    pub n: i32,
    pub d: i32,
}

impl Price {
    pub fn new(n: i32, d: i32) -> Self {
        assert!(d != 0, "Price denominator cannot be zero");
        Self { n, d }
    }

    pub fn from_f64(val: f64) -> Self {
        if val <= 0.0 {
            return Self::new(1, 1);
        }
        let d = 10_000_000;
        let n = (val * d as f64).round() as i32;
        let gcd = gcd(n.abs(), d);
        Self::new(n / gcd, d / gcd)
    }

    pub fn as_f64(&self) -> f64 {
        self.n as f64 / self.d as f64
    }

    pub fn inverse(&self) -> Self {
        Self::new(self.d, self.n)
    }
}

impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        (self.n as i64 * other.d as i64) == (other.n as i64 * self.d as i64)
    }
}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.n as i128 * other.d as i128;
        let right = other.n as i128 * self.d as i128;
        left.cmp(&right)
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn gcd(mut a: i32, mut b: i32) -> i32 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.max(1)
}
