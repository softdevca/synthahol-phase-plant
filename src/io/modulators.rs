use std::io;
use std::io::{Read, Seek};

use log::{trace, warn};
use music_note::midi;
use uom::si::f32::Time;
use uom::si::f32::{Frequency, Ratio};
use uom::si::frequency::hertz;
use uom::si::ratio::{percent, ratio};
use uom::si::time::{millisecond, second};

use crate::generator::LoopMode;
use crate::modulator::*;
use crate::*;
use crate::{NoteValue, PhasePlantReader, Rate};

// TODO: Make ModulatorBlock crate-private.

/// All modulators are stored in the preset using the same structure. Modulators
/// are converted to and from this structure for reading and writing. Having
/// specific modulators makes the models more clear than having everything in
/// one giant block.
#[derive(Clone, Debug)]
pub struct ModulatorBlock {
    pub mode: ModulatorMode,

    /// Uniquely identifies the modulator so they can be reordered without
    /// effecting modulation routing.
    pub id: ModulatorId,

    pub enabled: bool,
    pub minimized: bool,

    pub output_range: OutputRange,
    pub loop_mode: LoopMode,

    // Min, Max, Multiplier modulators
    pub input_a: f32,
    pub input_b: f32,
    pub multiplier: f32,

    pub depth: Ratio,
    pub retrigger: bool,
    pub phase_offset: Ratio,

    /// One shot was removed from LFO modulator and replaced with [loop_mode](Self::loop_mode)
    pub one_shot: bool,

    pub envelope: Envelope,
    pub rate: Rate,
    pub velocity_trigger_mode: VelocityTriggerMode,
    pub note_trigger_mode: NoteTriggerMode,
    pub trigger_threshold: f32,
    pub voice_mode: VoiceMode,

    // Audio Follower
    pub metering_mode: MeteringMode,
    pub gain: Decibels,
    pub audio_source: AudioSourceId,

    // Curve
    pub curve_time: Time,

    // LFO
    pub shape: Vec<CurvePoint>,
    pub shape_name: Option<String>,
    pub shape_path: Option<String>,

    // LFO Table
    pub lfo_table_smooth: Ratio,
    pub lfo_table_frame: f32,
    pub lfo_table_wavetable_contents: Vec<u8>,
    pub lfo_table_wavetable_path: Option<String>,

    /// Set when the shape is no longer the same as the contents of the file
    /// named by the shape path.
    pub shape_edited: bool,

    // MIDI CC
    pub controller_slot: Option<u32>,

    // Note
    pub root_note: u32,
    pub note_range: u32,

    // Pitch Tracker
    pub pitch_tracker_lowest: u32,
    pub pitch_tracker_root: u32,
    pub pitch_tracker_highest: u32,
    pub pitch_tracker_sensitivity: Ratio,

    // Random
    pub random_jitter: f32,
    pub random_smooth: f32,
    pub random_chaos: f32,

    // Slew Limiter
    pub slew_limiter_attack: Time,
    pub slew_limiter_decay: Time,
    pub slew_limiter_linked: bool,

    // Which group contains this modulator. `GROUP_ID_NONE` if it is not
    // contained in a group.
    pub group_id: GroupId,
}

