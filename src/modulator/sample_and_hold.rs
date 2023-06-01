//! [Sample & Hold Modulator](https://kilohearts.com/docs/modulation#sample_and_hold)

use std::any::Any;

use uom::si::ratio::ratio;

use crate::modulator::{Modulator, ModulatorMode};

use super::*;

#[derive(Debug, PartialEq)]
pub struct SampleAndHoldModulator {
    pub depth: Ratio,
    pub note_trigger_mode: NoteTriggerMode,
    pub trigger_threshold: f32,
    pub input_a: f32,
    pub input_b: f32,
}

impl Default for SampleAndHoldModulator {
    fn default() -> Self {
        Self {
            depth: Ratio::new::<ratio>(1.0),
            note_trigger_mode: NoteTriggerMode::Auto,
            trigger_threshold: 0.5,
            input_a: 0.0,
            input_b: 0.0,
        }
    }
}
impl Modulator for SampleAndHoldModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::SampleAndHold
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &[
            "sample_and_hold-2.0.0.phaseplant",
            "sample_and_hold-2.0.12.phaseplant",
            "sample_and_hold-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("sample_and_hold", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            let modulator: &SampleAndHoldModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator, &Default::default());
        }
    }

    #[test]
    fn parts() {
        let preset = read_modulator_preset(
            "sample_and_hold",
            "sample_and_hold-a50-b75-2.1.0.phaseplant",
        )
        .unwrap();
        let modulator: &SampleAndHoldModulator = preset.modulator(0).unwrap();
        assert_relative_eq!(modulator.input_a, 0.5);
        assert_relative_eq!(modulator.input_b, 0.75);

        let preset = read_modulator_preset(
            "sample_and_hold",
            "sample_and_hold-thresh25-always-2.1.0.phaseplant",
        )
        .unwrap();
        let modulator: &SampleAndHoldModulator = preset.modulator(0).unwrap();
        assert_relative_eq!(modulator.trigger_threshold, 0.25);
        assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::NoteOn);
    }
}
