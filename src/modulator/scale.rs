//! [Scale Modulator](https://kilohearts.com/docs/modulation#scale). The Scale
//! modulator was known as Multiply prior to Phase Plant version 2.

use std::any::Any;

use super::*;

#[derive(Debug, PartialEq)]
pub struct ScaleModulator {
    pub output_range: OutputRange,
    pub input_a: f32,
    pub input_b: f32,
    pub multiplier: f32,
    pub depth: Ratio,
}

impl Modulator for ScaleModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Scale
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::percent;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &["multiply-1.8.13.phaseplant", "scale-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("scale", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            assert_eq!(container.group_id, GROUP_ID_NONE);
            let modulator: &ScaleModulator = preset.modulator(0).unwrap();
            assert_eq!(OutputRange::Bipolar, modulator.output_range);
            assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
            assert_eq!(modulator.input_a, 0.0);
            assert_eq!(modulator.input_b, 1.0);
            assert_eq!(modulator.multiplier, 1.0);
        }
    }

    #[test]
    fn minimized_ab() {
        let preset =
            read_modulator_preset("scale", "multiply-a-1-b1-minimized-1.8.14.phaseplant").unwrap();
        assert_eq!(preset.modulator_containers.len(), 1);
        let container = preset.modulator_container(0).unwrap();
        assert!(container.enabled);
        assert!(container.minimized);
        let modulator: &ScaleModulator = preset.modulator(0).unwrap();
        assert_eq!(OutputRange::Bipolar, modulator.output_range);
        assert_eq!(modulator.input_a, -1.0);
        assert_eq!(modulator.input_b, 1.0);
        assert_eq!(modulator.multiplier, 1.0);
        assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
    }

    #[test]
    fn multiplier() {
        let preset =
            read_modulator_preset("scale", "multiply-multiplier1000-1.8.17.phaseplant").unwrap();
        assert_eq!(preset.modulator_containers.len(), 1);
        let container = preset.modulator_container(0).unwrap();
        assert!(container.enabled);
        assert!(!container.minimized);
        let modulator: &ScaleModulator = preset.modulator(0).unwrap();
        assert_eq!(OutputRange::Bipolar, modulator.output_range);
        assert_eq!(modulator.input_a, 0.0);
        assert_eq!(modulator.input_b, 1.0);
        assert_eq!(modulator.multiplier, 1000.0);
        assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
    }

    #[test]
    fn parts() {
        let preset = read_modulator_preset(
            "scale",
            "multiply-static5-depth66-disabled-1.8.14.phaseplant",
        )
        .unwrap();
        assert_eq!(preset.modulator_containers.len(), 1);
        let container = preset.modulator_container(0).unwrap();
        assert!(!container.enabled);
        assert!(!container.minimized);
        let modulator: &ScaleModulator = preset.modulator(0).unwrap();
        assert_eq!(OutputRange::Bipolar, modulator.output_range);
        assert_eq!(modulator.input_a, 0.0);
        assert_eq!(modulator.input_b, 1.0);
        assert_eq!(modulator.multiplier, 10.0);
        assert_relative_eq!(modulator.depth.get::<percent>(), 66.0);
    }
}
