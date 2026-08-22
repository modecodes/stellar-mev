use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub const STROOPS_PER_UNIT: i64 = 10_000_000;

/// Represents an amount in Stellar Stroops (1 XLM = 10,000,000 Stroops)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StroopAmount(pub i64);

impl StroopAmount {
    pub fn zero() -> Self {
        StroopAmount(0)
    }

    pub fn from_stroops(stroops: i64) -> Self {
        StroopAmount(stroops)
    }

    pub fn from_units(units: f64) -> Self {
        StroopAmount((units * STROOPS_PER_UNIT as f64).round() as i64)
    }

    pub fn as_stroops(&self) -> i64 {
        self.0
    }

    pub fn as_units(&self) -> f64 {
        self.0 as f64 / STROOPS_PER_UNIT as f64
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn is_positive(&self) -> bool {
        self.0 > 0
    }
}

impl Add for StroopAmount {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        StroopAmount(self.0 + rhs.0)
    }
}

impl AddAssign for StroopAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for StroopAmount {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        StroopAmount(self.0 - rhs.0)
    }
}

impl SubAssign for StroopAmount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl fmt::Display for StroopAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.7}", self.as_units())
    }
}
