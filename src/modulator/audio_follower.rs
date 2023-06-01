//! [Audio Follower](https://kilohearts.com/docs/modulation#audio_follower)
//! converts the amplitude of audio to a modulation signal.

use std::any::Any;

use uom::si::f32::Ratio;
use uom::si::ratio::percent;
use uom::si::time::millisecond;

use crate::*;

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct AudioSourceId {
    id: u32,
    name: String,
}

/// The default audio source is `Master`
impl Default for AudioSourceId {
    fn default() -> Self {
        Self::new(Self::bytes_to_id(b"main"), "Master".to_owned())
    }
}

impl Display for AudioSourceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl AudioSourceId {
    pub fn new(id: u32, name: String) -> Self {
        Self { id, name }
    }

    pub fn is_lane_1(&self) -> bool {
        self.id == Self::bytes_to_id(b"lan1")
    }

    pub fn is_lane_2(&self) -> bool {
        self.id == Self::bytes_to_id(b"lan2")
    }

    pub fn is_lane_3(&self) -> bool {
        self.id == Self::bytes_to_id(b"lan3")
    }

    pub fn is_master(&self) -> bool {
        self.id == Self::bytes_to_id(b"main")
    }

    const fn bytes_to_id(bytes: &[u8; 4]) -> u32 {
        u32::from_be_bytes(*bytes)
    }
}

#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum MeteringMode {
    // The discriminants correspond to the file format.
    Peak = 0,
    #[doc(alias = "RMS")]
    RootMeanSquared = 1,
}

impl MeteringMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown metering mode {id}"),
            )
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct AudioFollowerModulator {
    pub depth: Ratio,
    pub output_range: OutputRange,
    pub gain: Decibels,
    pub attack_time: Time,
    pub release_time: Time,
    pub audio_source: AudioSourceId,
    pub metering_mode: MeteringMode,
}

impl Default for AudioFollowerModulator {
    fn default() -> Self {
        Self {
            depth: Ratio::new::<percent>(100.0),
            output_range: OutputRange::Unipolar,
            gain: Decibels::ZERO,
            attack_time: Time::new::<millisecond>(10.0),
            release_time: Time::new::<millisecond>(100.0),
            audio_source: AudioSourceId::default(),
            metering_mode: MeteringMode::RootMeanSquared,
        }
    }
}

impl Modulator for AudioFollowerModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::AudioFollower
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
            "audio_follower-2.0.12.phaseplant",
            "audio_follower-2.1.0.phaseplant",
        ] {
            let preset = read_modulator_preset("audio_follower", file).unwrap();
            assert_eq!(preset.modulator_containers.len(), 1);
            let container = preset.modulator_container(0).unwrap();
            assert!(container.enabled);
            assert!(!container.minimized);
            assert_eq!(container.id, 0);
            let modulator: &AudioFollowerModulator = preset.modulator(0).unwrap();
            assert_eq!(modulator.depth.get::<percent>(), 100.0);
            assert_eq!(modulator.output_range, OutputRange::Unipolar);
            assert_eq!(modulator.gain.db(), 0.0);
            assert_relative_eq!(
                modulator.attack_time.get::<millisecond>(),
                10.0,
                epsilon = 0.0001
            );
            assert_relative_eq!(
                modulator.release_time.get::<millisecond>(),
                100.0,
                epsilon = 0.0001
            );
            assert!(modulator.audio_source.is_master());
            assert_eq!(modulator.metering_mode, MeteringMode::RootMeanSquared);
        }
    }

    #[test]
    fn parts() {
        let preset = read_modulator_preset(
            "audio_follower",
            "audio_follower-depth25-bipolar-2.1.0.phaseplant",
        )
        .unwrap();
        let modulator: &AudioFollowerModulator = preset.modulator(0).unwrap();
        assert_relative_eq!(modulator.depth.get::<percent>(), 25.0);
        assert_eq!(modulator.output_range, OutputRange::Bipolar);

        let preset = read_modulator_preset(
            "audio_follower",
            "audio_follower-gain10-att20-release200-2.0.12.phaseplant",
        )
        .unwrap();
        let modulator: &AudioFollowerModulator = preset.modulator(0).unwrap();
        assert_relative_eq!(modulator.gain.db(), 10.0, epsilon = 0.0001);
        assert_relative_eq!(
            modulator.attack_time.get::<millisecond>(),
            20.0,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            modulator.release_time.get::<millisecond>(),
            200.0,
            epsilon = 0.0001
        );

        let preset = read_modulator_preset(
            "audio_follower",
            "audio_follower-lane1-peak-2.0.12.phaseplant",
        )
        .unwrap();
        let modulator: &AudioFollowerModulator = preset.modulator(0).unwrap();
        assert!(modulator.audio_source.is_lane_1());
        assert_eq!(modulator.metering_mode, MeteringMode::Peak);
    }
}
