//! Phase Plant preset writing.
//!
//! All presets are upgraded to the most currently supported file format when
//! written.

use std::io::{Error, ErrorKind, Result, Seek, SeekFrom, Write};
use std::mem::size_of;

use byteorder::{LittleEndian, WriteBytesExt};
use log::{trace, Level};
use serde::Serialize;
use serde_json::ser::PrettyFormatter;
use serde_json::Serializer;
use uom::si::frequency::hertz;
use uom::si::ratio::ratio;
use uom::si::time::second;

use crate::generator::{BlankGenerator, Generator, GeneratorMode, Group};
use crate::io::generators::GeneratorBlock;
use crate::io::modulators::ModulatorBlock;
use crate::io::MetadataJson;
use crate::modulation::*;
use crate::modulator::{BlankModulator, Modulator};
use crate::text::HashTag;
use crate::*;

/// Files are written the same as this version of Phase Plant.
pub(crate) const WRITE_SAME_AS: PhasePlantRelease = PhasePlantRelease::V2_1_0;

const FORMAT_VERSION: Version<u32> = WRITE_SAME_AS.format_version();

const STRING_POOL_COUNT: usize = 200;

pub struct Message {
    /// The `error` level is not supported, return an `Error` instead
    pub level: Level,
    pub description: String,
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.level.to_string())?;
        f.write_str(": ")?;
        f.write_str(&self.description)
    }
}

pub struct WritePresetResult {
    pub messages: Vec<Message>,
}

/// Helper to make writing the Phase Plant format less verbose. Only version
/// 2 format presets are supported.
pub struct PhasePlantWriter<T: Write + Seek> {
    inner: T,
}

impl<T: Write + Seek> PhasePlantWriter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Write the given number of zero bytes. The behavior of `Seek::seek` is
    /// undefined when seeking beyond the end of the file.
    pub(crate) fn skip(&mut self, byte_count: usize) -> Result<()> {
        let mut remaining = byte_count;
        while remaining > 4 {
            self.write_u32(0)?;
            remaining -= 4;
        }
        for _ in 0..remaining {
            self.write_all_u8(&[0])?;
        }
        Ok(())
    }

    pub(crate) fn stream_position(&mut self) -> Result<u64> {
        self.inner.stream_position()
    }

    /// Position in the stream as infallible text, useful for tracing.
    fn pos_text(&mut self) -> String {
        self.stream_position()
            .map(|pos| pos.to_string())
            .unwrap_or_else(|_| "<unknown>".to_owned())
    }

    pub(crate) fn write_block_header(&mut self, header: &DataBlockHeader) -> Result<()> {
        self.write_u32(header.data_length as u32 + 1)?; // Include `used` field
        self.write_bool8(header.is_used)
    }

    pub(crate) fn write_bool8(&mut self, value: bool) -> Result<()> {
        self.inner.write_u8(value as u8)
    }

    pub(crate) fn write_bool32(&mut self, value: bool) -> Result<()> {
        self.inner.write_u32::<LittleEndian>(value as u32)
    }

    pub(crate) fn write_f32(&mut self, value: f32) -> Result<()> {
        self.inner.write_f32::<LittleEndian>(value)
    }

    pub(crate) fn write_hertz(&mut self, value: Frequency) -> Result<()> {
        self.write_f32(value.get::<hertz>())
    }

    pub(crate) fn write_decibels_db(&mut self, value: Decibels) -> Result<()> {
        self.write_f32(value.db())
    }

    pub(crate) fn write_decibels_linear(&mut self, value: Decibels) -> Result<()> {
        self.write_f32(value.linear())
    }

    pub(crate) fn write_ratio(&mut self, value: Ratio) -> Result<()> {
        self.write_f32(value.get::<ratio>())
    }

    pub(crate) fn write_seconds(&mut self, value: Time) -> Result<()> {
        self.write_f32(value.get::<second>())
    }

    pub(crate) fn write_snapin_id(&mut self, pos: Option<SnapinId>) -> Result<()> {
        self.write_u32(pos.unwrap_or_default() as u32)
    }

    pub(crate) fn write_u8(&mut self, value: u8) -> Result<()> {
        self.inner.write_u8(value)
    }

    pub(crate) fn write_u16(&mut self, value: u16) -> Result<()> {
        self.inner.write_u16::<LittleEndian>(value)
    }

    pub(crate) fn write_u32(&mut self, value: u32) -> Result<()> {
        self.inner.write_u32::<LittleEndian>(value)
    }

    pub(crate) fn write_path(&mut self, values: &[String]) -> Result<()> {
        let component_count = values.len() as u32;
        if component_count > PATH_COMPONENT_COUNT_MAX as u32 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Path component count of {component_count} exceeds {PATH_COMPONENT_COUNT_MAX}"
                ),
            ));
        }
        self.write_u32(component_count)?;

        for value in values {
            self.write_string_and_length(value.as_str())?;
        }
        Ok(())
    }

    pub(crate) fn write_string_and_length<S: AsRef<str>>(&mut self, value: S) -> Result<()> {
        self.write_u32(value.as_ref().len() as u32)?;
        self.write_all_u8(value.as_ref().as_bytes())
    }

    pub(crate) fn write_string_and_length_opt(&mut self, value: &Option<String>) -> Result<()> {
        self.write_string_and_length(match value {
            Some(s) => s,
            None => "",
        })
    }

    pub(crate) fn write_all_u8(&mut self, buf: &[u8]) -> Result<()> {
        self.inner.write_all(buf)
    }

    pub(crate) fn write_envelope(&mut self, envelope: &Envelope) -> Result<()> {
        self.write_seconds(envelope.delay)?;
        self.write_seconds(envelope.attack)?;
        self.write_f32(envelope.attack_curve)?;
        self.write_seconds(envelope.hold)?;
        self.write_seconds(envelope.decay)?;
        self.write_f32(envelope.decay_falloff)?;
        self.write_ratio(envelope.sustain)?;
        self.write_seconds(envelope.release)?;
        self.write_f32(envelope.release_falloff)
    }
}

