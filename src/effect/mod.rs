//! Effects live in snapins.
//!
//! Each snapin knows the version of the host (such as Phase Plant) that was
//! used to save the preset and an independent version number for the effect.

use std::any::Any;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek};

use downcast_rs::{impl_downcast, Downcast};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, FromRepr};

use crate::io::effects::{EffectRead, EffectReadReturn};
use crate::PhasePlantReader;

pub use self::bitcrush::*;
pub use self::carve_eq::*;
pub use self::channel_mixer::*;
pub use self::chorus::*;
pub use self::comb_filter::*;
pub use self::compressor::*;
pub use self::convolver::*;
pub use self::delay::*;
pub use self::disperser::*;
pub use self::distortion::*;
pub use self::dual_delay::*;
pub use self::dynamics::*;
pub use self::ensemble::*;
pub use self::faturator::*;
pub use self::filter::*;
pub use self::flanger::*;
pub use self::formant_filter::*;
pub use self::frequency_shifter::*;
pub use self::gain::*;
pub use self::gate::*;
pub use self::group::*;
pub use self::haas::*;
pub use self::ladder_filter::*;
pub use self::limiter::*;
pub use self::multipass::Multipass;
pub use self::nonlinear_filter::*;
pub use self::phase_distortion::*;
pub use self::phaser::*;
pub use self::pitch_shifter::*;
pub use self::resonator::*;
pub use self::reverb::*;
pub use self::reverser::*;
pub use self::ring_mod::*;
pub use self::slice_eq::*;
pub use self::snap_heap::*;
pub use self::stereo::*;
pub use self::tape_stop::*;
pub use self::three_band_eq::*;
pub use self::trance_gate::*;
pub use self::transient_shaper::*;

mod bitcrush;
mod carve_eq;
mod channel_mixer;
mod chorus;
mod comb_filter;
mod compressor;
mod convolver;
mod delay;
mod disperser;
mod distortion;
mod dual_delay;
mod dynamics;
mod ensemble;
mod faturator;
mod filter;
mod flanger;
mod formant_filter;
mod frequency_shifter;
mod gain;
mod gate;
mod group;
mod haas;
mod ladder_filter;
mod limiter;
mod multipass;
mod nonlinear_filter;
mod phase_distortion;
mod phaser;
mod pitch_shifter;
mod resonator;
mod reverb;
mod reverser;
mod ring_mod;
mod slice_eq;
mod snap_heap;
mod stereo;
mod tape_stop;
mod three_band_eq;
mod trance_gate;
mod transient_shaper;

pub type EffectVersion = u32;

pub trait Effect: Downcast + std::fmt::Debug {
    #[must_use]
    fn box_eq(&self, other: &dyn Any) -> bool;

    #[must_use]
    fn mode(&self) -> EffectMode;
}

impl_downcast!(Effect);

impl PartialEq for Box<dyn Effect> {
    fn eq(&self, other: &Box<dyn Effect>) -> bool {
        self.box_eq(other.as_any())
    }
}

impl Eq for Box<dyn Effect> {}

/// The discriminants are the four-byte ID stored in the preset file.
///
/// ```
/// use synthahol_phase_plant::effect::EffectMode;
///
/// assert_eq!(u32::from_le_bytes(*b"ksbc"), EffectMode::Bitcrush as u32);
/// ```
#[derive(Debug, FromRepr, PartialEq)]
#[repr(u32)]
pub enum EffectMode {
    Bitcrush = u32::from_le_bytes(*b"ksbc"),
    CarveEq = u32::from_le_bytes(*b"ksge"),
    ChannelMixer = u32::from_le_bytes(*b"kscm"),
    Chorus = u32::from_le_bytes(*b"ksch"),
    CombFilter = u32::from_le_bytes(*b"kscf"),
    Compressor = u32::from_le_bytes(*b"kscp"),
    Convolver = u32::from_le_bytes(*b"ksco"),
    Delay = u32::from_le_bytes(*b"ksdl"),
    Disperser = u32::from_le_bytes(*b"kdsp"),
    Distortion = u32::from_le_bytes(*b"ksdt"),
    DualDelay = u32::from_le_bytes(*b"ksdd"),
    Dynamics = u32::from_le_bytes(*b"ksot"),
    Ensemble = u32::from_le_bytes(*b"ksun"),
    Faturator = u32::from_le_bytes(*b"kfat"),
    Filter = u32::from_le_bytes(*b"ksfi"),
    Flanger = u32::from_le_bytes(*b"ksfl"),
    FormantFilter = u32::from_le_bytes(*b"ksvf"),
    FrequencyShifter = u32::from_le_bytes(*b"ksfs"),
    Gain = u32::from_le_bytes(*b"ksgn"),
    Gate = u32::from_le_bytes(*b"ksgt"),
    Group = u32::from_le_bytes(*b"grup"),
    Haas = u32::from_le_bytes(*b"ksha"),
    LadderFilter = u32::from_le_bytes(*b"ksla"),
    Limiter = u32::from_le_bytes(*b"kslt"),
    Multipass = u32::from_le_bytes(*b"kmup"),
    NonlinearFilter = u32::from_le_bytes(*b"ksdf"),
    PhaseDistortion = u32::from_le_bytes(*b"kspd"),
    Phaser = u32::from_le_bytes(*b"ksph"),
    PitchShifter = u32::from_le_bytes(*b"ksps"),
    Resonator = u32::from_le_bytes(*b"ksre"),
    Reverb = u32::from_le_bytes(*b"ksrv"),
    Reverser = u32::from_le_bytes(*b"ksrr"),
    RingMod = u32::from_le_bytes(*b"ksrm"),
    SliceEq = u32::from_le_bytes(*b"kpeq"),
    SnapHeap = u32::from_le_bytes(*b"kmic"),
    Stereo = u32::from_le_bytes(*b"ksst"),
    TapeStop = u32::from_le_bytes(*b"ksts"),
    ThreeBandEq = u32::from_le_bytes(*b"ksqe"),
    TranceGate = u32::from_le_bytes(*b"kstg"),
    TransientShaper = u32::from_le_bytes(*b"kstr"),
}