impl Default for ModulatorBlock {
    fn default() -> Self {
        Self {
            mode: ModulatorMode::Blank,
            id: 0,
            enabled: true,
            minimized: false,
            output_range: OutputRange::Unipolar,
            loop_mode: LoopMode::Infinite,
            input_a: 0.0,
            input_b: 0.0,
            multiplier: 1.0,
            depth: Ratio::new::<ratio>(1.0),
            retrigger: true,
            one_shot: false,
            rate: Rate {
                sync: false,
                frequency: Frequency::new::<hertz>(1.0),
                numerator: 4,
                denominator: NoteValue::Sixteenth,
            },
            velocity_trigger_mode: VelocityTriggerMode::Strike,
            note_trigger_mode: NoteTriggerMode::Auto,
            trigger_threshold: 0.5,
            voice_mode: VoiceMode::Unison,
            phase_offset: Ratio::zero(),
            envelope: Default::default(),

            // Curve
            curve_time: Time::new::<second>(1.0),

            // LFO
            shape: Vec::new(),
            shape_name: None,
            shape_path: None,
            shape_edited: false,

            // LFO Table
            lfo_table_smooth: Ratio::new::<percent>(0.05),
            lfo_table_frame: 0.0,
            lfo_table_wavetable_contents: Vec::new(),
            lfo_table_wavetable_path: None,

            controller_slot: None,
            note_range: 120,
            root_note: midi!(A, 4).into_byte() as u32,

            // Pitch tracker
            pitch_tracker_lowest: midi!(C, 2).into_byte() as u32,
            pitch_tracker_root: midi!(A, 4).into_byte() as u32,
            pitch_tracker_highest: midi!(C, 6).into_byte() as u32,
            pitch_tracker_sensitivity: Ratio::zero(),

            // Random
            random_jitter: 0.0,
            random_smooth: 0.0,
            random_chaos: 1.0,

            // Slew limiter
            slew_limiter_attack: Time::new::<millisecond>(100.0),
            slew_limiter_decay: Time::new::<millisecond>(100.0),
            slew_limiter_linked: true,

            gain: Decibels::ZERO,
            audio_source: AudioSourceId::default(),
            metering_mode: MeteringMode::RootMeanSquared,
            group_id: GROUP_ID_NONE,
        }
    }
}

impl ModulatorBlock {
    pub(crate) fn read_data_block<R: Read + Seek>(
        &mut self,
        reader: &mut PhasePlantReader<R>,
        header: &DataBlockHeader,
    ) -> io::Result<()> {
        match self.mode {
            ModulatorMode::Curve | ModulatorMode::Lfo | ModulatorMode::Remap => {
                let point_count = reader.read_u32()?;
                trace!(
                    "data block: curve points count {point_count}, pos {}",
                    reader.pos()
                );

                self.shape = Vec::with_capacity(point_count as usize);
                for _ in 0..point_count {
                    self.shape.push(CurvePoint {
                        x: reader.read_f32()?,
                        y: reader.read_f32()?,
                        curve_x: reader.read_f32()?,
                        curve_y: reader.read_f32()?,
                        mode: CurvePointMode::from_id(reader.read_u32()?)?,
                    });
                }
            }

            // Some factory presets like Keys/Purple Organ have a data block
            // for the Blank modulator. It's likely a left over from an issue
            // with an earlier version of the Phase Plant.
            ModulatorMode::Blank => {
                reader.skip(header.data_length as i64)?;
            }

            _ => warn!(
                "Unhandled {} data block at position {}",
                self.mode,
                reader.pos()
            ),
        }
        Ok(())
    }
}

//
// Conversion of modulators to and from blocks
//

impl From<&ModulatorBlock> for AudioFollowerModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            depth: block.depth,
            output_range: block.output_range,
            gain: block.gain,
            attack_time: block.envelope.attack,
            release_time: block.envelope.release,
            audio_source: block.audio_source.clone(),
            metering_mode: block.metering_mode,
        }
    }
}