impl GeneratorBlock {
    fn write<W: Write + Seek>(&self, writer: &mut PhasePlantWriter<W>) -> Result<()> {
        trace!("generator: mode {:?}", self.mode);
        writer.write_u32(self.mode as u32)?;
        writer.write_u32(self.id as u32)?;
        writer.write_bool32(self.enabled)?;
        trace!(
            "generator: tuning {}, pos {}",
            self.fine_tuning,
            writer.pos_text()
        );
        writer.write_f32(self.fine_tuning)?;
        writer.write_f32(self.harmonic)?;
        writer.write_hertz(self.shift)?;
        writer.write_ratio(self.level)?;
        writer.write_ratio(self.phase_offset)?;
        writer.write_ratio(self.phase_jitter)?;

        // Unison
        trace!(
            "generator: unison pos {}, voices {}",
            writer.pos_text(),
            self.unison.voices
        );
        let unison = &self.unison;
        writer.write_u32(unison.voices)?;
        writer.write_f32(unison.detune_cents)?;
        writer.write_ratio(unison.spread)?;
        writer.write_ratio(unison.blend)?;

        // Sample player
        trace!(
            "generator: sample player pos {}, root note {}",
            writer.pos_text(),
            self.base_pitch
        );
        writer.write_f32(self.base_pitch)?;
        writer.write_ratio(self.offset_position)?;
        writer.write_u32(self.sample_loop_mode as u32)?;
        writer.write_ratio(self.loop_start_position)?;
        writer.write_ratio(self.loop_length)?;
        writer.write_ratio(self.crossfade_amount)?;

        trace!("generator: wavetable frame pos {}", writer.pos_text());
        writer.write_f32(self.wavetable_frame)?;
        writer.write_hertz(self.band_limit)?;

        writer.write_u32(self.analog_waveform as u32)?;
        writer.write_f32(self.sync_multiplier)?;
        writer.write_ratio(self.pulse_width)?;
        writer.write_u32(self.seed_mode as u32)?;
        writer.write_decibels_db(self.noise_slope)?;
        writer.write_ratio(self.stereo)?;
        writer.write_u32(self.noise_waveform as u32)?;

        trace!("generator: filter effect pos {}", writer.pos_text());
        writer.write_u32(self.filter_effect.filter_mode as u32)?;
        writer.write_hertz(self.filter_effect.cutoff)?;
        writer.write_f32(self.filter_effect.q)?;
        writer.write_decibels_linear(self.filter_effect.gain)?;

        writer.write_u32(self.distortion_effect.mode as u32)?;
        writer.write_decibels_linear(self.distortion_effect.drive)?;
        writer.write_ratio(self.distortion_effect.bias)?;
        writer.write_ratio(self.distortion_effect.mix)?;

        writer.write_bool32(self.invert)?;
        writer.write_ratio(self.mix_level)?;
        writer.write_decibels_linear(self.output_gain)?;
        writer.write_ratio(self.pan)?;
        writer.write_u32(self.output_destination as u32)?;

        trace!("generator: envelope pos {}", writer.pos_text());
        writer.write_envelope(&self.envelope)
    }
}

