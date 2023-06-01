//! [Random Modulator](https://kilohearts.com/docs/modulation#random)

use std::any::Any;

use super::*;

#[derive(Clone, Copy, Debug, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum VoiceMode {
    // The discriminants correspond to the file format.
    Unison = 0,
    Independent = 1,
}

impl VoiceMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown voice mode {id}")))
    }
}

impl Display for VoiceMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            VoiceMode::Unison => "Unison",
            VoiceMode::Independent => "Independent",
        };
        f.write_str(msg)
    }
}

#[derive(Debug, PartialEq)]
pub struct RandomModulator {
    pub output_range: OutputRange,
    pub depth: Ratio,
    pub rate: Rate,
    pub jitter: f32,
    pub smooth: f32,
    pub chaos: f32,
    pub note_trigger_mode: NoteTriggerMode,
    pub trigger_threshold: f32,
    pub voice_mode: VoiceMode,
}

impl Modulator for RandomModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Random
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::frequency::hertz;
    use uom::si::ratio::percent;

    use crate::test::read_modulator_preset;

    use super::*;

    #[test]
    fn init() {
        for file in &[
            "random-1.7.0.phaseplant",
            "random-1.8.13.phaseplant",
            "random-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("random", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            assert_eq!(container.group_id, GROUP_ID_NONE);
            let modulator: &RandomModulator = preset.modulator(0).unwrap();
            assert_eq!(OutputRange::Bipolar, modulator.output_range);
            assert_relative_eq!(modulator.depth.get::<percent>(), 100.0);
            assert!(!modulator.rate.sync);
            assert_eq!(modulator.rate.frequency.get::<hertz>(), 1.0);
            assert_eq!(modulator.rate.numerator, 4);
            assert_eq!(modulator.rate.denominator, NoteValue::Sixteenth);
            assert_eq!(modulator.jitter, 0.0);
            assert_eq!(modulator.smooth, 0.0);
            assert_eq!(modulator.chaos, 1.0);
            assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Auto);
            assert_eq!(modulator.trigger_threshold, 0.5);
            assert_eq!(modulator.voice_mode, VoiceMode::Unison);
        }
    }

    #[test]
    fn jitter_smooth_chaos() {
        let preset =
            read_modulator_preset("random", "random-jit10-smo20-cha30-1.8.17.phaseplant").unwrap();
        let modulator: &RandomModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.jitter, 0.1);
        assert_eq!(modulator.smooth, 0.2);
        assert_eq!(modulator.chaos, 0.3);
    }

    #[test]
    fn thirty_two_of_them() {
        let preset = read_modulator_preset("random", "random-32_of_them-2.1.0.phaseplant").unwrap();
        assert_eq!(preset.modulator_containers.len(), 32);
        for index in 0..preset.modulator_containers.len() {
            let modulator: &RandomModulator = preset.modulator(index).unwrap();
            assert_eq!(modulator.output_range, OutputRange::Bipolar);
        }
    }

    #[test]
    fn triggers() {
        let preset = read_modulator_preset(
            "random",
            "random-trigger25-legato-independent-2.1.0.phaseplant",
        )
        .unwrap();
        let modulator: &RandomModulator = preset.modulator(0).unwrap();
        assert_eq!(modulator.trigger_threshold, 0.25);
        assert_eq!(modulator.note_trigger_mode, NoteTriggerMode::Legato);
        assert_eq!(modulator.voice_mode, VoiceMode::Independent);
    }
}
