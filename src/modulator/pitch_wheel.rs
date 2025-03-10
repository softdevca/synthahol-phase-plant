//! [Pitch Wheel Modulator](https://kilohearts.com/docs/modulation#pitch_wheel)
//! converts MIDI messages into modulation control signals.

use std::any::Any;

use uom::si::ratio::ratio;

use super::*;

#[derive(Debug, PartialEq)]
pub struct PitchWheelModulator {
    pub depth: Ratio,
    pub output_range: OutputRange,
}

impl Default for PitchWheelModulator {
    fn default() -> Self {
        Self {
            depth: Ratio::new::<ratio>(1.0),
            output_range: OutputRange::Bipolar,
        }
    }
}

impl Modulator for PitchWheelModulator {
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
        let modulator = PitchWheelModulator::default();
        assert_eq!(modulator.depth.get::<percent>(), 100.0);
        assert_eq!(modulator.output_range, OutputRange::Bipolar);
    }

    #[test]
    fn init() {
        for file in &[
            "pitch_wheel-2.0.0.phaseplant",
            "pitch_wheel-2.0.12.phaseplant",
            "pitch_wheel-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("pitch_wheel", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &PitchWheelModulator = preset.modulator(0).unwrap();
            assert_eq!(&PitchWheelModulator::default(), modulator);
        }
    }

    #[test]
    fn parts() {
        let preset = read_modulator_preset(
            "pitch_wheel",
            "pitch_wheel-depth50-inverted-2.1.0.phaseplant",
        )
        .unwrap();
        let modulator: &PitchWheelModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.depth.get::<percent>(), 50.0);
        assert_eq!(modulator.output_range, OutputRange::Inverted);
    }
}
