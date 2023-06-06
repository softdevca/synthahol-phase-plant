//! [Sample Player](https://kilohearts.com/docs/phase_plant/#sample_player)
//! outputs a sampled waveform.
//!
//! Sampler was known as Sample Player prior to Phase Plant version 2.

use std::any::Any;

use log::trace;
use uom::si::f32::Frequency;

use super::*;

#[doc(alias = "Sampler")]
#[derive(Clone, Debug, PartialEq)]
pub struct SamplePlayer {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,

    pub semi_cent: f32,
    pub harmonic: f32,

    #[doc(alias = "fine_tuning")]
    pub shift: Frequency,

    pub phase_offset: Ratio,
    pub phase_jitter: Ratio,
    pub level: Ratio,
    pub unison: Unison,

    /// Where the sample starts playing
    pub offset_position: Ratio,
    pub offset_locked: bool,

    /// Beginning of the loop area
    pub loop_start_position: Ratio,
    pub loop_locked: bool,
    pub loop_length: Ratio,
    pub loop_enabled: bool,
    pub loop_mode: LoopMode,
    pub crossfade_amount: Ratio,

    /// A file containing the samples to play back. Usually in a format like FLAC, MP3 or WAV.
    pub sample_contents: Vec<u8>,

    pub sample_name: Option<String>,
    pub sample_path: Option<String>,

    #[doc(alias = "root note")]
    pub base_pitch: f32,
    pub base_pitch_locked: bool,
}

impl Default for SamplePlayer {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::SamplePlayer.name().to_owned(),
            loop_start_position: Ratio::new::<percent>(50.0),
            loop_length: Ratio::new::<percent>(25.0),
            crossfade_amount: Ratio::new::<percent>(1.0),
            ..Default::default()
        })
    }
}

impl From<&GeneratorBlock> for SamplePlayer {
    fn from(block: &GeneratorBlock) -> Self {
        trace!(
            "sample player: converting from block, sample content len = {}",
            block.sample_contents.len()
        );
        SamplePlayer {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            semi_cent: block.fine_tuning,
            harmonic: block.harmonic,
            shift: block.shift,
            phase_offset: block.phase_offset,
            phase_jitter: block.phase_jitter,
            level: block.level,
            unison: block.unison,
            offset_locked: block.offset_locked,
            offset_position: block.offset_position,
            loop_locked: block.loop_locked,
            loop_start_position: block.loop_start_position,
            loop_length: block.loop_length,
            loop_enabled: block.loop_enabled,
            loop_mode: block.sample_loop_mode,
            crossfade_amount: block.crossfade_amount,
            sample_contents: block.sample_contents.clone(),
            sample_name: block.sample_name.clone(),
            sample_path: block.sample_path.clone(),
            base_pitch: block.base_pitch,
            base_pitch_locked: block.base_pitch_locked,
        }
    }
}

impl Generator for SamplePlayer {
    fn id(&self) -> Option<GeneratorId> {
        Some(self.id)
    }