impl From<&AudioFollowerModulator> for ModulatorBlock {
    fn from(modulator: &AudioFollowerModulator) -> Self {
        Self {
            mode: modulator.mode(),
            depth: modulator.depth,
            output_range: modulator.output_range,
            gain: modulator.gain,
            envelope: Envelope {
                delay: Default::default(),
                attack: modulator.attack_time,
                attack_curve: 0.0,
                hold: Default::default(),
                decay: Default::default(),
                decay_falloff: 0.0,
                sustain: Default::default(),
                release: modulator.release_time,
                release_falloff: 0.0,
            },
            audio_source: modulator.audio_source.clone(),
            metering_mode: modulator.metering_mode,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for BlankModulator {
    fn from(_block: &ModulatorBlock) -> Self {
        BlankModulator {}
    }
}

impl From<&BlankModulator> for ModulatorBlock {
    fn from(modulator: &BlankModulator) -> Self {
        Self {
            mode: modulator.mode(),

            // The blank modulator has slightly different defaults than
            // the other modulators, even the ones that don't use these fields.
            envelope: Envelope {
                decay: Time::new::<second>(0.001),
                ..Default::default()
            },

            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for CurveModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            loop_mode: block.loop_mode,
            rate: Rate {
                frequency: block.curve_time.recip(),
                ..block.rate.clone()
            },
            note_trigger_mode: block.note_trigger_mode,
            trigger_threshold: block.trigger_threshold,
            depth: block.depth,
            shape: block.shape.clone(),
            shape_name: block.shape_name.clone(),
            shape_path: block.shape_path.clone(),
            shape_edited: block.shape_edited,
        }
    }
}

impl From<&CurveModulator> for ModulatorBlock {
    fn from(modulator: &CurveModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            loop_mode: modulator.loop_mode,
            rate: modulator.rate.clone(),
            curve_time: modulator.rate.frequency.recip(),
            note_trigger_mode: modulator.note_trigger_mode,
            depth: modulator.depth,
            shape: modulator.shape.clone(),
            shape_name: modulator.shape_name.clone(),
            shape_path: modulator.shape_path.clone(),
            shape_edited: modulator.shape_edited,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for EnvelopeModulator {
    fn from(block: &ModulatorBlock) -> Self {
        EnvelopeModulator {
            envelope: block.envelope.clone(),
            depth: block.depth,
        }
    }
}

impl From<&EnvelopeModulator> for ModulatorBlock {
    fn from(modulator: &EnvelopeModulator) -> Self {
        Self {
            mode: modulator.mode(),
            envelope: modulator.envelope.clone(),
            depth: modulator.depth,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for Group {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            name: block.shape_name.clone(),
        }
    }
}

impl From<&Group> for ModulatorBlock {
    fn from(modulator: &Group) -> Self {
        Self {
            shape_name: modulator.name.clone(),
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for LfoModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            depth: block.depth,
            loop_mode: block.loop_mode,
            rate: block.rate.clone(),
            note_trigger_mode: block.note_trigger_mode,
            trigger_threshold: block.trigger_threshold,
            phase_offset: block.phase_offset,
            shape: block.shape.clone(),
            shape_name: block.shape_name.clone(),
            shape_path: block.shape_path.clone(),
            shape_edited: block.shape_edited,
        }
    }
}

impl From<&LfoModulator> for ModulatorBlock {
    fn from(modulator: &LfoModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            depth: modulator.depth,
            loop_mode: modulator.loop_mode,
            rate: modulator.rate.clone(),
            note_trigger_mode: modulator.note_trigger_mode,
            phase_offset: modulator.phase_offset,

            // The LFO Modulator doesn't use an envelope but the preset files
            // still contain an envelope. Overriding the default makes the files
            // exactly match.
            envelope: Envelope {
                decay: Time::new::<second>(0.1), // From Phase Plant 2.0.13
                ..Default::default()
            },

            shape: modulator.shape.clone(),
            shape_name: modulator.shape_name.clone(),
            shape_path: modulator.shape_path.clone(),
            shape_edited: modulator.shape_edited,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for LfoTableModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            depth: block.depth,
            rate: block.rate.clone(),
            loop_mode: block.loop_mode,
            note_trigger_mode: block.note_trigger_mode,
            trigger_threshold: block.trigger_threshold,
            phase_offset: block.phase_offset,
            smooth: block.lfo_table_smooth,
            frame: block.lfo_table_frame,
            wavetable_contents: block.lfo_table_wavetable_contents.clone(),
            wavetable_name: block.shape_name.clone(),
            wavetable_path: block.lfo_table_wavetable_path.clone(),
        }
    }
}

impl From<&LfoTableModulator> for ModulatorBlock {
    fn from(modulator: &LfoTableModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            depth: modulator.depth,
            rate: modulator.rate.clone(),
            loop_mode: modulator.loop_mode,
            note_trigger_mode: modulator.note_trigger_mode,
            phase_offset: modulator.phase_offset,
            lfo_table_smooth: modulator.smooth,
            lfo_table_frame: modulator.frame,
            shape_name: modulator.wavetable_name.clone(),
            lfo_table_wavetable_contents: modulator.wavetable_contents.clone(),
            lfo_table_wavetable_path: modulator.wavetable_path.clone(),
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for LowerLimitModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            input_a: block.input_a,
            input_b: block.input_b,
            depth: block.depth,
        }
    }
}

impl From<&LowerLimitModulator> for ModulatorBlock {
    fn from(modulator: &LowerLimitModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            input_a: modulator.input_a,
            input_b: modulator.input_b,
            depth: modulator.depth,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for MidiCcModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            depth: block.depth,
            controller_slot: block.controller_slot,
        }
    }
}

impl From<&MidiCcModulator> for ModulatorBlock {
    fn from(modulator: &MidiCcModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            depth: modulator.depth,
            controller_slot: modulator.controller_slot,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for MpeTimbreModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            depth: block.depth,
        }
    }
}

impl From<&MpeTimbreModulator> for ModulatorBlock {
    fn from(modulator: &MpeTimbreModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            depth: modulator.depth,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for NoteModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            depth: block.depth,
            note_range: block.note_range,
            root_note: block.root_note,
        }
    }
}

impl From<&NoteModulator> for ModulatorBlock {
    fn from(modulator: &NoteModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            depth: modulator.depth,
            root_note: modulator.root_note,
            note_range: modulator.note_range,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for NoteGateModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            depth: block.depth,
        }
    }
}

