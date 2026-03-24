//! Strong types for geometric metrics to avoid primitive obsession.

use crate::document::OrderedFloat;
use std::ops::{Add, Div, Mul, Sub};

/// A coordinate in world or local space.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Coordinate(pub f64);

impl Coordinate {
    pub const ZERO: Self = Self(0.0);
    pub const MAX: Self = Self(f64::MAX);
    pub const MIN: Self = Self(f64::MIN);

    #[must_use]
    pub const fn new(val: f64) -> Self {
        Self(val)
    }

    #[must_use]
    pub const fn min(self, other: Self) -> Self {
        if self.0 < other.0 {
            self
        } else {
            other
        }
    }

    #[must_use]
    pub const fn max(self, other: Self) -> Self {
        if self.0 > other.0 {
            self
        } else {
            other
        }
    }

    #[must_use]
    pub const fn is_finite(self) -> bool {
        self.0.is_finite()
    }
}

/// A scale factor.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ScaleFactor(pub f64);

impl ScaleFactor {
    #[must_use]
    pub const fn new(val: f64) -> Self {
        Self(val)
    }

    #[must_use]
    pub const fn one() -> Self {
        Self(1.0)
    }
}

/// An angle in radians.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Radians(pub f64);

impl Radians {
    #[must_use]
    pub const fn new(val: f64) -> Self {
        Self(val)
    }
}

/// Metrics for a rectangular element.
#[derive(Debug, Clone, Copy)]
pub struct RectMetrics {
    pub x: Coordinate,
    pub y: Coordinate,
    pub width: Coordinate,
    pub height: Coordinate,
}

impl RectMetrics {
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x: Coordinate(x),
            y: Coordinate(y),
            width: Coordinate(width),
            height: Coordinate(height),
        }
    }

    #[must_use]
    pub fn right(&self) -> Coordinate {
        self.x + self.width
    }

    #[must_use]
    pub fn bottom(&self) -> Coordinate {
        self.y + self.height
    }
}

impl Add for Coordinate {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for Coordinate {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Mul<f64> for Coordinate {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self(self.0 * rhs)
    }
}

impl Div<f64> for Coordinate {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self(self.0 / rhs)
    }
}

impl From<OrderedFloat> for Coordinate {
    fn from(val: OrderedFloat) -> Self {
        Self(val.0)
    }
}