impl Display for EffectMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl EffectMode {
    pub fn default_version(&self) -> EffectVersion {
        match self {
            EffectMode::Bitcrush => Bitcrush::default_version(),
            EffectMode::CarveEq => CarveEq::default_version(),
            EffectMode::ChannelMixer => ChannelMixer::default_version(),
            EffectMode::Chorus => Chorus::default_version(),
            EffectMode::CombFilter => CombFilter::default_version(),
            EffectMode::Compressor => Compressor::default_version(),
            EffectMode::Convolver => Convolver::default_version(),
            EffectMode::Delay => Delay::default_version(),
            EffectMode::Disperser => Disperser::default_version(),
            EffectMode::Distortion => Distortion::default_version(),
            EffectMode::DualDelay => DualDelay::default_version(),
            EffectMode::Dynamics => Dynamics::default_version(),
            EffectMode::Ensemble => Ensemble::default_version(),
            EffectMode::Faturator => Faturator::default_version(),
            EffectMode::Filter => Filter::default_version(),
            EffectMode::Flanger => Flanger::default_version(),
            EffectMode::FormantFilter => FormantFilter::default_version(),
            EffectMode::FrequencyShifter => FrequencyShifter::default_version(),
            EffectMode::Gain => Gain::default_version(),
            EffectMode::Gate => Gate::default_version(),
            EffectMode::Group => Group::default_version(),
            EffectMode::Haas => Haas::default_version(),
            EffectMode::LadderFilter => LadderFilter::default_version(),
            EffectMode::Limiter => Limiter::default_version(),
            EffectMode::Multipass => Multipass::default_version(),
            EffectMode::NonlinearFilter => NonlinearFilter::default_version(),
            EffectMode::PhaseDistortion => PhaseDistortion::default_version(),
            EffectMode::Phaser => Phaser::default_version(),
            EffectMode::PitchShifter => PitchShifter::default_version(),
            EffectMode::Resonator => Resonator::default_version(),
            EffectMode::Reverb => Reverb::default_version(),
            EffectMode::Reverser => Reverser::default_version(),
            EffectMode::RingMod => RingMod::default_version(),
            EffectMode::SliceEq => SliceEq::default_version(),
            EffectMode::SnapHeap => SnapHeap::default_version(),
            EffectMode::Stereo => Stereo::default_version(),
            EffectMode::TapeStop => TapeStop::default_version(),
            EffectMode::ThreeBandEq => ThreeBandEq::default_version(),
            EffectMode::TranceGate => TranceGate::default_version(),
            EffectMode::TransientShaper => TransientShaper::default_version(),
        }
    }

    pub(crate) fn is_host(&self) -> bool {
        use EffectMode::*;
        match self {
            // Even though SliceEq doesn't contain any snapins it is stored as
            // in the preset as a host.
            SliceEq => true,
            CarveEq | Multipass | SnapHeap => true,
            _ => false,
        }
    }