impl From<&NoteGateModulator> for ModulatorBlock {
    fn from(modulator: &NoteGateModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            depth: modulator.depth,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for PitchTrackerModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            depth: block.depth,
            output_range: block.output_range,
            audio_source: block.audio_source.clone(),
            lowest_note: block.pitch_tracker_lowest,
            root_note: block.pitch_tracker_root,
            highest_note: block.pitch_tracker_highest,
            sensitivity: block.pitch_tracker_sensitivity,
        }
    }
}

impl From<&PitchTrackerModulator> for ModulatorBlock {
    fn from(modulator: &PitchTrackerModulator) -> Self {
        Self {
            mode: modulator.mode(),
            depth: modulator.depth,
            output_range: modulator.output_range,
            audio_source: modulator.audio_source.clone(),
            pitch_tracker_lowest: modulator.lowest_note,
            pitch_tracker_root: modulator.root_note,
            pitch_tracker_highest: modulator.highest_note,
            pitch_tracker_sensitivity: modulator.sensitivity,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for PitchWheelModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            depth: block.depth,
            output_range: block.output_range,
        }
    }
}

impl From<&PitchWheelModulator> for ModulatorBlock {
    fn from(modulator: &PitchWheelModulator) -> Self {
        Self {
            mode: modulator.mode(),
            depth: modulator.depth,
            output_range: modulator.output_range,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for PressureModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            depth: block.depth,
            output_range: block.output_range,
        }
    }
}

impl From<&PressureModulator> for ModulatorBlock {
    fn from(modulator: &PressureModulator) -> Self {
        Self {
            mode: modulator.mode(),
            depth: modulator.depth,
            output_range: modulator.output_range,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for RemapModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            bipolar: block.output_range == OutputRange::Bipolar,
            depth: block.depth,
            shape: block.shape.clone(),
            shape_name: block.shape_name.clone(),
            shape_path: block.shape_path.clone(),
            shape_edited: block.shape_edited,
        }
    }
}

