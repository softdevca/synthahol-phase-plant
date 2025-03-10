//! [Granular Generator](https://kilohearts.com/docs/phase_plant#granular-generator)
//! plays short snippets from a sample.
//!
//! Granular Generator was added in Phase Plant 2.1.0;

use std::any::Any;

use uom::si::f32::Frequency;
use uom::si::ratio::percent;

use super::*;

/// Strumming pattern.
#[derive(Clone, Copy, Debug, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum ChordPickingPattern {
    // The discriminants correspond to the file format.
    Up = 0,
    Down = 1,
    UpDown = 2,
    Random = 3,
}

impl ChordPickingPattern {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown chord picking pattern {id}"),
            )
        })
    }
}

impl Display for ChordPickingPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ChordPickingPattern::*;
        let msg = match self {
            Up => "Up",
            Down => "Down",
            UpDown => "Up-Down",
            Random => "Random",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Copy, Debug, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum GranularSpawnRateMode {
    // The discriminants correspond to the file format.
    Rate = 0,
    Sync = 1,
    Density = 2,
}

impl GranularSpawnRateMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown grain spawn rate mode {id}"),
            )
        })
    }
}

impl Display for GranularSpawnRateMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            GranularSpawnRateMode::Rate => "Free Rate",
            GranularSpawnRateMode::Sync => "Synced Rate",
            GranularSpawnRateMode::Density => "Density",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GranularRandomization {
    pub position: Ratio,
    pub timing: Ratio,
    pub pitch: Frequency,
    pub level: Ratio,
    pub pan: Ratio,
    pub reverse: Ratio,
}

impl Default for GranularRandomization {
    fn default() -> Self {
        Self {
            position: Ratio::zero(),
            timing: Ratio::zero(),
            pitch: Frequency::zero(),
            level: Ratio::zero(),
            pan: Ratio::zero(),
            reverse: Ratio::zero(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum GranularChordMode {
    // The discriminants correspond to the file format.
    Octaves = 0,
    Fifths = 1,
    Minor = 2,
    MinorMin7 = 3,
    MinorMaj7 = 4,
    Major = 5,
    MajorMin7 = 6,
    MajorMaj7 = 7,
    Sus2 = 8,
    Sus4 = 9,
    Dim = 10,
    Dim7 = 11,
    PentatonicMaj = 12,
    PentatonicMinor = 13,
}

impl GranularChordMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown chord mode {id}")))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GranularChord {
    pub enabled: bool,
    pub picking_pattern: ChordPickingPattern,
    pub mode: GranularChordMode,
    pub range_octaves: f32,
}

impl Default for GranularChord {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: GranularChordMode::Major,
            picking_pattern: ChordPickingPattern::Random,
            range_octaves: 1.0,
        }
    }
}

/// Directions new grains travel.
#[derive(Copy, Clone, Debug, Eq, FromRepr, PartialEq)]
#[repr(u32)]
pub enum GranularDirection {
    // The discriminants correspond to the file format.
    /// Unipolar
    Start = 0,

