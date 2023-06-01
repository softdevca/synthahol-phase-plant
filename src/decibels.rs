#![allow(clippy::excessive_precision)]

use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Decibels(f32);

impl Decibels {
    pub const INFINITY: Decibels = Decibels::new(f32::INFINITY);
    pub const ZERO: Decibels = Decibels::new(0.0);

    pub const fn new(db: f32) -> Decibels {
        Decibels(db)
    }

    pub fn from_linear(linear: f32) -> Decibels {
        Decibels::new(linear.log10() * 20.0)
    }

    pub fn db(&self) -> f32 {
        self.0
    }

    /// Conversion between decibels and linear values.
    pub fn linear(&self) -> f32 {
        10.0_f32.powf(self.0 / 20.0)
    }
}

impl Display for Decibels {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} dB", &self.0)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::Decibels;

    #[test]
    fn constants() {
        assert_eq!(Decibels::INFINITY.db(), f32::INFINITY);
        assert_eq!(Decibels::INFINITY.linear(), f32::INFINITY);
        assert_eq!(Decibels::ZERO.db(), 0.0);
        assert_eq!(Decibels::ZERO.linear(), 1.0);
    }

    #[test]
    fn eq() {
        assert_eq!(Decibels(1.0), Decibels(1.0));
        assert_ne!(Decibels(-1.0), Decibels(1.0));
    }

    #[test]
    fn identity() {
        assert_relative_eq!(Decibels::new(10.0).db(), 10.0, epsilon = 0.00001);
        assert_relative_eq!(Decibels::from_linear(0.5).linear(), 0.5, epsilon = 0.00001);
    }

    #[test]
    fn linear_to_db() {
        assert_relative_eq!(Decibels::from_linear(1.0).0, 0.0);
        assert_relative_eq!(Decibels::from_linear(0.0).db(), f32::NEG_INFINITY);
        assert_relative_eq!(Decibels::from_linear(1.4125375446227544).0, 3.0);
        assert_relative_eq!(Decibels::from_linear(3.1622776601683795).db(), 10.0);
        assert_relative_eq!(Decibels::from_linear(0.31622776601683794).0, -10.0);
    }

    #[test]
    fn db_to_linear() {
        assert_relative_eq!(Decibels::new(0.0).linear(), 1.0);
        assert_relative_eq!(Decibels::new(f32::NEG_INFINITY).linear(), 0.0);
        assert_relative_eq!(Decibels::new(3.0).linear(), 1.4125375446227544);
        assert_relative_eq!(Decibels::new(10.0).linear(), 3.1622776601683795);
        assert_relative_eq!(Decibels::new(-10.0).linear(), 0.31622776601683794);
    }
}
