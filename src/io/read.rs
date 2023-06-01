use std::fs::File;
use std::io::prelude::*;
use std::io::{Cursor, Error, ErrorKind, Seek};
use std::mem::size_of;
use std::path::Path;
use std::str;

use byteorder::{LittleEndian, ReadBytesExt};
use log::{debug, trace, warn};
use serde::Deserialize;
use strum::IntoEnumIterator;
use uom::si::f32::{Frequency, Ratio, Time};
use uom::si::frequency::hertz;
use uom::si::ratio::{percent, ratio};
use uom::si::time::second;

use crate::effect::*;
use crate::generator::*;
use crate::io::generators::GeneratorBlock;
use crate::io::modulators::*;
use crate::io::MetadataJson;
use crate::modulation::{ModulationSource, ModulationTarget, MODULATIONS_MAX};
use crate::modulator::*;
use crate::text::TextOptionExt;
use crate::*;

/// Maximum length of a general purpose string. Helps to avoid corrupt files
/// from consuming larges amounts of resources.
const READ_STRING_LENGTH_MAX: u32 = 1024;

pub const MIN_SUPPORTED_RELEASE: PhasePlantRelease = PhasePlantRelease::V1_6_9;

/// Make reading the Phase Plant format less verbose. Phase Plant version 1
/// and 2 presets are supported.
pub struct PhasePlantReader<T: Read + Seek> {
    inner: T,
    pub(crate) format_version: Version<u32>,
}

impl<T: Read + Seek> PhasePlantReader<T> {
    pub fn new(inner: T) -> Result<Self, Error> {
        let mut reader = Self {
            inner,
            format_version: Version::new(0, 0, 0, 0),
        };

        let format_major = reader.read_u32()?;
        let format_patch = reader.read_u32()?;
        let format_minor = reader.read_u32()?;
        reader.format_version = Version::new(format_major, format_minor, format_patch, 0);
        Ok(reader)
    }

    pub fn is_release_at_least(&self, version: PhasePlantRelease) -> bool {
        self.format_version.is_at_least(&version.format_version())
    }

    /// If the version of Phase Plant is version 2.0 or after.
    pub fn is_version_at_least_2_0(&self) -> bool {
        self.is_release_at_least(PhasePlantRelease::V2_0_0)
    }

    /// If the version of Phase Plant is version 2.1 or after.
    pub fn is_version_at_least_2_1(&self) -> bool {
        self.is_release_at_least(PhasePlantRelease::V2_1_0)
    }

    pub(crate) fn stream_position(&mut self) -> Result<u64, Error> {
        self.inner.stream_position()
    }

    /// The stream position as infallible text. Will return `"<unknown>" `if
    /// the position cannot be determined.
    pub(crate) fn pos(&mut self) -> String {
        self.inner
            .stream_position()
            .map(|pos| pos.to_string())
            .unwrap_or_else(|_| "<unknown>".to_owned())
    }

    /// Read the next u32 and return an error with the given name for the kind
    /// of value if the value does not match.
    pub(crate) fn expect_u8(&mut self, expect: u8, name: &str) -> Result<(), Error> {
        match self.inner.read_u8()? {
            expected if expected == expect => Ok(()),
            unexpected => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Value {unexpected} ({unexpected:#x}) is not the excepted value of {expect:#x} for {name} at position {}",
                    self.stream_position()? - 1
                ),
            ))
        }
    }

    pub(crate) fn expect_bool32(&mut self, expect: bool, name: &str) -> Result<(), Error> {
        match self.read_u32()? {
            0 => Ok(()),
            1 => Ok(()),
            unexpected => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Value {unexpected} ({unexpected:#x}) is not the excepted value of {expect} for {name} at position {}",
                    self.stream_position()? - 4
                ),
            ))
        }
    }

    pub(crate) fn expect_f32(&mut self, expect: f32, name: &str) -> Result<(), Error> {
        match self.read_f32()? {
            expected if expected == expect => Ok(()),
            unexpected => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Value {unexpected} is not the excepted value of {expect} for {name} at position {}",
                    self.stream_position()? - 4
                ),
            ))
        }
    }

    /// Read the next u32 and return an error with the given name for the kind
    /// of value if the value does not match.
    pub(crate) fn expect_u32(&mut self, expect: u32, name: &str) -> Result<(), Error> {
        match self.read_u32()? {
            expected if expected == expect => Ok(()),
            unexpected =>
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Value {unexpected} ({unexpected:#x}) is not the excepted value of {expect:#x} for {name} at position {}",
                            self.stream_position()? - 4
                    ),
                ))
        }
    }

    pub(crate) fn read_bool8(&mut self) -> Result<bool, Error> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            n => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Value {n:#x} is not a 8-bit boolean at position {}",
                    self.stream_position()? - 4
                ),
            )),
        }
    }

    pub(crate) fn read_bool32(&mut self) -> Result<bool, Error> {
        match self.read_u32()? {
            0 => Ok(false),
            1 => Ok(true),
            n => Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Value {n:#x} is not a 32-bit boolean at position {}",
                    self.stream_position()? - 4
                ),
            )),
        }
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, Error> {
        self.inner.read_u8()
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16, Error> {
        self.inner.read_u16::<LittleEndian>()
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32, Error> {
        self.inner.read_u32::<LittleEndian>()
    }

    pub(crate) fn read_f32(&mut self) -> Result<f32, Error> {
        self.inner.read_f32::<LittleEndian>()
    }

    /// Read the length of the string then the string. An error is created if
    /// the string exceeds [`READ_STRING_LENGTH_MAX`].
    pub(crate) fn read_string_and_length(&mut self) -> Result<Option<String>, Error> {
        let len = self.read_u32()?;
        let string_pos = self.stream_position()?;
        if len > READ_STRING_LENGTH_MAX {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Text length of {len} ({len:#x}) exceeds {READ_STRING_LENGTH_MAX} characters at position {}",
                    self.stream_position()? as usize - size_of::<u32>()
                ),
            ))
        } else if len == 0 {
            Ok(None)
        } else {
            let mut buffer = vec![0u8; len as usize];
            self.inner.read_exact(&mut buffer)?;
            match str::from_utf8(&buffer) {
                Ok(text) => Ok(Some(text.to_string())),
                Err(err) => Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Cannot convert text to UTF-8 at position {string_pos}: {err}"),
                )),
            }
        }
    }

    pub(crate) fn read_metadata(&mut self) -> Result<Metadata, Error> {
        // Length includes the byte before the JSON actually starts.
        let metadata_length = self.read_u32()? as usize;
        if metadata_length > METADATA_LENGTH_MAX {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Metadata length of {metadata_length} is too large"),
            ));
        }

        // Unknown byte before the metadata starts.
        let unknown_m1 = self.read_u8()?;
        if unknown_m1 != 0 {
            warn!("unknown m1: unexpected value {unknown_m1}");
        }

        // JSON metadata.  Read using a buffer instead of directly from the
        // stream so the length of the metadata section specified in the file
        // is used.
        let mut json_buffer = vec![0_u8; metadata_length - 1];
        self.read_exact(&mut json_buffer)?;

        let mut deserializer = serde_json::Deserializer::from_reader(Cursor::new(json_buffer));
        let metadata_json = MetadataJson::deserialize(&mut deserializer)?;
        let author = metadata_json.author.trim().empty_to_none();
        let description = metadata_json.description.trim().empty_to_none();
        Ok(Metadata {
            name: None,
            author,
            description,
            category: None,
        })
    }

    /// Read a multi-part path.
    pub(crate) fn read_path(&mut self) -> Result<Vec<String>, Error> {
        let component_count = self.read_u32()? as usize;
        if component_count > PATH_COMPONENT_COUNT_MAX {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Path component count of {component_count} exceeds {PATH_COMPONENT_COUNT_MAX} at position {}",
                    self.pos()
                ),
            ));
        }

        let mut path = Vec::with_capacity(component_count);
        for _ in 0..component_count {
            path.push(self.read_string_and_length()?.unwrap_or_default());
        }
        Ok(path)
    }

    pub(crate) fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.inner.read_exact(buf)
    }

    pub(crate) fn read_block_header(&mut self) -> Result<DataBlockHeader, Error> {
        let length = self.read_u32()? as usize;
        if length == 0 {
            let message = format!(
                "Data block header had zero length at position {}",
                self.stream_position()? as usize - size_of::<u32>()
            );
            Err(Error::new(ErrorKind::InvalidData, message))
        } else {
            let is_used = self.read_bool8()?;
            let version_opt = if is_used {
                Some(self.read_u32()?)
            } else {
                None
            };
            let length = length - if is_used { 5 } else { 1 };
            Ok(DataBlockHeader::new(length, is_used, version_opt))
        }
    }

    pub(crate) fn read_envelope(&mut self) -> Result<Envelope, Error> {
        let delay = Time::new::<second>(self.read_f32()?);
        let attack = Time::new::<second>(self.read_f32()?);
        let attack_curve = self.read_f32()?;
        let hold = Time::new::<second>(self.read_f32()?);
        let decay = Time::new::<second>(self.read_f32()?);
        let decay_falloff = self.read_f32()?;
        let sustain = Ratio::new::<percent>(self.read_f32()?);
        let release = Time::new::<second>(self.read_f32()?);
        let release_falloff = self.read_f32()?;
        Ok(Envelope {
            delay,
            attack,
            attack_curve,
            hold,
            decay,
            decay_falloff,
            sustain,
            release,
            release_falloff,
        })
    }

    /// Prefer reading the data and comparing to expected values instead
    /// of blindly skipping over parts of the file. This will help ensure the
    /// understanding of the preset format is correct.
    pub(crate) fn skip(&mut self, bytes: i64) -> Result<u64, Error> {
        // BufReader has seek_relative
        self.inner.seek(std::io::SeekFrom::Current(bytes))
    }
}

