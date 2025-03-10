//! [Pitch Tracker Modulator](https://kilohearts.com/docs/modulation#pitch_tracker)
//! converts the frequency of audio to a modulation signal.

use std::any::Any;

use music_note::midi;
use uom::si::f32::Ratio;
use uom::si::ratio::percent;

use crate::*;

use super::*;

#[derive(Debug, PartialEq)]
pub struct PitchTrackerModulator {
    pub depth: Ratio,
    pub output_range: OutputRange,
    pub audio_source: AudioSourceId,
    pub sensitivity: Ratio,
    pub lowest_note: u32,
    pub root_note: u32,
    pub highest_note: u32,
}

impl Default for PitchTrackerModulator {
    fn default() -> Self {
        Self {
            depth: Ratio::new::<percent>(100.0),
            output_range: OutputRange::Unipolar,
            sensitivity: Ratio::zero(),
            audio_source: AudioSourceId::default(),
            lowest_note: midi!(C, 2).into_byte() as u32,
            root_note: midi!(A, 4).into_byte() as u32,
            highest_note: midi!(C, 6).into_byte() as u32,
        }
    }
}

impl Modulator for PitchTrackerModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::PitchTracker
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
        for file in &[
            "pitch_tracker-2.0.0.phaseplant",
            "pitch_tracker-2.0.12.phaseplant",
            "pitch_tracker-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("pitch_tracker", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            assert_eq!(container.id, 0);
            let modulator: &PitchTrackerModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator.depth.get::<percent>(), 100.0);
            assert_eq!(modulator.output_range, OutputRange::Bipolar);
            assert!(modulator.audio_source.is_master());
            assert_eq!(modulator.sensitivity.get::<percent>(), 50.0);
            assert_eq!(modulator.lowest_note, midi!(C, 2).into_byte() as u32);
            assert_eq!(modulator.root_note, midi!(A, 4).into_byte() as u32);
            assert_eq!(modulator.highest_note, midi!(C, 6).into_byte() as u32);
        }
    }

    #[test]
    fn notes() {
        let preset =
            read_modulator_preset("pitch_tracker", "pitch_tracker-d1-to-d7-2.0.12.phaseplant")
                .unwrap();
        let modulator: &PitchTrackerModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.lowest_note, midi!(D, 1).into_byte() as u32);
        assert_eq!(modulator.highest_note, midi!(D, 7).into_byte() as u32);

        let preset =
            read_modulator_preset("pitch_tracker", "pitch_tracker-d1-a5-d7-2.1.0.phaseplant")
                .unwrap();
        let modulator: &PitchTrackerModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.lowest_note, midi!(D, 1).into_byte() as u32);
        assert_eq!(modulator.root_note, midi!(A, 5).into_byte() as u32);
        assert_eq!(modulator.highest_note, midi!(D, 7).into_byte() as u32);
    }

    #[test]
    fn parts() {
        let preset = read_modulator_preset(
            "pitch_tracker",
            "pitch_tracker-lane2-sens75-2.0.12.phaseplant",
        )
        .unwrap();
        let modulator: &PitchTrackerModulator = preset.modulator(0).unwrap();
        assert!(modulator.audio_source.is_lane_2());
        assert_relative_eq!(
            modulator.sensitivity.get::<percent>(),
            75.0,
            epsilon = 0.0001
        );
    }
}
