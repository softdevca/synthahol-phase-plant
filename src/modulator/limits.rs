//! [Lower Limit Modulator](https://kilohearts.com/docs/modulation#lower_limit_upper_limit)
//! and
//! [Upper Limit Modulator](https://kilohearts.com/docs/modulation#lower_limit_upper_limit).
//!
//! They were known as Min and Max prior to Phase Plant version 2.

use std::any::Any;

use uom::si::ratio::ratio;

use crate::modulator::{Modulator, ModulatorMode, OutputRange};
use crate::*;

/// Formerly known as "Max"
#[derive(Debug, PartialEq)]
pub struct LowerLimitModulator {
    pub depth: Ratio,
    pub output_range: OutputRange,
    pub input_a: f32,
    pub input_b: f32,
}

impl Default for LowerLimitModulator {
    fn default() -> Self {
        Self {
            depth: Ratio::new::<ratio>(1.0),
            output_range: OutputRange::Bipolar,
            input_a: 0.0,
            input_b: 0.0,
        }
    }
}

/// Formerly known as "Min"
#[derive(Debug, PartialEq)]
pub struct UpperLimitModulator {
    pub output_range: OutputRange,
    pub input_a: f32,
    pub input_b: f32,
    pub depth: Ratio,
}

impl Default for UpperLimitModulator {
    fn default() -> Self {
        Self {
            output_range: OutputRange::Bipolar,
            input_a: 0.0,
            input_b: 0.0,
            depth: Ratio::new::<ratio>(1.0),
        }
    }
}

impl Modulator for LowerLimitModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::LowerLimit
    }
}

impl Modulator for UpperLimitModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::UpperLimit
    }
}

#[cfg(test)]
mod test {
    //! The tests for Phase Plant version 1 presets are named `min` and `max`.
    //! The tests for version 2 preset are `upper_limit` and `lower_limit` to
    //! reflect the name change in version 2.

    use approx::assert_relative_eq;
    use uom::si::ratio::percent;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn max_init() {
        for file in &["max-1.8.13.phaseplant", "lower_limit-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("limits", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let modulator: &LowerLimitModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &Default::default());
        }
    }

    #[test]
    fn max_bypassed() {
        let preset = read_modulator_preset("limits", "max-bypassed-1.8.13.phaseplant").unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(!container.minimized);
        let modulator: &LowerLimitModulator = preset.modulator(0).unwrap();
        assert!(!container.enabled);
        assert_eq!(OutputRange::Bipolar, modulator.output_range);
        assert_eq!(modulator.input_a, 0.0);
        assert_eq!(modulator.input_b, 0.0);
        assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
    }

    #[test]
    fn max_values() {
        let preset =
            read_modulator_preset("limits", "max-a25-b-25-depth50-1.8.13.phaseplant").unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(container.enabled);
        assert!(!container.minimized);
        let modulator: &LowerLimitModulator = preset.modulator(0).unwrap();
        assert_eq!(OutputRange::Bipolar, modulator.output_range);
        assert_eq!(modulator.input_a, 0.25);
        assert_eq!(modulator.input_b, -0.25);
        assert_relative_eq!(modulator.depth.get::<percent>(), 50.0);
    }

    #[test]
    fn max_minimized() {
        let preset = read_modulator_preset("limits", "max-minimized-1.8.13.phaseplant").unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(container.enabled);
        assert!(container.minimized);
        let modulator: &LowerLimitModulator = preset.modulator(0).unwrap();
        assert_eq!(OutputRange::Bipolar, modulator.output_range);
        assert_eq!(modulator.input_a, 0.0);
        assert_eq!(modulator.input_b, 0.0);
        assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
    }

    #[test]
    fn min_init() {
        for file in &["min-1.8.13.phaseplant", "upper_limit-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("limits", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let modulator: &UpperLimitModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &Default::default());
        }
    }
}