impl Preset {
    pub fn write<W: Write + Seek>(&self, writer: &mut W) -> Result<WritePresetResult> {
        let mut writer = PhasePlantWriter::new(writer);

        //
        // Header
        //

        // All of the presets are written with the currently supported version.
        writer.write_u32(FORMAT_VERSION.major)?;
        writer.write_u32(FORMAT_VERSION.patch)?;
        writer.write_u32(FORMAT_VERSION.minor)?;

        // Phase Plant traditionally uses hashtags at the end of the description for categorization.
        let mut description = String::with_capacity(64);
        if let Some(desc) = &self.metadata.description {
            description.push_str(desc.as_str())
        }
        if let Some(cat) = &self.metadata.category {
            if !description.is_empty() {
                description.push(' ');
            }
            description.push_str(<dyn HashTag>::from_lossy(cat).as_str())
        }

        //
        // Metadata
        //

        let metadata = MetadataJson {
            description: Some(description), // Use a blank string instead of null
            author: self.metadata.author.clone().or_else(|| Some(String::new())),
        };

        // Use the same spacing as Phase Plant so the files match as closely as
        // possibles to make comparing the files easier.
        let mut metadata_json = Vec::with_capacity(128);
        let formatter = PrettyFormatter::with_indent(b"    ");
        let mut serializer = Serializer::with_formatter(&mut metadata_json, formatter);
        metadata.serialize(&mut serializer)?;
        metadata_json.push(b'\n');

        // Use the exact metadata from test presets when comparing during debugging. serde
        // doesn't use the same formatting as Phase Plant.
        // let metadata_json = "{\n    \"description\": \"\",\n    \"author\": \"softdev.ca\"\n}\n";
        // let metadata_json = "{\n    \"description\": \"\",\n    \"author\": \"\"\n}\n";

        writer.write_u32(metadata_json.len() as u32 + 1)?;
        writer.write_u8(0)?; // Unknown value, always 0
        writer.write_all_u8(&metadata_json)?;
        writer.write_u32(1)?; // Unknown value, always 1

        //
        // Modulation
        //

        let modulation_count = self.modulations.len();
        trace!(
            "modulation: count {modulation_count}, pos {}",
            writer.pos_text()
        );
        if modulation_count > MODULATIONS_MAX {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unexpected number of modulation items ({modulation_count} is more than {MODULATIONS_MAX})"),
            ));
        }
        writer.write_u32(modulation_count as u32)?;

        let default_modulation = Modulation::default();
        for modulation_index in 0..MODULATIONS_MAX {
            let modulation = self
                .modulations
                .get(modulation_index)
                .unwrap_or(&default_modulation);
            writer.write_u32(modulation.source.id())?;
            writer.write_u32(modulation.target.id())?;
            writer.write_f32(modulation.amount.value)?;
        }

        writer.write_u32(1)?; // unknown_m3 always 1

        //
        // Lanes
        //

        // Snapins, poly, mute and solo are later in the data.
        let default_lane = &Default::default();
        for lane_index in 0..Lane::COUNT {
            let lane = &self.lanes.get(lane_index).unwrap_or(default_lane);
            trace!("lane {lane_index}: {lane:?}, pos {}", writer.pos_text());
            trace!(
                "lane {lane_index}: enabled {}, gain {}, mix {:?}, destination {}",
                lane.enabled,
                lane.gain,
                lane.mix,
                lane.destination
            );
            writer.write_bool32(lane.enabled)?;
            writer.write_decibels_linear(lane.gain)?;
            writer.write_ratio(lane.mix)?;
            writer.write_u32(lane.destination as u32)?;
        }

        //
        // Macro controls
        //

        trace!(
            "macro controls: values {:?}, pos {}",
            &self
                .macro_controls
                .iter()
                .map(|ctrl| ctrl.value)
                .collect::<Vec<_>>(),
            writer.pos_text()
        );
        for macro_control in &self.macro_controls {
            writer.write_f32(macro_control.value)?;
        }
        let unused_macro_controls = MacroControl::COUNT - self.macro_controls.len();
        writer.skip(unused_macro_controls * 4)?;

        //
        // Modulators
        //

        // Convert to blocks.
        trace!(
            "modulators: pos {}, modulators {:?}",
            writer.pos_text(),
            self.modulator_containers
        );
        let mut mod_blocks: Vec<ModulatorBlock> = Vec::with_capacity(MODULATORS_MAX);
        fn default_mod() -> Box<dyn Modulator> {
            Box::new(BlankModulator {})
        }
        let default_mod = &default_mod();
        for mod_index in 0..MODULATORS_MAX {
            let modulator = self
                .modulator_containers
                .get(mod_index)
                .map(|container| &container.modulator)
                .unwrap_or_else(|| default_mod);

            // Blank modulators don't have an ID.
            let mut mod_block = modulator.as_block();
            if modulator.mode().is_blank() {
                mod_block.id = 0;
            }

            assert_eq!(modulator.mode(), mod_block.mode);
            mod_blocks.push(mod_block);
        }

        // Write the blocks.
        for block in &mod_blocks {
            let mod_start_pos = writer.stream_position()?;
            trace!("modulator: mode {}, start pos {mod_start_pos}", block.mode,);

            writer.write_u32(block.mode as u32)?;
            writer.write_u32(block.id as u32)?;
            writer.write_bool32(block.enabled)?;

            // Min and max modulators
            writer.write_f32(block.input_a)?;
            writer.write_f32(block.input_b)?;

            // LFO
            if !block.mode.is_blank() {
                trace!("modulator: start of LFO block pos {}", writer.pos_text());
            }
            writer.write_ratio(block.depth)?;
            writer.write_bool32(block.retrigger)?;
            writer.write_u32(block.output_range as u32)?;
            writer.write_hertz(block.rate.frequency)?;
            writer.write_u32(block.rate.numerator)?;
            writer.write_u32(block.rate.denominator as u32)?;
            writer.write_bool32(block.rate.sync)?;
            if !block.mode.is_blank() {
                trace!(
                    "modulator: envelope {:?}, pos {}",
                    block.envelope,
                    writer.pos_text()
                );
            }
            writer.write_envelope(&block.envelope)?;
            if !block.mode.is_blank() {
                trace!("modulator: envelope end pos {}", writer.pos_text());
            }
            writer.write_ratio(block.phase_offset)?;
            writer.write_bool32(block.one_shot)?;
            writer.write_f32(block.multiplier)?;
            if !block.mode.is_blank() {
                trace!("modulator: unknown pos {}", writer.pos_text());
            }
            writer.write_f32(1.0)?; // FIXME: Unknown1
            writer.write_ratio(block.random_smooth)?;
            writer.write_ratio(block.random_jitter)?;
            writer.write_ratio(block.random_chaos)?;

            let modulator_end_pos = writer.stream_position()?;
            writer
                .skip(MODULATOR_BLOCK_SIZE - (modulator_end_pos - mod_start_pos - 12) as usize)?;
        }

        writer.skip(4)?; // unknown_f
        writer.skip(4)?; // unknown_g

        writer.write_u32(self.polyphony)?;

        trace!(
            "modulator: retrigger enabled {}, pos {}",
            self.retrigger_enabled,
            writer.pos_text()
        );
        writer.write_bool32(self.retrigger_enabled)?;

        // Glide
        writer.write_bool32(self.glide_enabled)?;
        writer.write_bool32(self.glide_legato)?;
        writer.write_f32(self.glide_time)?;

        //
        // Generators
        //

        let mut gen_blocks = Vec::with_capacity(GENERATORS_MAX as usize);
        fn default_generator() -> Box<dyn Generator> {
            // Function to get a box of dyn
            Box::new(BlankGenerator {})
        }
        let default_generator = &default_generator();
        for index in 0..GENERATORS_MAX {
            let generator = self
                .generators
                .get(index as usize)
                .unwrap_or(default_generator);
            let mut block = generator.as_block();

            // The position isn't stored with the generators. The position is 1
            // by default in the blank blocks.
            block.id = if generator.mode() == GeneratorMode::Blank {
                0 // FIXME: This contradicts the comment
            } else {
                index as GeneratorId
            };

            gen_blocks.push(block);
        }

        trace!("generators start pos {}", writer.pos_text());
        for block in &gen_blocks {
            let start_pos = writer.stream_position()?;
            if !block.mode.is_blank() {
                trace!("generator: mode {}, pos {start_pos}", block.mode);
            }

            block.write(&mut writer)?;

            let generator_end_pos = writer.stream_position()?;
            writer.skip(GeneratorBlock::SIZE - (generator_end_pos - start_pos) as usize)?;
        }

        //
        // Unison
        //

        trace!("global unison: pos {}", writer.pos_text());
        writer.write_u32(self.unison.voices)?;
        writer.write_f32(self.unison.detune_cents)?;
        writer.write_ratio(self.unison.spread)?;
        writer.write_ratio(self.unison.blend)?;

        writer.write_f32(self.master_gain)?;

        //
        // Lanes
        //

        let default_lane = Lane::default();
        for lane_index in 0..Lane::COUNT {
            trace!("lane {lane_index}: pos {}", writer.pos_text());
            let lane = &self.lanes.get(lane_index).unwrap_or(&default_lane);
            writer.write_u8(lane.poly_count)?;
            writer.skip(1)?;
            writer.write_bool8(lane.mute)?;
            writer.skip(1)?;
            writer.write_bool8(lane.solo)?;
            writer.skip(1)?;
            writer.skip(2)?;

            // The last lane has less padding
            if lane_index < Lane::COUNT - 1 {
                writer.skip(2)?;
            }
        }

        //
        // Minimized
        //

        trace!("modulator: minimized pos {}", writer.pos_text());
        for block in &mod_blocks {
            writer.write_bool32(block.minimized)?;
        }

        trace!("generator: minimized pos {}", writer.pos_text());
        for block in &gen_blocks {
            writer.write_bool32(block.minimized)?;
        }

        writer.skip(4)?;

        for _block in &gen_blocks {
            // FIXME: NO IDEA WHAT THIS IS, HARD CODED TO BYPASS CURRENT DIFF STOPPING POINT
            writer.write_u32(8)?;
            writer.write_u32(4)?;
            writer.write_u32(0)?;
        }

        //
        // Locks
        //

        trace!("generator: locks: pos {}", writer.pos_text());
        for block in &gen_blocks {
            writer.write_bool32(block.base_pitch_locked)?;
            writer.write_bool32(block.offset_locked)?;
            writer.write_bool32(block.loop_locked)?;
        }

        for mod_block in &mut mod_blocks {
            writer.write_bool32(mod_block.shape_edited)?;
        }

        //
        // Filter slope
        //

        for block in &gen_blocks {
            writer.write_u32(block.filter_effect.slope)?;
        }

        // Unknown
        trace!("unknown blocks: pos 2 {}", writer.pos_text());
        writer.skip(128)?;
        for _block in &gen_blocks {
            writer.write_u32(69)?; // unknown_g
            writer.write_u32(120)?; // unknown_m
        }

        //
        // Unison
        //

        trace!("unison mode: blocks pos {}", writer.pos_text());
        for block in &gen_blocks {
            writer.write_u32(block.unison.mode as u32)?;
        }
        writer.write_u32(self.unison.mode as u32)?;

        for block in &gen_blocks {
            writer.write_ratio(block.unison.bias)?;
        }
        writer.write_ratio(self.unison.bias)?;

        for block in &gen_blocks {
            writer.write_bool32(block.unison.enabled)?;
        }
        writer.write_bool32(self.unison.enabled)?;

        //
        // Loop enabled
        //

        trace!("loop enabled: pos {}", writer.pos_text());
        for block in &gen_blocks {
            writer.write_bool32(block.loop_enabled)?;
        }

        trace!("version 2: block A: pos {}", writer.pos_text());
        for _ in 0..100 {
            writer.write_u32(0)?;
            writer.write_bool32(true)?;
        }

        trace!("version 2: block B: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_u8(0)?;
        }

        trace!("version 2: block D: pos {}", writer.pos_text());
        for mod_block in &mut mod_blocks {
            writer.write_f32(1.0)?;
            writer.write_u32(mod_block.group_id)?;
            writer.write_f32(0.5)?;
            writer.write_u32(3)?;
            writer.write_u32(0)?;
            writer.write_bool32(true)?;
            writer.write_u32(36)?; // 0x24
            writer.write_u32(84)?; // 0x54
            writer.write_f32(0.5)?;
            writer.write_u32(69)?;
            writer.write_u32(0xFFFFFFFF)?; // -1 as i32
            writer.write_u32(0)?;
        }

        trace!("version 2: block E: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_u32(0)?;
        }

        trace!("version 2: block F: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_f32(1.0)?;
            writer.write_u32(0)?;
            writer.write_u32(0)?;
            writer.write_f32(1.0)?;
            writer.write_u32(0)?;
        }

        trace!("version 2: block G: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_u32(0)?;
            writer.write_u32(0)?;
            writer.write_f32(1.0)?;
            writer.write_f32(1.0)?;
            writer.write_u32(4)?;
            writer.write_u32(4)?;
            writer.write_u32(0)?;
            writer.write_u32(0)?;
            writer.write_u32(0)?;
            writer.write_f32(1.0)?;
            writer.write_u32(0)?;
        }

        trace!("version 2: block H: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_f32(1.0)?;
            writer.write_f32(0.002)?; // 0x6F 12 03 3B
        }

        trace!("version 2: block I: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_f32(1.0)?; // 0x00 00 80 3F
        }

        trace!("version 2: block J: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_u32(0)?;
            writer.write_u32(0)?;
            writer.write_u32(0)?;
        }

        trace!("version 2: block K: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_bool32(true)?;
        }

        // Added in Phase Plant 2.0.12
        trace!("version 2: block L: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_f32(0.1)?; // 0xCD CC CC 3D
            writer.write_f32(0.1)?; // 0xCD CC CC 3D
        }

        // Added in Phase Plant 2.0.13
        trace!("version 2: block M: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_bool32(true)?;
        }

        //
        // Lanes containing Effects
        //

        for lane_index in 0..Lane::COUNT {
            trace!("lane {lane_index}: pos {}", writer.pos_text());
            if lane_index >= self.lanes.len() {
                writer.write_u32(0)?;
            } else {
                let lane = &self.lanes[lane_index];
                let snapin_count = lane.snapins.len();
                trace!(
                    "lane {lane_index}: snapin count {snapin_count}, snapins {:?}",
                    lane.snapins
                );
                writer.write_u32(snapin_count as u32)?;

                for snapin in &lane.snapins {
                    let mut effect_id_bytes = u32::to_ne_bytes(snapin.effect.mode() as u32);
                    effect_id_bytes.reverse();
                    let effect_id = String::from_utf8_lossy(&effect_id_bytes).into_owned();
                    trace!(
                        "snapin: {}, id {effect_id}, pos {}",
                        snapin.name,
                        writer.pos_text()
                    );
                    writer.write_all_u8(effect_id.as_bytes())?;

                    // The version of Phase Plant the preset was saved with. The host can adjust
                    // the effect based on the version so it sounds the same across versions.
                    writer.write_u8(WRITE_SAME_AS.version().patch)?;
                    writer.write_u8(WRITE_SAME_AS.version().minor)?;
                    writer.write_u8(WRITE_SAME_AS.version().major)?;
                    writer.write_u8(WRITE_SAME_AS.version().extra)?;

                    writer.write_string_and_length(snapin.name.as_str())?;
                    writer.write_u16(snapin.id)?;

                    // Effect
                    let effect_start_pos = writer.stream_position()?;
                    writer.write_u32(0)?; // Length, updated later
                    writer.write_u32(FORMAT_VERSION.major)?;
                    writer.write_u32(snapin.effect_version)?;
                    writer.write_bool32(true)?; // Unknown snapin bool
                    writer.write_string_and_length(snapin.preset_name.as_str())?;
                    // FIXME: Probably need to do something different with effects with metadata. See reading.
                    writer.write_path(&snapin.preset_path)?;
                    writer.write_bool32(snapin.preset_edited)?;
                    writer.write_u8(0)?; // snapin_prologue_unknown
                    snapin.effect.write(&mut writer, snapin)?;
                    let effect_end_pos = writer.stream_position()?;
                    writer.inner.seek(SeekFrom::Start(effect_start_pos))?;
                    writer.write_u32(
                        (effect_end_pos - effect_start_pos - 4/* Don't count length field */)
                            as u32,
                    )?;
                    writer.inner.seek(SeekFrom::Start(effect_end_pos))?;
                }
            }
        }

        for _ in 0..32 {
            // "niam" is "main" reversed
            writer.write_all_u8("niam".as_bytes())?;
            writer.write_string_and_length("Master")?;
        }

        //
        // String pool
        //

        trace!("string pool: pos {}", writer.pos_text());
        let mut string_pool: Vec<Option<String>> = Vec::with_capacity(STRING_POOL_COUNT);
        for block in &gen_blocks {
            string_pool.push(block.sample_name.clone());
        }

        for block in &mod_blocks {
            string_pool.push(block.shape_name.clone());
        }

        for block in &gen_blocks {
            // Don't store the default names.
            if block.name != block.mode.name() {
                // Phase Plant 1.8 has a limit on the length of group names.  The file format
                // doesn't have a limit but the application does.
                let mut truncated = block.name.to_owned();
                if block.mode == GeneratorMode::Group {
                    truncated.truncate(Group::MAX_NAME_LENGTH)
                }
                string_pool.push(Some(truncated))
            } else {
                string_pool.push(None);
            }
        }

        for index in 0..MacroControl::COUNT {
            string_pool.push(self.macro_controls.get(index).map(|c| c.name.clone()));
        }

        // 104
        for block in &gen_blocks {
            string_pool.push(block.wavetable_name.clone());
        }

        // 136
        for block in &mod_blocks {
            string_pool.push(block.shape_path.clone());
        }

        for _ in 0..(STRING_POOL_COUNT - string_pool.len()) {
            string_pool.push(None)
        }
        assert_eq!(STRING_POOL_COUNT, string_pool.len());
        trace!("string pool: contents: {:?}", string_pool);

        for item in &string_pool {
            match item {
                Some(s) => writer.write_string_and_length(s),
                _ => writer.write_string_and_length(""),
            }?
        }

        trace!("version 2: block Q: pos {}", writer.pos_text());
        for _ in 0..32 {
            writer.write_u32(0)?;
            writer.write_u32(0)?;
        }

        //
        // Data blocks
        //

        trace!(
            "data block: modulators {:?}, pos {}",
            mod_blocks
                .iter()
                .map(|modulator| modulator.mode)
                .collect::<Vec<_>>(),
            writer.pos_text()
        );

        // Each modulator has two consecutive data blocks.
        for mod_block in &mod_blocks {
            let version = WRITE_SAME_AS.format_version().major;
            if !mod_block.shape.is_empty() {
                let point_count = mod_block.shape.len();
                let data_length = point_count * (5 * size_of::<u32>()) + (2 * size_of::<u32>());
                trace!(
                    "data block: shape, points {point_count}, data length {data_length}, pos {}",
                    writer.stream_position()? - 4
                );
                writer.write_block_header(&DataBlockHeader::new_used(data_length, version))?;
                writer.write_u32(1)?; // FIXME: unknown_a, see reader
                writer.write_u32(point_count as u32)?;
                for point in &mod_block.shape {
                    writer.write_f32(point.x)?;
                    writer.write_f32(point.y)?;
                    writer.write_f32(point.curve_x)?;
                    writer.write_f32(point.curve_y)?;
                    writer.write_u32(point.mode as u32)?;
                }

                writer.write_block_header(&DataBlockHeader::new_unused())?;
            } else {
                writer.write_block_header(&DataBlockHeader::new_unused())?;
                writer.write_block_header(&DataBlockHeader::new_unused())?;
            }
        }

        trace!(
            "data block: generators {:?}, pos {}",
            self.generators
                .iter()
                .map(|generator| generator.name())
                .collect::<Vec<String>>(),
            writer.pos_text()
        );

        for block in &gen_blocks {
            trace!("data block: generator pos {}", writer.pos_text());

            // Sample path and contents
            let sample_path_len = block.sample_path.as_ref().map(|s| s.len()).unwrap_or(0);
            let sample_used = sample_path_len > 0 || !block.sample_contents.is_empty();
            if sample_used {
                writer.write_u32(
                    (sample_path_len + block.sample_contents.len()) as u32 + 4 /* Mode */
                        + 4 /* Path string length */
                        + 4 /* Sample contents length */
                        + 1, /* Used */
                )?;
                writer.write_bool8(sample_used)?;

                writer.write_u32(2)?; // Mode
                writer.write_string_and_length_opt(&block.sample_path)?;

                // Sample contents
                writer.write_u32(block.sample_contents.len() as u32)?;
                writer.write_all_u8(&block.sample_contents)?;
            } else {
                writer.write_u32(1)?;
                writer.write_bool8(false)?;
            }

            // Wavetable path
            let wavetable_path_len = block.wavetable_path.as_ref().map(|s| s.len()).unwrap_or(0);
            let wavetable_used = wavetable_path_len > 0 || !block.wavetable_contents.is_empty();
            if wavetable_used {
                writer
                    .write_u32((wavetable_path_len + block.wavetable_contents.len()) as u32 + 13)?;
                writer.write_bool8(wavetable_used)?;
                writer.write_u32(3)?; // Mode
                writer.write_string_and_length_opt(&block.wavetable_path)?;

                // Wavetable contents
                writer.write_u32(block.wavetable_contents.len() as u32)?;
                writer.write_all_u8(&block.wavetable_contents)?;
            } else {
                writer.write_u32(1)?;
                writer.write_bool8(false)?;
            }
        }

        for _ in 0..32 {
            writer.write_block_header(&DataBlockHeader::new_unused())?;
        }

        for _ in 0..32 {
            writer.write_block_header(&DataBlockHeader::new_unused())?;
        }

        writer.inner.flush()?;

        Ok(WritePresetResult {
            messages: Vec::new(),
        })
    }
}

