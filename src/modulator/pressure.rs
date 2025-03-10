//! [Pressure Modulator](https://kilohearts.com/docs/modulation#pressure)
//! converts MIDI messages into modulation control signals.

use std::any::Any;

use uom::si::ratio::ratio;

use super::*;

#[derive(Debug, PartialEq)]
pub struct PressureModulator {
    pub depth: Ratio,
    pub output_range: OutputRange,
}

impl Default for PressureModulator {
    fn default() -> Self {
        Self {
            depth: Ratio::new::<ratio>(1.0),
            output_range: OutputRange::Unipolar,
        }
    }
}

impl Modulator for PressureModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Pressure
    }
}

#[cfg(test)]
mod test {
    use uom::si::ratio::percent;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn default() {
        let modulator = PressureModulator::default();
        assert_eq!(modulator.depth.get::<percent>(), 100.0);
        assert_eq!(modulator.output_range, OutputRange::Unipolar);
    }

    #[test]
    fn init() {
        for file in &["pressure-1.8.13.phaseplant", "pressure-2.1.0.phaseplant"] {
            let preset = read_modulator_preset("pressure", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &PressureModulator = preset.modulator(0).unwrap();
            assert_eq!(&PressureModulator::default(), modulator);
        }
    }

    #[test]
    fn parts() {
        let preset = read_modulator_preset(
            "pressure",
            "pressure-depth50-bipolar-disabled-2.1.0.phaseplant",
        )
        .unwrap();
        let container = preset.modulator_container(0).unwrap();
        assert!(!container.enabled);
        assert!(!container.minimized);
        let modulator: &PressureModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.depth.get::<percent>(), 50.0);
        assert_eq!(modulator.output_range, OutputRange::Bipolar);
    }
}