impl From<&RemapModulator> for ModulatorBlock {
    fn from(modulator: &RemapModulator) -> Self {
        Self {
            mode: modulator.mode(),
            depth: modulator.depth,
            output_range: if modulator.bipolar {
                OutputRange::Bipolar
            } else {
                OutputRange::Unipolar
            },
            shape: modulator.shape.clone(),
            shape_name: modulator.shape_name.clone(),
            shape_path: modulator.shape_path.clone(),
            shape_edited: modulator.shape_edited,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for SampleAndHoldModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            depth: block.depth,
            note_trigger_mode: block.note_trigger_mode,
            trigger_threshold: block.trigger_threshold,
            input_a: block.input_a,
            input_b: block.input_b,
        }
    }
}

impl From<&SampleAndHoldModulator> for ModulatorBlock {
    fn from(modulator: &SampleAndHoldModulator) -> Self {
        Self {
            mode: modulator.mode(),
            depth: modulator.depth,
            note_trigger_mode: modulator.note_trigger_mode,
            trigger_threshold: modulator.trigger_threshold,
            input_a: modulator.input_a,
            input_b: modulator.input_b,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for SlewLimiterModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            attack: block.slew_limiter_attack,
            decay: block.slew_limiter_decay,
            linked: block.slew_limiter_linked,
        }
    }
}

impl From<&SlewLimiterModulator> for ModulatorBlock {
    fn from(modulator: &SlewLimiterModulator) -> Self {
        Self {
            mode: modulator.mode(),
            slew_limiter_attack: modulator.attack,
            slew_limiter_decay: modulator.decay,
            slew_limiter_linked: modulator.linked,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for RandomModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            depth: block.depth,
            rate: block.rate.clone(),
            jitter: block.random_jitter,
            smooth: block.random_smooth,
            chaos: block.random_chaos,
            note_trigger_mode: block.note_trigger_mode,
            trigger_threshold: block.trigger_threshold,
            voice_mode: block.voice_mode,
        }
    }
}

impl From<&RandomModulator> for ModulatorBlock {
    fn from(modulator: &RandomModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            depth: modulator.depth,
            rate: modulator.rate.clone(),
            random_jitter: modulator.jitter,
            random_smooth: modulator.smooth,
            random_chaos: modulator.chaos,
            note_trigger_mode: modulator.note_trigger_mode,
            trigger_threshold: modulator.trigger_threshold,
            voice_mode: modulator.voice_mode,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for ScaleModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            depth: block.depth,
            input_a: block.input_a,
            input_b: block.input_b,
            multiplier: block.multiplier,
        }
    }
}

impl From<&ScaleModulator> for ModulatorBlock {
    fn from(modulator: &ScaleModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            depth: modulator.depth,
            input_a: modulator.input_a,
            input_b: modulator.input_b,
            multiplier: modulator.multiplier,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for UpperLimitModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            output_range: block.output_range,
            input_a: block.input_a,
            input_b: block.input_b,
            depth: block.depth,
        }
    }
}

impl From<&UpperLimitModulator> for ModulatorBlock {
    fn from(modulator: &UpperLimitModulator) -> Self {
        Self {
            mode: modulator.mode(),
            output_range: modulator.output_range,
            input_a: modulator.input_a,
            input_b: modulator.input_b,
            depth: modulator.depth,
            ..Default::default()
        }
    }
}

impl From<&ModulatorBlock> for VelocityModulator {
    fn from(block: &ModulatorBlock) -> Self {
        Self {
            depth: block.depth,
            output_range: block.output_range,
            trigger_mode: block.velocity_trigger_mode,
        }
    }
}

impl From<&VelocityModulator> for ModulatorBlock {
    fn from(modulator: &VelocityModulator) -> Self {
        Self {
            mode: modulator.mode(),
            depth: modulator.depth,
            output_range: modulator.output_range,
            velocity_trigger_mode: modulator.trigger_mode,
            ..Default::default()
        }
    }
}
