//! Modulators create control signals.
//!
//! Each modulator is assigned 100 bytes to store their data.  They must read or
//! skip the entire amount.

// FIXME: From manual: "By clicking the little blue triangle that you can find
// at the top right of most modulator modules you can change its output range.
// The available options are unipolar (0 to 1), bipolar (−1 to 1) and inverted (1 to 0).

use std::any::Any;
use std::fmt::{Display, Formatter};

use downcast_rs::{impl_downcast, Downcast};
use strum_macros::FromRepr;

use crate::*;

pub use self::audio_follower::*;
pub use self::blank::*;
pub use self::curve::*;
pub use self::envelope::*;
pub use self::group::*;
pub use self::lfo::*;
pub use self::lfo_table::*;
pub use self::limits::*;
pub use self::midi_cc::*;
pub use self::mpe_timbre::*;
pub use self::note::*;
pub use self::note_gate::*;
pub use self::pitch_tracker::*;
pub use self::pitch_wheel::*;
pub use self::pressure::*;
pub use self::random::*;
pub use self::remap::*;
pub use self::sample_and_hold::*;
pub use self::scale::*;
pub use self::slew_limiter::*;
pub use self::velocity::*;

mod audio_follower;
mod blank;
mod curve;
mod envelope;
mod group;
mod lfo;
mod lfo_table;
mod limits;
mod midi_cc;
mod mpe_timbre;
mod note;
mod note_gate;
mod pitch_tracker;
mod pitch_wheel;
mod pressure;
mod random;
mod remap;
mod sample_and_hold;
mod scale;
mod slew_limiter;
mod velocity;

pub type GroupId = u32;

// TODO: Replace most usages by using Option<GroupId>
pub const GROUP_ID_NONE: GroupId = 0xFFFFFFFF; // Would be -1 if signed

pub type ModulatorId = u8;

/// The bipolar, unipolar and inverted output range options in the modulator
/// interface in Phase Plant do not change the behavior. These options are
/// shortcuts for setting the high, medium and low depths.
#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum OutputRange {
    // The discriminants correspond to the file format.
    /// From 0 to 1
    Unipolar = 0,
    /// -1 to +1
    Bipolar = 1,
    /// 1 to 0
    Inverted = 2,
}

impl OutputRange {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown output range mode {id}"),
            )
        })
    }

    pub fn symbol(&self) -> char {
        match self {
            OutputRange::Unipolar => '+',
            OutputRange::Bipolar => '±',
            OutputRange::Inverted => '-',
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, FromRepr)]
#[repr(u32)]
pub enum ModulatorMode {
    // The discriminants correspond to the file format.
    AudioFollower = 12,
    Blank = 0,
    Curve = 21,
    Envelope = 1,
    Group = 19,
    Lfo = 2,
    LfoTable = 20,
    #[doc(alias = "Max")]
    LowerLimit = 9,
    MidiCc = 16,
    MpeTimbre = 18,
    Remap = 17,
    SampleAndHold = 11,
    #[doc(alias = "Multiply")]
    Scale = 7,
    Note = 3,
    NoteGate = 13,
    PitchTracker = 14,
    PitchWheel = 15,
    Pressure = 5,
    Random = 8,
    SlewLimiter = 22,
    #[doc(alias = "Min")]
    UpperLimit = 10,
    Velocity = 4,
}

impl ModulatorMode {
    pub(crate) fn is_blank(&self) -> bool {
        self == &ModulatorMode::Blank
    }

    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown modulator mode {id}"),
            )
        })
    }
}

impl Display for ModulatorMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ModulatorMode::*;
        let name = match self {
            AudioFollower => "Audio Follower",
            Blank => "Blank",
            Curve => "Curve",
            Envelope => "Envelope",
            Group => "Group",
            Lfo => "LFO",
            LfoTable => "LFO Table",
            LowerLimit => "Lower Limit",
            MidiCc => "MIDI CC",
            MpeTimbre => "MPE Timbre",
            Note => "Note",
            NoteGate => "Note Gate",
            PitchTracker => "Pitch Tracker",
            PitchWheel => "Pitch Wheel",
            Pressure => "Pressure",
            Random => "Random",
            Remap => "Remap",
            SampleAndHold => "Sample & Hold",
            Scale => "Scale",
            SlewLimiter => "Slew Limiter",
            UpperLimit => "Upper Limit",
            Velocity => "Velocity",
        };
        f.write_str(name)
    }
}

/// Similar to a [`Snapin`] but for modulators instead of generators.
#[derive(Debug)]
pub struct ModulatorContainer {
    /// Identifier for the contained modulator.
    pub id: ModulatorId,

    pub group_id: GroupId,
    pub enabled: bool,
    pub minimized: bool,
    pub modulator: Box<dyn Modulator>,
}

impl ModulatorContainer {
    pub fn new(id: ModulatorId, modulator: Box<dyn Modulator>) -> Self {
        Self {
            id,
            group_id: GROUP_ID_NONE,
            enabled: true,
            minimized: false,
            modulator,
        }
    }
}

impl Eq for ModulatorContainer {}

impl PartialEq for ModulatorContainer {
    fn eq(&self, other: &Self) -> bool {
        self.enabled == other.enabled
            && self.minimized == other.minimized

            // Using .eq() instead of == quiets a Clippy warning about
            // unnecessary dereferencing.
            && self.modulator.eq(&other.modulator)
    }
}

pub trait Modulator: Downcast + std::fmt::Debug {
    fn as_block(&self) -> ModulatorBlock;
    fn box_eq(&self, other: &dyn Any) -> bool;
    fn mode(&self) -> ModulatorMode;
}

impl_downcast!(Modulator);

impl PartialEq for Box<dyn Modulator> {
    fn eq(&self, other: &Box<dyn Modulator>) -> bool {
        self.box_eq(other.as_any())
    }
}

impl Preset {
    pub fn modulator_container(&self, mod_index: usize) -> Option<&ModulatorContainer> {
        self.modulator_containers.get(mod_index)
    }
    pub fn modulator<T: Modulator>(&self, mod_index: usize) -> Option<&T> {
        self.modulator_containers
            .get(mod_index)?
            .modulator
            .downcast_ref::<T>()
    }
}