    pub fn name(&self) -> &str {
        use EffectMode::*;
        match self {
            Bitcrush => "Bitcrush",
            CarveEq => "Carve EQ",
            ChannelMixer => "Channel Mixer",
            Chorus => "Chorus",
            CombFilter => "Comb Filter",
            Compressor => "Compressor",
            Convolver => "Convolver",
            Delay => "Delay",
            Disperser => "Disperser",
            Distortion => "Distortion",
            DualDelay => "Dual Delay",
            Dynamics => "Dynamics",
            Ensemble => "Ensemble",
            Faturator => "Faturator",
            Filter => "Filter",
            Flanger => "Flanger",
            FormantFilter => "Formant Filter",
            FrequencyShifter => "Frequency Shifter",
            Gain => "Gain",
            Gate => "Gate",
            Group => "Group",
            Haas => "Haas",
            LadderFilter => "Ladder Filter",
            Limiter => "Limiter",
            Multipass => "Multipass",
            NonlinearFilter => "Nonlinear Filter",
            PhaseDistortion => "Phase Distortion",
            Phaser => "Phaser",
            PitchShifter => "Pitch Shifter",
            Resonator => "Resonator",
            Reverb => "Reverb",
            Reverser => "Reverser",
            RingMod => "Ring Mod",
            SliceEq => "Slice EQ",
            SnapHeap => "Snap Heap",
            Stereo => "Stereo",
            TapeStop => "Tape Stop",
            ThreeBandEq => "3-Band EQ",
            TranceGate => "Trance Gate",
            TransientShaper => "Transient Shaper",
        }
    }
}

impl EffectMode {
    pub fn read_effect<R: Read + Seek>(
        &self,
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        match self {
            Self::Bitcrush => Bitcrush::read(reader, effect_version),
            Self::CarveEq => CarveEq::read(reader, effect_version),
            Self::ChannelMixer => ChannelMixer::read(reader, effect_version),
            Self::Chorus => Chorus::read(reader, effect_version),
            Self::CombFilter => CombFilter::read(reader, effect_version),
            Self::Compressor => Compressor::read(reader, effect_version),
            Self::Convolver => Convolver::read(reader, effect_version),
            Self::Delay => Delay::read(reader, effect_version),
            Self::Disperser => Disperser::read(reader, effect_version),
            Self::Distortion => Distortion::read(reader, effect_version),
            Self::DualDelay => DualDelay::read(reader, effect_version),
            Self::Dynamics => Dynamics::read(reader, effect_version),
            Self::Ensemble => Ensemble::read(reader, effect_version),
            Self::Faturator => Faturator::read(reader, effect_version),
            Self::Filter => Filter::read(reader, effect_version),
            Self::Flanger => Flanger::read(reader, effect_version),
            Self::FormantFilter => FormantFilter::read(reader, effect_version),
            Self::FrequencyShifter => FrequencyShifter::read(reader, effect_version),
            Self::Gain => Gain::read(reader, effect_version),
            Self::Gate => Gate::read(reader, effect_version),
            Self::Group => Group::read(reader, effect_version),
            Self::Haas => Haas::read(reader, effect_version),
            Self::LadderFilter => LadderFilter::read(reader, effect_version),
            Self::Limiter => Limiter::read(reader, effect_version),
            Self::Multipass => Multipass::read(reader, effect_version),
            Self::NonlinearFilter => NonlinearFilter::read(reader, effect_version),
            Self::PhaseDistortion => PhaseDistortion::read(reader, effect_version),
            Self::Phaser => Phaser::read(reader, effect_version),
            Self::PitchShifter => PitchShifter::read(reader, effect_version),
            Self::Resonator => Resonator::read(reader, effect_version),
            Self::Reverb => Reverb::read(reader, effect_version),
            Self::Reverser => Reverser::read(reader, effect_version),
            Self::RingMod => RingMod::read(reader, effect_version),
            Self::SliceEq => SliceEq::read(reader, effect_version),
            Self::SnapHeap => SnapHeap::read(reader, effect_version),
            Self::Stereo => Stereo::read(reader, effect_version),
            Self::TapeStop => TapeStop::read(reader, effect_version),
            Self::ThreeBandEq => ThreeBandEq::read(reader, effect_version),
            Self::TranceGate => TranceGate::read(reader, effect_version),
            Self::TransientShaper => TransientShaper::read(reader, effect_version),
        }
    }
}

/// The discriminants are an ID that precedes the name of the mode in the
/// preset file.
#[derive(Clone, Copy, Debug, EnumIter, Eq, PartialEq)]
#[repr(u32)]
pub enum SidechainMode {
    Off = 0xFFFFFFFF,
    Sideband = 0x73646230, // "sbd0" in little endian
}

impl SidechainMode {
    pub(crate) fn from_name(name: &str) -> Result<SidechainMode, Error> {
        match SidechainMode::iter().find(|mode| mode.to_string() == name) {
            Some(mode) => Ok(mode),
            None => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unknown sidechain mode '{name}'"),
            )),
        }
    }
}

impl Display for SidechainMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            SidechainMode::Off => "Off",
            SidechainMode::Sideband => "Sideband",
        };
        f.write_str(msg)
    }
}