    /// Bipolar
    Midpoint = 1,
}

impl GranularDirection {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown granular direction {id}"),
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GranularEnvelope {
    pub attack_time: Ratio,
    pub attack_curve: f32,
    pub decay_time: Ratio,
    pub decay_curve: f32,
}

impl Default for GranularEnvelope {
    fn default() -> Self {
        Self {
            attack_time: Ratio::new::<percent>(25.0),
            attack_curve: 0.0,
            decay_time: Ratio::new::<percent>(25.0),
            decay_curve: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GranularGenerator {
    pub id: GeneratorId,
    pub enabled: bool,
    pub name: String,
    pub fine_tuning: f32,
    pub harmonic: f32,
    pub shift: Frequency,

    /// Percentage of 360 degrees.
    pub phase_offset: Ratio,

    /// Percentage of 360 degrees.
    pub phase_jitter: Ratio,

    /// Amplitude of the waveform. Gain is set in the Out generator.
    pub level: Ratio,

    /// A file containing the samples to play back. Usually in a format like FLAC, MP3 or WAV.
    pub sample_contents: Vec<u8>,

    pub sample_name: Option<String>,
    pub sample_path: Option<String>,

    #[doc(alias = "root note")]
    pub base_pitch: f32,
    pub base_pitch_locked: bool,

    /// Playback position.
    pub position: Ratio,

    pub direction: GranularDirection,
    pub envelope: GranularEnvelope,
    pub align_phases: bool,
    pub grains: f32,
    pub grain_length: Time,

    // Adjust the length relative to the note
    pub auto_grain_length: bool,

    pub spawn_rate_mode: GranularSpawnRateMode,
    pub randomization: GranularRandomization,
    pub chord: GranularChord,
    pub warm_start: bool,
}

impl Default for GranularGenerator {
    fn default() -> Self {
        Self::from(&GeneratorBlock {
            name: GeneratorMode::GranularGenerator.name().to_owned(),
            granular_position: Ratio::new::<percent>(2.5),
            granular_grains: 4.0,
            ..GeneratorBlock::default()
        })
    }
}

impl From<&GeneratorBlock> for GranularGenerator {
    fn from(block: &GeneratorBlock) -> Self {
        Self {
            id: block.id,
            enabled: block.enabled,
            name: block.name.to_owned(),
            level: block.level,
            fine_tuning: block.fine_tuning,
            harmonic: block.harmonic,
            shift: block.shift,
            phase_offset: block.phase_offset,
            phase_jitter: block.phase_jitter,
            sample_contents: block.sample_contents.clone(),
            sample_name: block.sample_name.clone(),
            sample_path: block.sample_path.clone(),
            base_pitch: block.base_pitch,
            base_pitch_locked: block.base_pitch_locked,
            position: block.granular_position,
            direction: block.granular_direction,
            envelope: block.granular_envelope.clone(),
            align_phases: block.granular_align_phases,
            grains: block.granular_grains,
            grain_length: block.granular_grain_length,
            auto_grain_length: block.granular_auto_grain_length,
            spawn_rate_mode: block.granular_spawn_rate_mode,
            randomization: block.granular_randomization.clone(),
            chord: block.granular_chord.clone(),
            warm_start: block.granular_warm_start,
        }
    }
}

impl Generator for GranularGenerator {
    fn id(&self) -> Option<GeneratorId> {
        Some(self.id)
    }

    fn as_block(&self) -> GeneratorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn mode(&self) -> GeneratorMode {
        GeneratorMode::GranularGenerator
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_granular(&self) -> Option<&GranularGenerator> {
        self.downcast_ref::<GranularGenerator>()
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use music_note::midi;
    use uom::si::frequency::hertz;
    use uom::si::ratio::percent;
    use uom::si::ratio::ratio;
    use uom::si::time::millisecond;

    use crate::test::read_generator_preset;

    use super::*;

    #[test]
    fn auto_length_adjust() {
        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-auto_length_adjust_disabled-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert!(!generator.auto_grain_length);
    }

    /// The factory Chaotic Saw sample.
    #[test]
    fn chaotic_saw() {
        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-chaotic_saw-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.sample_name, Some("Chaotic Saw".to_owned()));
        assert_eq!(
            generator.sample_path,
            Some("factory/Grains/Additive/Chaotic Saw.flac".to_owned())
        );

        // Factory samples are not embedded in the preset.
        assert!(generator.sample_contents.is_empty());
    }

    #[test]
    fn chord() {
        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-chord-fifths-range3oct-pickup-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert!(generator.chord.enabled);
        assert_eq!(generator.chord.mode, GranularChordMode::Fifths);
        assert_eq!(generator.chord.range_octaves, 3.0);
        assert_eq!(generator.chord.picking_pattern, ChordPickingPattern::Up);

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-chord-pent_min-range8oct-pick_down-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert!(generator.chord.enabled);
        assert_eq!(generator.chord.mode, GranularChordMode::PentatonicMinor);
        assert_eq!(generator.chord.range_octaves, 8.0);
        assert_eq!(generator.chord.picking_pattern, ChordPickingPattern::Down);

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-chord-sus2-range0-pick_up_down-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert!(generator.chord.enabled);
        assert_eq!(generator.chord.mode, GranularChordMode::Sus2);
        assert_eq!(generator.chord.range_octaves, 0.0);
        assert_eq!(generator.chord.picking_pattern, ChordPickingPattern::UpDown);
    }

    #[test]
    fn default() {
        let generator = GranularGenerator::default();
        assert!(generator.enabled);
        assert_eq!(generator.name(), "Granular".to_owned());
        assert_eq!(generator.fine_tuning, 0.0);
        assert_eq!(generator.harmonic, 1.0);
        assert_eq!(generator.shift, Frequency::zero());
        assert_eq!(generator.phase_offset, Ratio::zero());
        assert_eq!(generator.phase_jitter, Ratio::zero());
        assert_eq!(generator.level.get::<percent>(), 100.0);
        assert!(generator.sample_contents.is_empty());
        assert!(generator.sample_name.is_none());
        assert!(generator.sample_path.is_none());
        assert_eq!(generator.base_pitch, midi!(C, 4).into_byte() as f32);
        assert!(!generator.base_pitch_locked);
        assert_relative_eq!(generator.position.get::<percent>(), 2.5);
        assert_eq!(generator.direction, GranularDirection::Start);
        assert_eq!(generator.spawn_rate_mode, GranularSpawnRateMode::Density);
        assert_eq!(generator.envelope, GranularEnvelope::default());
        assert!(!generator.align_phases);
        assert_eq!(generator.grains, 4.0);
        assert_relative_eq!(generator.grain_length.get::<millisecond>(), 250.0);
        assert!(generator.auto_grain_length);
        assert_eq!(generator.spawn_rate_mode, GranularSpawnRateMode::Density);
        assert_eq!(generator.randomization, GranularRandomization::default());
        assert_eq!(generator.chord, GranularChord::default());
        assert!(!generator.warm_start);
        assert!(generator.sample_contents.is_empty());
    }

    #[test]
    fn direction() {
        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-direction-midpoint-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.direction, GranularDirection::Midpoint);
    }

    #[test]
    fn init() {
        // for file in &["granular_generator-2.1.0.phaseplant"] {
        let file = "granular_generator-2.1.0.phaseplant";
        let preset = read_generator_preset("granular_generator", file).unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert!(generator.enabled);
        assert_eq!(generator.name(), "Granular".to_owned());
        assert_eq!(generator.fine_tuning, 0.0);
        assert_eq!(generator.harmonic, 1.0);
        assert_eq!(generator.shift, Frequency::zero());
        assert_eq!(generator.phase_offset, Ratio::zero());
        assert_eq!(generator.phase_jitter, Ratio::zero());
        assert_eq!(generator.level.get::<percent>(), 100.0);
        assert!(generator.sample_contents.is_empty());
        assert!(generator.sample_name.is_none());
        assert!(generator.sample_path.is_none());
        assert_eq!(generator.base_pitch, midi!(C, 4).into_byte() as f32);
        assert!(!generator.base_pitch_locked);
        assert_eq!(generator.position.get::<percent>(), 2.5);
        assert_eq!(generator.direction, GranularDirection::Start);
        assert_eq!(generator.spawn_rate_mode, GranularSpawnRateMode::Density);
        assert_eq!(generator.envelope, GranularEnvelope::default());
        assert!(!generator.align_phases);
        assert_eq!(generator.grains, 4.0);
        assert_relative_eq!(generator.grain_length.get::<millisecond>(), 250.0);
        assert!(generator.auto_grain_length);
        assert_eq!(generator.spawn_rate_mode, GranularSpawnRateMode::Density);
        assert_eq!(generator.randomization, GranularRandomization::default());
        assert_eq!(generator.chord, GranularChord::default());
        assert!(!generator.warm_start);
        // }
    }

    #[test]
    fn parts() {
        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-grains16-length100-pitch5-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert!(generator.enabled);
        assert_relative_eq!(generator.grains, 16.0);
        assert_relative_eq!(
            generator.grain_length.get::<millisecond>(),
            100.0,
            epsilon = 0.001
        );
        assert_relative_eq!(generator.fine_tuning, 5.0, epsilon = 0.001);

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-harmonic5-rate-reverse25-2.0.16.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.harmonic, 5.0);
        assert_eq!(generator.spawn_rate_mode, GranularSpawnRateMode::Rate);
        assert_eq!(generator.randomization.reverse.get::<percent>(), 25.0);

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-pan25-reverse15-chord-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_relative_eq!(generator.randomization.pan.get::<percent>(), 25.0);
        assert_relative_eq!(generator.randomization.reverse.get::<percent>(), 15.0);
        assert!(generator.chord.enabled);

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-phase_jitter15-warm_start-decay40-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert!(generator.warm_start);
        assert_relative_eq!(generator.phase_jitter.get::<ratio>(), 15.0 / 360.0);
        assert_relative_eq!(generator.envelope.decay_time.get::<percent>(), 40.0);

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-position60-disabled-2.1.0.phaseplant",
        )
        .unwrap();
        let group: &Group = preset.generator(0).unwrap();
        assert!(group.enabled);
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_relative_eq!(generator.position.get::<percent>(), 60.0, epsilon = 0.001);
        assert!(!generator.enabled);

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-randomize-pos10-timing20-pitch2-level50-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_relative_eq!(
            generator.randomization.position.get::<percent>(),
            10.0,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            generator.randomization.timing.get::<percent>(),
            20.0,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            generator.randomization.pitch.get::<hertz>(),
            2.0,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            generator.randomization.level.get::<percent>(),
            50.0,
            epsilon = 0.0001
        );

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-root_d5-align_phases-level75-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.base_pitch, midi!(D, 5).into_byte() as f32);
        assert!(!generator.base_pitch_locked);
        assert!(generator.align_phases);
        assert_eq!(generator.level.get::<percent>(), 75.0);

        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-shift125-phase_offset10-sync-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_relative_eq!(generator.shift.get::<hertz>(), 125.0, epsilon = 0.001);
        assert_relative_eq!(generator.phase_offset.get::<ratio>(), 10.0 / 360.0);
        assert_eq!(generator.spawn_rate_mode, GranularSpawnRateMode::Sync);
    }

    #[test]
    fn sample() {
        let preset = read_generator_preset(
            "granular_generator",
            "granular_generator-sample_custom-2.1.0.phaseplant",
        )
        .unwrap();
        let generator: &GranularGenerator = preset.generator(1).unwrap();
        assert_eq!(generator.sample_name, Some("sample".to_owned()));
        assert_eq!(generator.sample_path, Some("user/sample.wav".to_owned()));
        assert_eq!(generator.sample_contents.len(), 78186);
        assert_eq!(&generator.sample_contents[0..4], b"fLaC");
    }
}