    fn as_block(&self) -> GeneratorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn mode(&self) -> GeneratorMode {
        GeneratorMode::SamplePlayer
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_sampler(&self) -> Option<&SamplePlayer> {
        self.downcast_ref::<SamplePlayer>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use music_note::midi;
    use uom::si::f32::Frequency;
    use uom::si::frequency::hertz;
    use uom::si::ratio::ratio;

    use crate::test::read_generator_preset;

    use super::*;

    // FIXME: Add version 2 tests

    // FIXME: Add fn default() to check that init matches the default

    fn assert_default(format_version: &Version<u32>, generator: &SamplePlayer) {
        assert!(generator.enabled);
        assert_eq!(generator.name(), "Sampler".to_owned());
        assert_eq!(generator.level.get::<percent>(), 100.0);
        assert!(!generator.base_pitch_locked);
        assert_eq!(generator.harmonic, 1.0);
        assert_eq!(generator.shift, Frequency::zero());
        assert_eq!(generator.phase_offset, Ratio::zero());
        assert_eq!(generator.phase_jitter, Ratio::zero());
        assert_eq!(generator.sample_name, None);
        assert_eq!(generator.sample_path, None);
        assert!(generator.sample_contents.is_empty());
        assert_eq!(generator.base_pitch, midi!(C, 4).into_byte() as f32);

        // Loop
        assert!(!generator.loop_enabled);
        assert!(!generator.loop_locked);
        assert!(!generator.offset_locked);
        assert_eq!(generator.offset_position.get::<percent>(), 0.0);
        assert_eq!(generator.loop_start_position.get::<percent>(), 50.0);
        assert_eq!(generator.loop_length.get::<percent>(), 25.0);
        assert_eq!(generator.crossfade_amount.get::<percent>(), 1.0);

        // In Phase Plant 1.8.5 the default loop mode changed from `Off` to `Infinite`.
        if format_version.is_at_least(&PhasePlantRelease::V1_8_5.format_version()) {
            assert_eq!(generator.loop_mode, LoopMode::Infinite);
        } else {
            assert_eq!(generator.loop_mode, LoopMode::Off);
        }

        // In Phase Plant 1.8.5 the default Unison voices changed from 1 to 4.
        if format_version.is_at_least(&PhasePlantRelease::V1_8_5.format_version()) {
            assert_eq!(generator.unison, Default::default());
        } else {
            assert_eq!(
                generator.unison,
                Unison {
                    voices: 1,
                    ..Default::default()
                }
            );
        }
    }

    #[test]
    fn default() {
        assert_default(&WRITE_SAME_AS.format_version(), &SamplePlayer::default());
    }

    #[test]
    fn init() {
        for file in &[
            "sample_player-1.7.0.phaseplant",
            "sample_player-1.7.5.phaseplant",
            "sample_player-1.7.11.phaseplant",
            "sample_player-1.8.0.phaseplant",
            "sample_player-1.8.4.phaseplant",
            "sample_player-1.8.5.phaseplant",
            "sample_player-1.8.13.phaseplant",
            "sample_player-2.0.12.phaseplant",
            "sample_player-2.1.0.phaseplant",
        ] {
            let preset = read_generator_preset("sample_player", file).unwrap();
            let generator: &SamplePlayer = preset.generator(1).unwrap();
            assert_default(&preset.format_version, generator);
        }
    }

    #[test]
    fn custom_sample() {
        let preset = read_generator_preset(
            "sample_player",
            "sample_player-custom-sample-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert_eq!(generator.sample_name, Some("sine-440-3sec".to_owned()));
        assert_eq!(
            generator.sample_path,
            Some("87b15e79acf1193fac4ab63484f7547622145200/sine-440-3sec.flac".to_owned())
        );
        assert_eq!(generator.sample_contents.len(), 217344);
        assert_eq!(&generator.sample_contents[..4], "fLaC".as_bytes());
    }

    #[test]
    fn disabled() {
        let preset =
            read_generator_preset("sample_player", "sample_player-disabled-1.8.16.phaseplant")
                .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert!(!generator.enabled);
    }

    #[test]
    fn name_and_path() {
        // 2tambos has the generators Group, Blank, Output, Sample Player, Sample Player
        let preset =
            read_generator_preset("sample_player", "sample_player-2tambos-1.8.18.phaseplant")
                .unwrap();
        let generator1: &SamplePlayer = preset.generator(3).unwrap();
        assert_eq!(generator1.level.get::<percent>(), 100.0);
        assert_eq!(generator1.sample_name, Some("Tambourine Hit 1".to_owned()));
        assert_eq!(
            generator1.sample_path,
            Some("factory/Alfheim/Tambourine/Tambourine Hit 1.flac".to_owned())
        );
        let generator2: &SamplePlayer = preset.generator(2).unwrap();
        assert_eq!(generator2.sample_name, Some("Tambourine Hit 2".to_owned()));
        assert_eq!(
            generator2.sample_path,
            Some("factory/Alfheim/Tambourine/Tambourine Hit 2.flac".to_owned())
        );

        // 3rhodes has the generators Group, Sample Player, Output, Sample Player, Sample Player
        let preset =
            read_generator_preset("sample_player", "sample_player-3rhodes-1.8.13.phaseplant")
                .unwrap();
        let generator1: &SamplePlayer = preset.generator(1).unwrap();
        assert_eq!(generator1.level.get::<percent>(), 100.0);
        assert_eq!(generator1.sample_name, Some("Roads A (C2)".to_owned()));
        assert_eq!(
            generator1.sample_path,
            Some("factory/Symplesound/Decays/Roads A (C2).flac".to_owned())
        );

        let generator2: &SamplePlayer = preset.generator(3).unwrap();
        assert_eq!(generator2.base_pitch, midi!(C, 4).into_byte() as f32);
        assert_eq!(generator2.sample_name, Some("Roads A (C4)".to_owned()));
        assert_eq!(
            generator2.sample_path,
            Some("factory/Symplesound/Decays/Roads A (C4).flac".to_owned())
        );

        let generator3: &SamplePlayer = preset.generator(4).unwrap();
        assert_eq!(generator3.sample_name, Some("Roads A (C6)".to_owned()));
        assert_eq!(
            generator3.sample_path,
            Some("factory/Symplesound/Decays/Roads A (C6).flac".to_owned())
        );
    }

    #[test]
    fn loop_alto() {
        let preset = read_generator_preset(
            "sample_player",
            "sample_player-alto_choir-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert!(generator.loop_enabled);
        assert_relative_eq!(
            generator.offset_position.get::<percent>(),
            5.74,
            epsilon = 0.01
        );
        assert_relative_eq!(
            generator.loop_start_position.get::<percent>(),
            32.72,
            epsilon = 0.001
        );
        assert_relative_eq!(generator.loop_length.get::<percent>(), 66.3, epsilon = 0.01);
        assert_relative_eq!(
            generator.crossfade_amount.get::<percent>(),
            3.05,
            epsilon = 0.001
        );
        assert_eq!(generator.loop_mode, LoopMode::Infinite);
        assert_eq!(generator.sample_name, Some("Alto Choir".to_owned()));
        assert_eq!(
            generator.sample_path,
            Some("factory/Symplesound/Choirs/Alto Choir.flac".to_owned())
        );

        let preset = read_generator_preset(
            "sample_player",
            "sample_player-alto_choir-no_loop-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert!(!generator.loop_enabled);

        let preset = read_generator_preset(
            "sample_player",
            "sample_player-alto_choir-loop-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert!(generator.loop_enabled);

        let preset = read_generator_preset(
            "sample_player",
            "sample_player-alto_choir-ping_pong-crossfade50-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert_relative_eq!(
            generator.offset_position.get::<percent>(),
            5.74,
            epsilon = 0.01
        );
        assert_relative_eq!(
            generator.loop_start_position.get::<percent>(),
            32.72,
            epsilon = 0.001
        );
        assert_relative_eq!(generator.loop_length.get::<percent>(), 66.3, epsilon = 0.01);
        assert_relative_eq!(
            generator.crossfade_amount.get::<percent>(),
            50.0,
            epsilon = 0.001
        );
        assert_eq!(generator.loop_mode, LoopMode::PingPong);

        let preset = read_generator_preset(
            "sample_player",
            "sample_player-alto_choir-reverse-length50-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert_relative_eq!(
            generator.offset_position.get::<percent>(),
            5.74,
            epsilon = 0.01
        );
        assert_relative_eq!(
            generator.loop_start_position.get::<percent>(),
            32.72,
            epsilon = 0.001
        );
        assert_relative_eq!(generator.loop_length.get::<percent>(), 50.0, epsilon = 0.01);
        assert_relative_eq!(
            generator.crossfade_amount.get::<percent>(),
            3.05,
            epsilon = 0.001
        );
        assert_eq!(generator.loop_mode, LoopMode::Reverse);

        let preset = read_generator_preset(
            "sample_player",
            "sample_player-alto_choir-sustain-start50-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert_relative_eq!(
            generator.offset_position.get::<percent>(),
            5.74,
            epsilon = 0.01
        );
        assert_relative_eq!(
            generator.loop_start_position.get::<percent>(),
            50.0,
            epsilon = 0.001
        );
        assert_relative_eq!(
            generator.loop_length.get::<percent>(),
            43.82,
            epsilon = 0.01
        );
        assert_relative_eq!(
            generator.crossfade_amount.get::<percent>(),
            3.05,
            epsilon = 0.001
        );
        assert_eq!(generator.loop_mode, LoopMode::Sustain);
    }

    #[test]
    fn loop_lock() {
        let preset = read_generator_preset(
            "sample_player",
            "sample_player-loop_lock-phase_offset15-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert!(generator.loop_locked);
        assert_relative_eq!(
            generator.phase_offset.get::<ratio>(),
            0.041667,
            epsilon = 0.001
        );
    }

    #[test]
    fn offset_lock() {
        let preset = read_generator_preset(
            "sample_player",
            "sample_player-offset_lock-shift15-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert!(generator.offset_locked);
        assert_relative_eq!(generator.shift.get::<hertz>(), 15.0, epsilon = 0.001);
    }

    #[test]
    fn tuning_lock() {
        let preset = read_generator_preset(
            "sample_player",
            "sample_player-root_lock-phase_jitter15-1.8.16.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert!(generator.base_pitch_locked);
        assert_relative_eq!(
            generator.phase_jitter.get::<ratio>(),
            0.041667,
            epsilon = 0.001
        );
    }

    #[test]
    fn root_offset_loop() {
        let preset = read_generator_preset(
            "sample_player",
            "sample_player-root_a4-offset33%-loop-1.8.13.phaseplant",
        )
        .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        assert!(!generator.loop_enabled);
        assert_eq!(generator.offset_position.get::<percent>(), 33.0);
        assert_eq!(generator.base_pitch, midi!(A, 4).into_byte() as f32);
    }

    #[test]
    fn unison() {
        let preset =
            read_generator_preset("sample_player", "sample_player-unison-1.8.16.phaseplant")
                .unwrap();
        let generator: &SamplePlayer = preset.generator(1).unwrap();
        let unison = generator.unison;
        assert!(unison.enabled);
        assert_eq!(unison.voices, 7);
        assert_eq!(unison.mode, UnisonMode::MinorMaj7);
        assert_relative_eq!(unison.detune_cents, 30.0);
        assert_relative_eq!(unison.spread.get::<percent>(), 15.0, epsilon = 0.001);
        assert_relative_eq!(unison.blend.get::<percent>(), 40.0, epsilon = 0.001);
        assert_relative_eq!(unison.bias.get::<percent>(), 15.0, epsilon = 0.001);
    }
}