#[cfg(test)]
mod test {
    use std::io::{Cursor, Seek, SeekFrom};

    use crate::test::read_preset;

    use super::*;

    #[test]
    fn init_version_1() {
        let mut init_preset = read_preset("init", "init-1.8.13.phaseplant");
        // The init preset file includes non-default metadata
        init_preset.metadata = Metadata::default();

        // The preset is always written as the latest supported version.  Make
        // them match so they can be compared.
        let compare_preset = Preset::default();
        init_preset.format_version = compare_preset.format_version;

        assert_eq!(init_preset, compare_preset);
    }

    #[test]
    fn init_version_2() {
        let mut init_preset = read_preset("init", "init-2.0.12.phaseplant");
        // The init preset file includes non-default metadata
        init_preset.metadata = Metadata::default();

        // The preset is always written as the latest supported version.  Make
        // them match so they can be compared.
        let compare_preset = Preset::default();
        init_preset.format_version = FORMAT_VERSION;

        assert_eq!(init_preset, compare_preset);
    }

    /// Write the default preset and read it back, making sure the contents
    /// match the default. The files cannot be compared directly because the
    /// defaults in the unused areas can change between versions.
    // #[test]
    fn _defaults() {
        let default_preset = Preset::default();
        assert!(default_preset.retrigger_enabled);

        // Writing the preset shouldn't product any warning messages.
        let mut cursor = Cursor::new(Vec::with_capacity(16 * 1024));
        let write_result = default_preset.write(&mut cursor).unwrap();
        assert!(write_result.messages.is_empty());
        cursor.seek(SeekFrom::Start(0)).unwrap();

        // The default preset must not change when saved and reloaded.
        let read_back_preset = Preset::read(&mut cursor, None).expect("default preset");
        assert_eq!(default_preset, read_back_preset);
    }
}