impl Preset {
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Preset, Error> {
        let mut file = File::open(path.as_ref())?;
        let name_opt = path
            .as_ref()
            .file_stem()
            .map(|os_str| os_str.to_string_lossy());
        let name_str = name_opt.map(|name| name.to_string());
        Self::read(&mut file, name_str)
    }

    pub fn read<R: Read + Seek>(reader: &mut R, name: Option<String>) -> Result<Preset, Error> {
        let mut reader = PhasePlantReader::new(reader)?;

        //
        // Header
        //

        debug!("Preset format version {}", reader.format_version);
        if !PhasePlantRelease::is_likely_format_version(&reader.format_version) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Not a Phase Plant preset",
            ));
        } else if !reader
            .format_version
            .is_at_least(&MIN_SUPPORTED_RELEASE.format_version())
        {
            let message = format!(
                "Version {} presets are not supported",
                reader.format_version
            );
            return Err(Error::new(ErrorKind::InvalidData, message));
        }

        //
        // Metadata
        //

        let mut metadata = reader.read_metadata()?;
        if metadata.name.is_none() {
            metadata.name = name;
        }

        reader.expect_bool32(true, "unknown_read_1")?;

        //
        // Modulation
        //

        trace!("modulation: start pos {}", reader.pos());
        let modulation_count = reader.read_u32()? as usize;
        if modulation_count > MODULATIONS_MAX {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Modulation count of {modulation_count} is greater than {MODULATIONS_MAX}"),
            ));
        }

        let mut modulations = Vec::with_capacity(modulation_count);
        for modulation_index in 0..modulation_count {
            trace!("modulation: index {modulation_index}, pos {}", reader.pos());

            let source_id = reader.read_u32()?;
            let destination_id = reader.read_u32()?;
            let amount = Ratio::new::<percent>(reader.read_f32()? * 100.0);
            trace!("modulation: source {source_id:#x}, destination = {destination_id:#x}, percent = {amount:?}");

            let source: ModulationSource = source_id.into();
            if let ModulationSource::Unknown { .. } = source {
                warn!("Unknown modulation source: {source}");
            }

            let destination: ModulationTarget = destination_id.into();
            if let ModulationTarget::Unknown { .. } = destination {
                warn!("Unknown modulation destination {destination}");
            }

            // The curve and enabled state are later in the preset.
            modulations.push(Modulation::new(source, destination, amount))
        }

        // Skip over the unused modulations.
        let unused_modulation_count = MODULATIONS_MAX - modulation_count;
        trace!("modulation: skipping {unused_modulation_count} unused modulations");
        reader.skip((12 * unused_modulation_count) as i64)?;
        trace!("modulation: end pos {}", reader.pos());

        reader.expect_u32(1, "unknown_m3")?;

        //
        // Lanes
        //

        let mut lanes = Vec::with_capacity(LANE_COUNT);
        for lane_index in 0..LANE_COUNT {
            trace!("lane {}: pos {}", lane_index, reader.pos());
            let enabled = reader.read_bool32()?;
            let gain = reader.read_f32()?;
            let mix = Ratio::new::<ratio>(reader.read_f32()?);

            let dest_id = reader.read_u32()?;
            trace!(
                "lane: dest id {dest_id}, enabled {enabled}, pos {}",
                reader.pos()
            );
            let destination = LaneDestination::from_id(dest_id)?;

            // Snapins, poly, mute and solo are later in the preset.
            lanes.push(Lane {
                enabled,
                snapins: Vec::new(),
                destination,
                poly_count: 0,
                mute: false,
                solo: false,
                gain,
                mix,
            });
        }

        //
        // Macro values
        //

        // The macros names are later in the file in the string pool.
        trace!("macro controls: pos {}", reader.pos());
        let mut macro_controls = Vec::with_capacity(MacroControl::COUNT);
        for _ in 0..MacroControl::COUNT {
            macro_controls.push(MacroControl {
                name: Default::default(),
                value: reader.read_f32()?,
                polarity: OutputRange::Unipolar,
            });
        }
        trace!(
            "macro controls: values {:?}",
            macro_controls
                .iter()
                .map(|ctrl| ctrl.value)
                .collect::<Vec<_>>()
        );

        //
        // Modulators
        //

        trace!("modulators: pos {}", reader.pos());
        let mut mod_blocks = Vec::with_capacity(MODULATORS_MAX);
        for _ in 0..MODULATORS_MAX {
            let mode = ModulatorMode::from_id(reader.read_u32()?)?;

            let id = reader.read_u32()?;
            if id as usize > MODULATORS_MAX {
                let msg = format!("Modulator ID is greater than {MODULATORS_MAX}");
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
            let id = id as ModulatorId;

            let enabled = reader.read_bool32()?;
            let start_pos = reader.stream_position()?;
            if !mode.is_blank() {
                trace!("modulator: mode {mode}, id {id}, enabled {enabled}, pos {start_pos}");
            }

            // Min and max modulators
            let input_a = reader.read_f32()?;
            let input_b = reader.read_f32()?;
            if !mode.is_blank() {
                trace!("modulator: input a {input_a}, input b {input_b}");
            }

            // LFO
            let depth = Ratio::new::<ratio>(reader.read_f32()?);

            // Retrigger was replaced by NoteTriggerMode in Phase Plant 2.
            let retrigger = reader.read_bool32()?;
            let note_trigger_mode = if retrigger {
                NoteTriggerMode::Auto
            } else {
                NoteTriggerMode::Never
            };

            let output_range = OutputRange::from_id(reader.read_u32()?)?;

            let rate = Rate {
                frequency: Frequency::new::<hertz>(reader.read_f32()?),
                numerator: reader.read_u32()?,
                denominator: NoteValue::from_id(reader.read_u32()?)?,
                sync: reader.read_bool32()?,
            };

            let envelope = reader.read_envelope()?;
            if !mode.is_blank() {
                trace!("modulator: envelope {envelope:?}");
            }

            let phase_offset = Ratio::new::<ratio>(reader.read_f32()?);

            // One shot was replaced by loop mode Phase Plant 2.
            let one_shot = reader.read_bool32()?;
            let loop_mode = if one_shot {
                LoopMode::Off
            } else {
                LoopMode::Infinite
            };

            let multiplier = reader.read_f32()?;

            reader.expect_f32(1.0, "modulator: unknown_1")?;

            let smooth = reader.read_f32()?;
            let jitter = reader.read_f32()?;
            let chaos = reader.read_f32()?;

            let block = ModulatorBlock {
                mode,
                id,
                enabled,
                output_range,
                input_a,
                input_b,
                multiplier,
                depth,
                rate,
                phase_offset,
                loop_mode,
                note_trigger_mode,
                envelope,
                random_jitter: jitter,
                random_smooth: smooth,
                random_chaos: chaos,
                ..Default::default()
            };
            mod_blocks.push(block);

            let remaining =
                MODULATOR_BLOCK_SIZE as i64 - (reader.stream_position()? - start_pos) as i64;
            if remaining != 0 {
                let msg = format!("Modulator block had {remaining} bytes remaining");
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
        }

        if !reader.is_release_at_least(PhasePlantRelease::V1_7_0) {
            reader.expect_u32(0, "early_version_extra_1")?;
        }

        let mod_wheel_value = Ratio::new::<ratio>(reader.read_f32()?);
        let master_pitch = reader.read_f32()?;
        let polyphony = reader.read_u32()?;
        trace!("modulator: mod wheel value {mod_wheel_value:?}, master pitch {master_pitch}, polyphony {polyphony}");

        let retrigger_enabled = reader.read_bool32()?;

        // Glide
        let glide_enabled = reader.read_bool32()?;
        let glide_legato = reader.read_bool32()?;
        let glide_time = reader.read_f32()?;
        trace!("glide: enabled {glide_enabled}, legato {glide_legato}, time {glide_time}");

        //
        // Generators
        //

        let mut gen_blocks = Vec::with_capacity(GENERATORS_MAX as usize);
        for gen_index in 0..GENERATORS_MAX {
            let start_pos = reader.stream_position()?;

            let mode = GeneratorMode::from_id(reader.read_u32()?)?;

            let id = reader.read_u32()?;
            if !mode.is_blank() {
                trace!("generator: mode {mode}, index {gen_index}, id {id}, start pos {start_pos}");
            }
            if id > GENERATORS_MAX as u32 {
                let msg = format!("Generator {gen_index} has an ID of {id}, which is greater than {GENERATORS_MAX}");
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
            let id = id as GeneratorId;

            let enabled = reader.read_bool32()?;
            let fine_tuning = reader.read_f32()?;
            let harmonic = reader.read_f32()?;
            let shift = Frequency::new::<hertz>(reader.read_f32()?);
            let level = reader.read_f32()?;
            let phase_offset = Ratio::new::<ratio>(reader.read_f32()?);
            let phase_jitter = Ratio::new::<ratio>(reader.read_f32()?);
            if !mode.is_blank() {
                trace!("generator: unison start pos {}", reader.pos());
            }
            let unison = Unison {
                // Other properties are later in the file
                voices: reader.read_u32()?,
                detune: reader.read_f32()?,
                spread: reader.read_f32()?,
                blend: reader.read_f32()?,
                ..Default::default()
            };
            if !mode.is_blank() {
                trace!(
                    "generator: unison end pos {}, unison {unison:?}",
                    reader.pos()
                );
            }

            // Sample player
            let base_pitch = reader.read_f32()?;
            let offset_position = reader.read_f32()?;
            let loop_mode = LoopMode::from_id(reader.read_u32()?)?;
            let loop_start_position = reader.read_f32()?;
            let loop_length = reader.read_f32()?;
            let crossfade_amount = reader.read_f32()?;
            if !mode.is_blank() {
                trace!("generator: wavetable frame pos {}", reader.pos());
            }
            let wavetable_frame = reader.read_f32()?;
            let band_limit = reader.read_f32()?;
            if !mode.is_blank() {
                trace!(
                    "generator: band_limit {band_limit}, pos {}",
                    reader.stream_position()? - 4
                );
            }

            let analog_waveform = AnalogWaveform::from_id(reader.read_u32()?)?;

            let sync = reader.read_f32()?;
            let pulse_width = reader.read_f32()?;
            let seed_random = reader.read_bool32()?;
            let noise_slope = reader.read_f32()?;
            let stereo = reader.read_f32()?;

            let noise_waveform = NoiseWaveform::from_id(reader.read_u32()?)?;

            let filter_mode = FilterMode::from_id(reader.read_u32()?)?;

            let filter_effect = Filter {
                filter_mode,
                cutoff_frequency: reader.read_f32()?,
                q: reader.read_f32()?,
                gain: Decibels::from_linear(reader.read_f32()?),
                ..Default::default()
            };

            let distortion_mode_id = reader.read_u32()?;
            let distortion_mode =
                match DistortionMode::iter().find(|mode| *mode as u32 == distortion_mode_id) {
                    Some(mode) => mode,
                    None => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Distortion mode {distortion_mode_id} not recognized"),
                        ));
                    }
                };

            let mut distortion_effect = Distortion::new();
            distortion_effect.mode = distortion_mode;
            distortion_effect.drive = Decibels::from_linear(reader.read_f32()?);
            distortion_effect.bias = reader.read_f32()?;
            distortion_effect.dynamics = 0.0; // Not in the Phase Plant interface
            distortion_effect.spread = 0.0;
            distortion_effect.mix = Ratio::new::<ratio>(reader.read_f32()?);

            let invert = reader.read_bool32()?;
            let mix_level = reader.read_f32()?;
            let output_gain = Decibels::from_linear(reader.read_f32()?);
            let pan = Ratio::new::<ratio>(reader.read_f32()?);

            let output_destination = OutputDestination::from_id(reader.read_u32()?)?;
            if !mode.is_blank() {
                trace!(
                    "generator: output destination {output_destination}, envelope pos {}",
                    reader.pos()
                );
            }

            let envelope = reader.read_envelope()?;

            let block = GeneratorBlock {
                id,
                mode,
                enabled,
                name: mode.name().to_owned(),
                fine_tuning,
                harmonic,
                shift,
                phase_offset,
                phase_jitter,
                level,
                unison,
                analog_waveform,
                sync_multiplier: sync,
                pulse_width,
                base_pitch,
                offset_position,
                loop_start_position,
                loop_length,
                sample_loop_mode: loop_mode,
                crossfade_amount,
                invert,
                filter_effect,
                distortion_effect,
                band_limit,
                mix_level,
                noise_waveform,
                noise_slope,
                stereo,
                seed_random,
                pan,
                output_gain,
                output_destination,
                envelope,
                wavetable_frame,
                ..Default::default()
            };
            gen_blocks.push(block);

            let remaining =
                GeneratorBlock::SIZE as i64 - (reader.stream_position()? - start_pos) as i64;
            if remaining != 0 {
                let msg = format!(
                    "Generator {gen_index} starting at {start_pos} had {remaining} bytes remaining"
                );
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
        }

        //
        // Unison
        //

        trace!("global unison: pos {}", reader.pos());
        let unison_voices = reader.read_u32()?;
        if !(1..=Unison::VOICES_MAX).contains(&unison_voices) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unexpected number of unison voices ({})", unison_voices),
            ));
        }
        let unison_detune = reader.read_f32()?;
        let unison_spread = reader.read_f32()?;
        let unison_blend = reader.read_f32()?;

        let master_gain = reader.read_f32()?;

        //
        // Lanes
        //

        trace!("lanes: pos {}", reader.stream_position()?);
        for (lane_index, lane) in lanes.iter_mut().enumerate() {
            // How many lanes from left to right are poly.
            lane.poly_count = reader.read_u8()?;

            reader.expect_u8(0, "lane_unknown_1")?;

            lane.mute = reader.read_bool8()?;

            reader.expect_u8(0, "lane_unknown_2")?;

            lane.solo = reader.read_bool32()?;

            // The last lane has less padding
            if lane_index < LANE_COUNT - 1 {
                reader.expect_u8(0, "lane_unknown_3")?;
                reader.expect_u8(0, "lane_unknown_4")?;
            }
        }

        //
        // Minimized
        //

        trace!("modulator: minimized pos {}", reader.pos());
        for mod_index in 0..MODULATORS_MAX {
            let minimized = reader.read_bool32()?;
            if mod_index < mod_blocks.len() {
                mod_blocks[mod_index].minimized = minimized;
            }
        }

        trace!("generator: minimized pos {}", reader.pos());
        for block in &mut gen_blocks {
            block.minimized = reader.read_bool32()?;
        }

        reader.expect_u32(0, "unknown_g1")?;

        // FIXME: Find version where it's actually added.
        if reader.is_release_at_least(PhasePlantRelease::V1_7_3) {
            for _ in 0..32 {
                // FIXME: LFO grid and snapping?
                reader.skip(12)?;
                // reader.expect_u32(8, "block_q_unknown_1")?;
                // reader.expect_u32(8, "block_q_unknown_1")?;
                // reader.expect_u32(1, "block_q_unknown_1")?;
            }
        }

        //
        // Locks
        //

        if reader.is_release_at_least(PhasePlantRelease::V1_7_4) {
            trace!("generator: sample player locks pos {}", reader.pos());
            for block in &mut gen_blocks {
                block.base_pitch_locked = reader.read_bool32()?;
                block.offset_locked = reader.read_bool32()?;
                block.loop_locked = reader.read_bool32()?;
            }
        }

        // FIXME: GUESS AT VERSION
        if reader.is_release_at_least(PhasePlantRelease::V1_8_0) {
            for mod_block in &mut mod_blocks {
                mod_block.shape_edited = reader.read_bool32()?;
            }
        }

        //
        // Filter slope
        //

        if reader.is_release_at_least(PhasePlantRelease::V1_8_0) {
            trace!("generator: filter effect slopes pos {}", reader.pos());
            for block in &mut gen_blocks {
                block.filter_effect.slope = reader.read_u32()?;
            }
        }

        // Unknown
        // FIXME: GUESS AT VERSION
        if reader.is_release_at_least(PhasePlantRelease::V1_8_0) {
            trace!("unknown blocks YY: pos {}", reader.pos());
            reader.skip(128)?;
        }

        // Note modulator
        trace!("modulator: note modulator: pos {}", reader.pos());
        if reader.is_release_at_least(PhasePlantRelease::V1_8_5) {
            for mod_block in &mut mod_blocks {
                mod_block.root_note = reader.read_u32()?;
                mod_block.note_range = reader.read_u32()?;
                if !mod_block.mode.is_blank()
                    && mod_block.note_range < *NoteModulator::NOTE_RANGE.start() as u32
                    || mod_block.note_range > *NoteModulator::NOTE_RANGE.end() as u32
                {
                    let msg = format!(
                        "Note range {} for {} is outside the valid range of {} to {}",
                        mod_block.note_range,
                        mod_block.mode,
                        NoteModulator::NOTE_RANGE.start(),
                        NoteModulator::NOTE_RANGE.end()
                    );
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
            }
        }

        //
        // Unison
        //

        trace!("generator: unison mode pos {}", reader.pos());
        let unison = if reader.is_release_at_least(PhasePlantRelease::V1_8_5) {
            for block in &mut gen_blocks {
                block.unison.mode = UnisonMode::from_id(reader.read_u32()?)?;
            }

            let mode = UnisonMode::from_id(reader.read_u32()?)?;

            for block in &mut gen_blocks {
                block.unison.bias = reader.read_f32()?;
            }

            let bias = reader.read_f32()?;

            for block in &mut gen_blocks {
                block.unison.enabled = reader.read_bool32()?;
            }

            let enabled = reader.read_bool32()?;

            Unison {
                enabled,
                voices: unison_voices,
                mode,
                detune: unison_detune,
                spread: unison_spread,
                blend: unison_blend,
                bias,
            }
        } else {
            Unison::default()
        };
        debug!("global unison: {:?}", unison);

        //
        // Loop enabled
        //

        trace!("loop enabled: pos {}", reader.pos());
        if reader.is_release_at_least(PhasePlantRelease::V1_8_5) {
            for block in &mut gen_blocks {
                block.loop_enabled = reader.read_bool32()?;
            }
        }

        if reader.is_version_at_least_2_0() {
            trace!("modulation: curves pos {}", reader.pos());
            for modulation in &mut modulations {
                modulation.curve = Ratio::new::<percent>(reader.read_f32()?);
                modulation.enabled = reader.read_bool32()?;
            }
            for _ in 0..(MODULATIONS_MAX - modulations.len()) {
                reader.expect_f32(0.0, "modulation_curve")?;
                reader.expect_bool32(true, "modulation_enabled")?;
            }

            trace!("macro controls: polarities pos {}", reader.pos());
            for macro_control in &mut macro_controls {
                macro_control.polarity = OutputRange::from_id(reader.read_u32()?)?;
            }

            for mod_block in &mut mod_blocks {
                mod_block.gain = Decibels::from_linear(reader.read_f32()?);
                mod_block.group_id = reader.read_u32()?;
                mod_block.trigger_threshold = reader.read_f32()?;
                mod_block.note_trigger_mode = NoteTriggerMode::from_id(reader.read_u32()?)?;
                reader.expect_u32(0, "block_d_unknown_2")?;
                mod_block.metering_mode = MeteringMode::from_id(reader.read_u32()?)?;

                // Pitch Tracker
                mod_block.pitch_tracker_lowest = reader.read_u32()?;
                mod_block.pitch_tracker_highest = reader.read_u32()?;
                mod_block.pitch_tracker_sensitivity = Ratio::new::<ratio>(reader.read_f32()?);
                mod_block.pitch_tracker_root = reader.read_u32()?;

                let controller_slot_id = reader.read_u32()?;
                if controller_slot_id != 0xFFFFFFFF {
                    mod_block.controller_slot = Some(controller_slot_id);
                }

                mod_block.velocity_trigger_mode = VelocityTriggerMode::from_id(reader.read_u32()?)?;
            }

            for mod_block in &mut mod_blocks {
                mod_block.lfo_table_frame = reader.read_f32()?;
            }

            trace!("version 2.0: block F: pos {}", reader.pos());
            for mod_block in &mut mod_blocks {
                reader.expect_f32(1.0, "block_f_unknown_1")?;
                mod_block.loop_mode = LoopMode::from_id(reader.read_u32()?)?;
                reader.expect_u32(0, "block_f_unknown_2")?;

                // These two values are not known. They are general 0.0 and
                // 1.0 respectively.  They have sometimes changed when
                // adding a third point to the Slope preset in the Curve
                // modulator. They often add up to 1.0 but not always.
                let _unknown_block_f_3 = reader.read_f32()?;
                let _unknown_block_f_4 = reader.read_f32()?;
            }

            trace!("curve_output: block g: pos {}", reader.pos());
            for gen in &mut gen_blocks {
                gen.curve_edited = reader.read_bool32()?;
                let _curve_block_unknown_1 = reader.read_bool32()?;
                reader.expect_f32(1.0, "block_g3_3")?;
                gen.rate.frequency = Frequency::new::<hertz>(reader.read_f32()?);
                gen.rate.numerator = reader.read_u32()?;
                gen.rate.denominator = NoteValue::from_id(reader.read_u32()?)?;
                gen.rate.sync = reader.read_bool32()?;
                gen.curve_loop_mode = LoopMode::from_id(reader.read_u32()?)?;
                gen.curve_loop_start = Ratio::new::<ratio>(reader.read_f32()?);
                gen.curve_loop_length = Ratio::new::<ratio>(reader.read_f32()?);
                gen.settings_locked = reader.read_bool32()?;
            }

            for mod_block in &mut mod_blocks {
                mod_block.curve_time = Time::new::<second>(reader.read_f32()?);

                // The reciprocal of the rate frequency for the LFO Table
                // modulator.
                let _lfo_table_time = Time::new::<second>(reader.read_f32()?);
            }

            for gen in &mut gen_blocks {
                gen.curve_length = Time::new::<second>(reader.read_f32()?);
            }

            trace!("version 2.0: block J: pos {}", reader.pos());
            for mod_block in &mut mod_blocks {
                reader.expect_u32(0, "block_j_unknown_1")?;
                mod_block.voice_mode = VoiceMode::from_id(reader.read_u32()?)?;
                reader.expect_u32(0, "block_j_unknown_3")?;
            }

            for gen in &mut gen_blocks {
                gen.output_enabled = reader.read_bool32()?;
            }
        }

        // Slew limiter.
        if reader.is_release_at_least(PhasePlantRelease::V2_0_12) {
            for mod_block in &mut mod_blocks {
                mod_block.slew_limiter_attack = Time::new::<second>(reader.read_f32()?);
                mod_block.slew_limiter_decay = Time::new::<second>(reader.read_f32()?);
            }
        }
        if reader.is_release_at_least(PhasePlantRelease::V2_0_13) {
            for mod_block in &mut mod_blocks {
                mod_block.slew_limiter_linked = reader.read_bool32()?;
            }
        }

        // Granular generator
        if reader.is_version_at_least_2_1() {
            trace!("granular: pos {}", reader.pos());
            for gen in &mut gen_blocks {
                let start_pos = reader.stream_position()?;
                gen.granular_position = Ratio::new::<ratio>(reader.read_f32()?);
                gen.granular_direction = GranularDirection::from_id(reader.read_u32()?)?;
                gen.granular_grains = reader.read_f32()?;

                // Randomization
                gen.granular_randomization.position = Ratio::new::<ratio>(reader.read_f32()?);
                gen.granular_randomization.timing = Ratio::new::<ratio>(reader.read_f32()?);
                gen.granular_randomization.pitch = Frequency::new::<hertz>(reader.read_f32()?);
                gen.granular_randomization.pan = Ratio::new::<ratio>(reader.read_f32()?);
                gen.granular_randomization.level = Ratio::new::<ratio>(reader.read_f32()?);
                gen.granular_randomization.reverse = Ratio::new::<ratio>(reader.read_f32()?);

                gen.granular_align_phases = reader.read_bool32()?;
                gen.granular_warm_start = reader.read_bool32()?;
                gen.granular_auto_grain_length = reader.read_bool32()?;

                gen.granular_envelope = GranularEnvelope {
                    attack_time: Ratio::new::<ratio>(reader.read_f32()?),
                    attack_curve: reader.read_f32()?,
                    decay_time: Ratio::new::<ratio>(reader.read_f32()?),
                    decay_curve: reader.read_f32()?,
                };

                gen.granular_grain_length = Time::new::<second>(reader.read_f32()?);
                gen.granular_chord.enabled = reader.read_bool32()?;
                gen.granular_chord.range_octaves = reader.read_f32()?;
                gen.granular_chord.mode = GranularChordMode::from_id(reader.read_u32()?)?;

                let read_length = reader.stream_position()? - start_pos;
                if read_length != 80 {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Granular block read {read_length} bytes instead of 80"),
                    ));
                }
            }

            trace!("block X (version 2.1): pos {}", reader.pos());
            for gen_block in &mut gen_blocks {
                reader.expect_f32(10.0, "block_x_unknown1")?;
                reader.expect_u32(4, "block_x_unknown2")?;
                reader.expect_u32(4, "block_x_unknown3")?;
                gen_block.granular_spawn_rate_mode =
                    GranularSpawnRateMode::from_id(reader.read_u32()?)?;
                gen_block.granular_chord.picking_pattern =
                    ChordPickingPattern::from_id(reader.read_u32()?)?;
            }
        }

        //
        // Lanes
        //

        trace!("lane: start of lanes pos {}", reader.pos());
        for lane in &mut lanes {
            let snapin_count = reader.read_u32()?;
            for snapin_index in 0..snapin_count {
                trace!("lane snapin: index {snapin_index}, pos {}", reader.pos());

                // Read the four characters for the name in the right order.
                let mut effect_id_bytes = [0_u8; 4];
                reader.read_exact(&mut effect_id_bytes)?;
                let effect_id = u32::from_be_bytes(effect_id_bytes);
                let effect_mode = match EffectMode::from_repr(effect_id) {
                    Some(mode) => mode,
                    None => {
                        let effect_id_bytes = u32::to_le_bytes(effect_id);
                        let effect_id = String::from_utf8_lossy(&effect_id_bytes).into_owned();
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            format!("Unknown effect mode \"{effect_id}\""),
                        ));
                    }
                };

                // 0x00MMmmPP = Major.Minor.Patch where each number is a single
                // byte. The Group effect will have a host version of 0.0.0-0,
                // perhaps because it doesn't have an audio path.
                let version_patch = reader.read_u8()?;
                let version_minor = reader.read_u8()?;
                let version_major = reader.read_u8()?;
                let version_extra = reader.read_u8()?;
                let host_version =
                    Version::new(version_major, version_minor, version_patch, version_extra);
                trace!(
                    "lane: effect mode '{effect_mode}', host version {host_version}, position pos {}",
                    reader.pos()
                );

                let name_opt = reader.read_string_and_length()?;
                let name_desc = name_opt
                    .clone()
                    .unwrap_or_else(|| String::from("<unknown>"));

                let position = reader.read_u16()?;
                if position == 0 {
                    let msg = format!("Invalid position {position} for snapin {name_desc}");
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }

                let effect_length = reader.read_u32()?;
                let effect_start_pos = reader.stream_position()?;
                let slot_format_major = reader.read_u32()?;
                debug!("lane snapin: slot format {slot_format_major}, host version {host_version}, effect length {effect_length}, start location {effect_start_pos}");
                let effect_version;
                let effect_read_return = if effect_mode.is_host() {
                    if slot_format_major == 1 {
                        let _header_length = reader.read_u32()?;
                        let _format_major = reader.read_u32()?;
                    }

                    effect_version = reader.read_u32()?;
                    let format_version_major = reader.read_u32()?;
                    trace!("lane snapin: snapin host version {effect_version}, format version major {format_version_major}");
                    let metadata = reader.read_metadata()?;

                    let mut effect_read_return;
                    effect_read_return = effect_mode.read_effect(&mut reader, effect_version)?;

                    let effect_remaining = effect_length as i64
                        - (reader.stream_position()? - effect_start_pos) as i64;
                    if effect_remaining != 0 {
                        let msg = format!("Snapin host {} version {effect_version} had {effect_remaining} bytes remaining", effect_mode.name());
                        return Err(Error::new(ErrorKind::InvalidData, msg));
                    }
                    effect_read_return.metadata = metadata;
                    effect_read_return
                } else {
                    if slot_format_major == 1 {
                        // FIXME: IF this is never hit then move effect_version up level.
                        panic!("Slot format major 1 not supported for non-host effects");
                    }
                    effect_version = reader.read_u32()?;
                    trace!("lane snapin: effect version {effect_version}, host version {host_version}, slot format {slot_format_major}");

                    let preset_name = if reader.read_bool32()? {
                        reader.read_string_and_length()?
                    } else {
                        None
                    };

                    let mut preset_path = Vec::new();
                    if slot_format_major == 5 {
                        preset_path.push(reader.read_string_and_length()?.unwrap_or_default());
                    } else {
                        preset_path = reader.read_path()?;
                    }

                    if host_version.is_zero()
                        || host_version.is_at_least(&PhasePlantRelease::V1_7_0.version())
                    {
                        reader.skip(1)?;
                    }

                    let preset_edited = slot_format_major > 5 && reader.read_bool32()?;

                    let mut effect_read_return =
                        effect_mode.read_effect(&mut reader, effect_version)?;

                    effect_read_return.preset_name = preset_name;
                    effect_read_return.preset_path = preset_path;
                    effect_read_return.preset_edited = preset_edited;
                    effect_read_return
                };

                let effect_end_pos = reader.stream_position()?;
                let remaining = effect_length as i64 - (effect_end_pos - effect_start_pos) as i64;
                if remaining != 0 {
                    let msg = format!("Effect {name_desc} version {effect_version} starting at {effect_start_pos} had {remaining} bytes remaining");
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }

                let snapin = Snapin {
                    name: name_opt.unwrap().to_string(),
                    enabled: effect_read_return.enabled,
                    minimized: effect_read_return.minimized,
                    position,
                    metadata: effect_read_return.metadata,
                    preset_name: effect_read_return.preset_name.unwrap_or_default(),
                    preset_path: effect_read_return.preset_path,
                    preset_edited: effect_read_return.preset_edited,
                    host_version,
                    effect_version,
                    effect: effect_read_return.effect,
                };
                debug!("snapin {:?}", snapin);
                lane.snapins.push(snapin);
            }
        }
        trace!("lane: end of lanes pos {}", reader.pos());

        // Audio sources used by the Audio Follower modulator.
        if reader.is_version_at_least_2_0() {
            for mod_block in &mut mod_blocks {
                let id = u32::from_le(reader.read_u32()?);
                let name = reader.read_string_and_length()?.unwrap_or_default();
                mod_block.audio_source = AudioSourceId::new(id, name);
            }
        }

        //
        // String pool
        //

        trace!("string pool: pos {}", reader.pos());
        let string_pool_len = if reader.is_release_at_least(PhasePlantRelease::V1_8_5) {
            200
        } else if reader.is_release_at_least(PhasePlantRelease::V1_8_0) {
            200 - 32
        } else {
            200 - 32 - 32
        };
        let mut string_pool = Vec::with_capacity(string_pool_len);
        for _ in 0..string_pool_len {
            string_pool.push(reader.read_string_and_length()?);
        }
        trace!(
            "string pool: length {}, contents: {string_pool:?}",
            string_pool.len()
        );

        for (index, gen_block) in gen_blocks.iter_mut().enumerate() {
            gen_block.sample_name = string_pool[index].to_owned();
        }

        for (index, mod_block) in mod_blocks.iter_mut().enumerate() {
            mod_block.shape_name = string_pool[index + 32].clone();
        }

        for (index, gen_block) in gen_blocks.iter_mut().enumerate() {
            let name = string_pool[index + 64].to_owned().unwrap_or_default();
            if !name.is_empty() {
                gen_block.name = name;
            }
        }

        for (index, macro_control) in macro_controls.iter_mut().enumerate() {
            macro_control.name = string_pool[index + 96].clone().unwrap_or_default();
        }

        for (index, gen_block) in gen_blocks.iter_mut().enumerate() {
            gen_block.wavetable_name = string_pool[index + 104].to_owned();
        }

        if reader.is_release_at_least(PhasePlantRelease::V1_8_0) {
            for (index, mod_block) in mod_blocks.iter_mut().enumerate() {
                mod_block.shape_path = string_pool[index + 136].clone();
            }
        }

        // FIXME: Detection code to figure out what the added 32 strings are for
        if reader.is_release_at_least(PhasePlantRelease::V1_8_5) {
            for (index, text_opt) in string_pool[168..200].iter().enumerate() {
                if let Some(text) = text_opt {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!(
                            "Unexpected empty string pool text, have '{text}' at index {index}"
                        ),
                    ));
                }
            }
        }

        if reader.is_version_at_least_2_0() {
            for gen in &mut gen_blocks {
                gen.curve_name = reader.read_string_and_length()?;
                gen.curve_path = reader.read_string_and_length()?;
            }
        }

        //
        // Data blocks
        //

        trace!(
            "data block: modulators {:?} pos {}",
            mod_blocks
                .iter()
                .map(|modulator| modulator.mode)
                .collect::<Vec<_>>(),
            reader.pos()
        );

        for mod_block in &mut mod_blocks {
            // Each modulator has two consecutive data blocks.
            for data_block_index in 0..2 {
                let data_pos = reader.stream_position()?;
                let data_header = reader.read_block_header()?;
                if data_header.is_used {
                    mod_block.read_data_block(&mut reader)?;
                }

                let remaining = -(reader.stream_position()? as i64
                    - data_pos as i64
                    - data_header.data_length_with_header() as i64);
                if remaining != 0 {
                    let msg = format!("Modulator {} had {remaining} bytes remaining after data block {data_block_index} starting at {data_pos}", mod_block.mode);
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
            }
        }

        trace!(
            "data block: generators {:?}, pos {}",
            gen_blocks.iter().map(|gen| gen.mode).collect::<Vec<_>>(),
            reader.pos()
        );
        for (gen_index, gen_block) in gen_blocks.iter_mut().enumerate() {
            //
            // Sample player
            //

            let start_pos = reader.stream_position()?;
            let header = reader.read_block_header()?;
            let expected_end_pos = start_pos as usize + header.data_length_with_header();
            if gen_block.mode != GeneratorMode::Blank {
                trace!("data block: sample player: index {gen_index}, start pos {start_pos}, expected end pos {expected_end_pos}, header {header:?}");
            }

            if header.is_used() {
                let mode_id = header.mode_id().expect("sampler header mode");
                gen_block.sample_path = reader.read_string_and_length()?;

                if mode_id == 1 {
                    // No additional data.
                } else if mode_id == 3 {
                    reader.expect_u8(0, "sample_player_contents_1")?;
                } else if mode_id != 2 {
                    let msg = format!(
                        "Unknown sample player data block mode {mode_id} at position {}",
                        reader.stream_position()? - size_of::<u32>() as u64
                    );
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }

                let remaining = expected_end_pos as i64 - reader.stream_position()? as i64;
                if remaining != 0 {
                    let contents_length = reader.read_u32()?;
                    let mut contents = vec![0u8; contents_length as usize];
                    reader.read_exact(&mut contents)?;
                    gen_block.sample_contents = contents;
                }
            }

            let remaining = expected_end_pos as i64 - reader.stream_position()? as i64;
            if remaining != 0 {
                let msg = format!("Sample player data block index {gen_index} had {remaining} bytes remaining at {}", reader.pos());
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }

            //
            // Wavetable
            //

            let start_pos = reader.stream_position()?;
            let header = reader.read_block_header()?;
            let expected_end_pos = start_pos as usize + header.data_length_with_header();
            if gen_block.mode != GeneratorMode::Blank {
                trace!("data block: wavetable: index {gen_index}, start pos {start_pos}, expected end pos {expected_end_pos}, header {header:?}");
            }

            if header.is_used {
                let mode_id = header.mode_id().expect("wavetable header mode");
                if mode_id != 1 && mode_id != 3 {
                    let msg = format!(
                        "Unknown wavetable data block mode {mode_id} at position {}",
                        reader.stream_position()? - size_of::<u32>() as u64
                    );
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }

                gen_block.wavetable_path = reader.read_string_and_length()?;
                trace!(
                    "data block: mode wavetable path {:?}, name {:?}",
                    gen_block.wavetable_path,
                    gen_block.sample_name
                );

                if mode_id == 3 {
                    gen_block.wavetable_edited = reader.read_bool8()?;
                }

                let remaining = expected_end_pos as i64 - reader.stream_position()? as i64;
                if remaining != 0 {
                    let contents_length = reader.read_u32()?;
                    trace!("data block: wavetable contents length {contents_length}");
                    let mut contents = vec![0u8; contents_length as usize];
                    reader.read_exact(&mut contents)?;
                    gen_block.wavetable_contents = contents;
                }
            }

            let remaining = expected_end_pos as i64 - reader.stream_position()? as i64;
            if remaining != 0 {
                let msg = format!("Wavetable data block {gen_index} starting at {start_pos} had {remaining} bytes remaining");
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
            reader.skip(remaining)?;
        }

        if reader.is_version_at_least_2_0() {
            // LFO Table
            trace!("modulator: lfo table data block position {}", reader.pos());
            for mod_block in &mut mod_blocks {
                let header = reader.read_block_header()?;
                if header.is_used {
                    mod_block.lfo_table_wavetable_path = reader.read_string_and_length()?;

                    let has_contents = reader.read_bool8()?;
                    if has_contents {
                        let contents_length = reader.read_u32()?;
                        let mut contents = vec![0u8; contents_length as usize];
                        reader.read_exact(&mut contents)?;
                        mod_block.lfo_table_wavetable_contents = contents;
                    }
                }
            }

            trace!(
                "modulator: curve output data block position {}",
                reader.pos()
            );
            for gen in &mut gen_blocks {
                let start_pos = reader.stream_position()?;
                if gen.mode != GeneratorMode::Blank {
                    trace!("data block: curve_output: start pos {start_pos}");
                }

                let header = reader.read_block_header()?;
                let expected_end_pos = start_pos as usize + header.data_length_with_header();
                if header.is_used {
                    gen.read_data_block(&mut reader)?;
                }

                let remaining = expected_end_pos as i64 - reader.stream_position()? as i64;
                if remaining != 0 {
                    let msg = format!("Curve output data block starting at {start_pos} had {remaining} bytes remaining");
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
            }
        }

        // Should be at the end of the file.
        if reader.read_u8().is_ok() {
            warn!(
                "Expected end of file was not found at position {}",
                reader.pos()
            );
        }

        // Convert the modulator blocks to modulators.
        let mut modulator_containers: Vec<ModulatorContainer> = Vec::with_capacity(MODULATORS_MAX);
        for block in mod_blocks
            .iter()
            .filter(|block| block.mode != ModulatorMode::Blank)
        {
            use ModulatorMode::*;
            let modulator: Box<dyn Modulator> = match block.mode {
                AudioFollower => Box::new(AudioFollowerModulator::from(block)),
                Blank => Box::new(BlankModulator::from(block)),
                Curve => Box::new(CurveModulator::from(block)),
                Envelope => Box::new(EnvelopeModulator::from(block)),
                Group => Box::new(modulator::Group::from(block)),
                Lfo => Box::new(LfoModulator::from(block)),
                LfoTable => Box::new(LfoTableModulator::from(block)),
                LowerLimit => Box::new(LowerLimitModulator::from(block)),
                MidiCc => Box::new(MidiCcModulator::from(block)),
                MpeTimbre => Box::new(MpeTimbreModulator::from(block)),
                Note => Box::new(NoteModulator::from(block)),
                NoteGate => Box::new(NoteGateModulator::from(block)),
                PitchTracker => Box::new(PitchTrackerModulator::from(block)),
                PitchWheel => Box::new(PitchWheelModulator::from(block)),
                Pressure => Box::new(PressureModulator::from(block)),
                Scale => Box::new(ScaleModulator::from(block)),
                Random => Box::new(RandomModulator::from(block)),
                Remap => Box::new(RemapModulator::from(block)),
                SampleAndHold => Box::new(SampleAndHoldModulator::from(block)),
                SlewLimiter => Box::new(SlewLimiterModulator::from(block)),
                UpperLimit => Box::new(UpperLimitModulator::from(block)),
                Velocity => Box::new(VelocityModulator::from(block)),
            };

            let container = ModulatorContainer {
                id: block.id,
                group_id: block.group_id,
                enabled: block.enabled,
                minimized: block.minimized,
                modulator,
            };

            modulator_containers.push(container);
        }

        // Convert the generator blocks to generators, removing the Blank generators.
        // FIXME: Sort by the position.
        let mut generators: Vec<Box<dyn Generator>> = Vec::with_capacity(GENERATORS_MAX as usize);
        for block in gen_blocks
            .iter()
            .filter(|block| block.mode != GeneratorMode::Blank)
        {
            let generator: Box<dyn Generator> = match block.mode {
                GeneratorMode::AnalogOscillator => Box::new(AnalogOscillator::from(block)),
                GeneratorMode::AuxRouting => Box::new(AuxRouting::from(block)),
                GeneratorMode::Blank => Box::new(BlankGenerator::from(block)),
                GeneratorMode::CurveOutput => Box::new(CurveOutput::from(block)),
                GeneratorMode::EnvelopeOutput => Box::new(EnvelopeOutput::from(block)),
                GeneratorMode::DistortionEffect => Box::new(DistortionEffect::from(block)),
                GeneratorMode::FilterEffect => Box::new(FilterEffect::from(block)),
                GeneratorMode::GranularGenerator => Box::new(GranularGenerator::from(block)),
                GeneratorMode::Group => Box::new(generator::Group::from(block)),
                GeneratorMode::MixRouting => Box::new(MixRouting::from(block)),
                GeneratorMode::NoiseGenerator => Box::new(NoiseGenerator::from(block)),
                GeneratorMode::SamplePlayer => Box::new(SamplePlayer::from(block)),
                GeneratorMode::WavetableOscillator => Box::new(WavetableOscillator::from(block)),
            };
            generators.push(generator);
        }

        Ok(Preset {
            format_version: reader.format_version,
            generators,
            mod_wheel_value,
            glide_enabled,
            glide_time,
            glide_legato,
            lanes,
            macro_controls,
            master_gain,
            master_pitch,
            metadata,
            modulations,
            modulator_containers,
            polyphony,
            retrigger_enabled,
            unison,
        })
    }
}

#[cfg(test)]
mod test {
    use std::str;

    use approx::assert_relative_eq;

    use crate::test::read_preset;
    use crate::tests::test_data_path;
    use crate::*;

    #[test]
    fn glide() {
        let preset = read_preset("misc", "glide-on-1.8.13.phaseplant");
        assert!(preset.glide_enabled);
        assert!(!preset.glide_legato);
        assert_relative_eq!(preset.glide_time, 0.0);

        let preset = read_preset("misc", "glide-on-42ms-legato-1.8.13.phaseplant");
        assert!(preset.glide_enabled);
        assert!(preset.glide_legato);
        assert_relative_eq!(preset.glide_time, 0.042);
    }

    /// Test all of the presets in the init directory. They must have the file
    /// name `init-#.#.#.phaseplant`.
    #[test]
    fn init() {
        let preset_dir = test_data_path(&["init"]);
        for dir_entry_result in preset_dir
            .read_dir()
            .expect("read directory {preset_dir:?}")
        {
            let dir_entry = &dir_entry_result.unwrap();
            let pathbuf = dir_entry.path();
            let path = pathbuf.as_path();
            if !dir_entry.file_type().unwrap().is_dir()
                && path.extension().unwrap_or_default() == "phaseplant"
            {
                let mut preset = read_preset("init", &path.file_name().unwrap().to_string_lossy());
                let metadata = &preset.metadata;
                assert!(metadata.description.is_none());

                let file_stem = path.file_stem().map(|s| s.to_string_lossy().to_string());
                assert_eq!(metadata.name, file_stem);

                // Clear the identifying metadata and version so it matches the
                // default for comparison.
                preset.metadata = Metadata::default();
                preset.format_version = Preset::default().format_version;
                assert_eq!(Preset::default(), preset, "File {path:?}");
            }
        }
    }

    #[test]
    fn lane_disabled_version_1() {
        let preset = read_preset("lanes", "lane-1disabled-1.8.13.phaseplant");
        assert!(!preset.lanes[0].enabled);
        assert!(preset.lanes[1].enabled);
        assert!(preset.lanes[2].enabled);
    }

    #[test]
    fn lane_disabled_version_2() {
        let preset = read_preset("lanes", "lane-1disabled-2.0.12.phaseplant");
        assert!(!preset.lanes[0].enabled);
        assert!(preset.lanes[1].enabled);
        assert!(preset.lanes[2].enabled);
    }

    #[test]
    fn lane_gain() {
        let preset = read_preset("lanes", "lane-gains-3-5-10-1.8.13.phaseplant");
        assert_relative_eq!(
            Decibels::from_linear(preset.lanes[0].gain).db(),
            3.0,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            Decibels::from_linear(preset.lanes[1].gain).db(),
            5.0,
            epsilon = 0.0001
        );
        assert_relative_eq!(
            Decibels::from_linear(preset.lanes[2].gain).db(),
            10.0,
            epsilon = 0.0001
        );
    }

    #[test]
    fn lane_parts_version_1() {
        let preset = read_preset("lanes", "lane-1poly-2mute-3solo-1.8.13.phaseplant");
        assert!(preset.lanes[0].enabled);
        assert_eq!(preset.lanes[0].poly_count, 1);
        assert!(!preset.lanes[0].mute);
        assert!(!preset.lanes[0].solo);
        assert!(preset.lanes[1].enabled);
        assert_eq!(preset.lanes[1].poly_count, 0);
        assert!(preset.lanes[1].mute);
        assert!(!preset.lanes[1].solo);
        assert!(preset.lanes[2].enabled);
        assert_eq!(preset.lanes[2].poly_count, 0);
        assert!(!preset.lanes[2].mute);
        assert!(preset.lanes[2].solo);

        let preset = read_preset("lanes", "lane-all-master-1.8.13.phaseplant");
        assert_eq!(preset.lanes[0].destination, LaneDestination::Master);
        assert_eq!(preset.lanes[1].destination, LaneDestination::Master);
        assert_eq!(preset.lanes[2].destination, LaneDestination::Master);

        let preset = read_preset("lanes", "lane-mix-25%-50%-75%-1.8.13.phaseplant");
        assert_eq!(preset.lanes[0].mix.get::<percent>(), 25.0);
        assert_eq!(preset.lanes[1].mix.get::<percent>(), 50.0);
        assert_eq!(preset.lanes[2].mix.get::<percent>(), 75.0);

        let preset = read_preset("lanes", "lane-mix-35%-65%-90%-1.8.13.phaseplant");
        assert_eq!(preset.lanes[0].mix.get::<percent>(), 35.0);
        assert_eq!(preset.lanes[1].mix.get::<percent>(), 65.0);
        assert_eq!(preset.lanes[2].mix.get::<percent>(), 90.0);

        // FIXME: Test lane*-haas
    }

    #[test]
    fn lane_parts_version_2() {
        let preset = read_preset("lanes", "lane-1poly-2mute-3solo-2.0.12.phaseplant");
        assert!(preset.lanes[0].enabled);
        assert_eq!(preset.lanes[0].poly_count, 1);
        assert!(!preset.lanes[0].mute);
        assert!(!preset.lanes[0].solo);
        assert!(preset.lanes[1].enabled);
        assert_eq!(preset.lanes[1].poly_count, 0);
        assert!(preset.lanes[1].mute);
        assert!(!preset.lanes[1].solo);
        assert!(preset.lanes[2].enabled);
        assert_eq!(preset.lanes[2].poly_count, 0);
        assert!(!preset.lanes[2].mute);
        assert!(preset.lanes[2].solo);
    }

    #[test]
    fn master_gain() {
        let preset = read_preset("misc", "master-gain-+3db-1.8.13.phaseplant");
        assert_relative_eq!(
            Decibels::from_linear(preset.master_gain).db(),
            3.0,
            epsilon = 0.000001
        );

        let preset = read_preset("misc", "master-gain-+10db-1.8.13.phaseplant");
        assert_relative_eq!(
            Decibels::from_linear(preset.master_gain).db(),
            10.0,
            epsilon = 0.000001
        );

        let preset = read_preset("misc", "master-gain--20db-1.8.13.phaseplant");
        assert_relative_eq!(
            Decibels::from_linear(preset.master_gain).db(),
            -20.0,
            epsilon = 0.000001
        );

        let preset = read_preset("misc", "master-gain--inf-1.8.13.phaseplant");
        assert_relative_eq!(
            Decibels::from_linear(preset.master_gain).db(),
            f32::NEG_INFINITY,
            epsilon = 0.000001
        );
    }

    #[test]
    fn master_pitch() {
        let preset = read_preset("misc", "master-pitch-12semis-50cents-1.8.13.phaseplant");
        assert_relative_eq!(preset.master_pitch, 12.5);
    }

    #[test]
    fn polyphony() {
        let preset = read_preset("misc", "polyphony-4-legato-1.8.13.phaseplant");
        assert_eq!(preset.polyphony, 4);
    }

    #[test]
    fn retrigger() {
        let preset = read_preset("misc", "polyphony-4-legato-1.8.13.phaseplant");
        assert!(!preset.retrigger_enabled);
    }

    /// Check handling of Unicode characters.
    #[test]
    fn unicode() {
        let preset = read_preset("misc", "unicode-name-desc-macro-1.8.13.phaseplant");
        let emoji = str::from_utf8(&[0xf0, 0x9f, 0x92, 0x96]).unwrap();
        assert_eq!(preset.metadata.description.unwrap(), emoji);
        assert_eq!(preset.macro_controls[0].name, emoji);
    }

    #[test]
    fn unison_enabled_version_1() {
        let preset = read_preset("unison", "unison-on-1.8.13.phaseplant");
        let unison = &preset.unison;
        assert!(unison.enabled);
        assert_eq!(unison.voices, 4);
        assert_eq!(unison.mode, UnisonMode::Smooth);
        assert_eq!(unison.detune, 25.0); // cents
        assert_eq!(unison.spread, 0.0); // 0%
        assert_eq!(unison.blend, 1.0); // 100%
        assert_eq!(unison.bias, 0.0); // 0%
    }

    #[test]
    fn unison_enabled_version_2() {
        let preset = read_preset("unison", "unison-on-2.0.12.phaseplant");
        let unison = &preset.unison;
        assert!(unison.enabled);
        assert_eq!(unison.voices, 4);
        assert_eq!(unison.mode, UnisonMode::Smooth);
        assert_eq!(unison.detune, 25.0); // cents
        assert_eq!(unison.spread, 0.0); // 0%
        assert_eq!(unison.blend, 1.0); // 100%
        assert_eq!(unison.bias, 0.0); // 0%
    }

    #[test]
    fn unison_parts_version_1() {
        let preset = read_preset("unison", "unison-8voice-hard-99ct-1.8.13.phaseplant");
        let unison = &preset.unison;
        assert!(unison.enabled);
        assert_eq!(unison.voices, 8);
        assert_eq!(unison.mode, UnisonMode::Hard);
        assert_eq!(unison.detune, 99.0); // cents
        assert_eq!(unison.spread, 0.0); // 0%
        assert_eq!(unison.blend, 1.0); // 100%
        assert_eq!(unison.bias, 0.0); // 0%

        let preset = read_preset("unison", "unison-bias--33%-1.8.13.phaseplant");
        assert_relative_eq!(preset.unison.bias, -0.33);

        let preset = read_preset("unison", "unison-blend-25%-1.8.13.phaseplant");
        assert_relative_eq!(preset.unison.blend, 0.25);

        let preset = read_preset("unison", "unison-blend-88%-1.8.13.phaseplant");
        assert_relative_eq!(preset.unison.blend, 0.88);

        let preset = read_preset("unison", "unison-detune-1.2ct-1.8.13.phaseplant");
        assert_relative_eq!(preset.unison.detune, 1.2345678);

        let preset = read_preset("unison", "unison-detune-50ct-1.8.13.phaseplant");
        assert_relative_eq!(preset.unison.detune, 50.0);

        let preset = read_preset("unison", "unison-spread-25%-1.8.13.phaseplant");
        assert_relative_eq!(preset.unison.spread, 0.25);

        let preset = read_preset("unison", "unison-spread-66%-1.8.13.phaseplant");
        assert_relative_eq!(preset.unison.spread, 0.66);

        let preset = read_preset("unison", "unison-spread-88%-1.8.13.phaseplant");
        assert_relative_eq!(preset.unison.spread, 0.88);
    }

    #[test]
    fn unison_mode() {
        let file_modes = [
            ("dim", UnisonMode::Dim),
            ("fifths", UnisonMode::Fifths),
            ("freqstack", UnisonMode::FreqStack),
            ("hard", UnisonMode::Hard),
            ("harmonics", UnisonMode::Harmonics),
            ("major", UnisonMode::Major),
            ("major7", UnisonMode::Major7),
            ("majormaj7", UnisonMode::MajorMaj7),
            ("minor", UnisonMode::Minor),
            ("minor7", UnisonMode::Minor7),
            ("minormaj7", UnisonMode::MinorMaj7),
            ("octaves", UnisonMode::Octaves),
            ("pitchstack", UnisonMode::PitchStack),
            ("shepard", UnisonMode::Shepard),
            ("smooth", UnisonMode::Smooth),
            ("sus2", UnisonMode::Sus2),
            ("sus4", UnisonMode::Sus4),
            ("synthetic", UnisonMode::Synthetic),
        ];
        for (file, mode) in file_modes.iter() {
            let file_name = format!("unison-mode-{}-1.8.13.phaseplant", file);
            let preset = read_preset("unison", &file_name);
            assert_eq!(preset.unison.mode, *mode);
        }
    }
}
