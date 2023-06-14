use std::io;
use std::io::{Read, Seek, Write};

use crate::effect::EffectMode;
use crate::*;

pub struct EffectReadReturn {
    pub effect: Box<dyn Effect>,
    pub enabled: bool,
    pub minimized: bool,
    pub group_id: Option<SnapinId>,
    pub metadata: Metadata,
    pub preset_name: Option<String>,
    pub preset_path: Vec<String>,
    pub preset_edited: bool,
}

impl EffectReadReturn {
    pub(crate) fn new(
        effect: Box<dyn Effect>,
        enabled: bool,
        minimized: bool,
        group_id: Option<SnapinId>,
    ) -> Self {
        Self {
            effect,
            enabled,
            minimized,
            group_id,
            metadata: Default::default(),
            preset_name: None,
            preset_path: vec![],
            preset_edited: false,
        }
    }
}

pub(crate) trait EffectRead {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn>;
}

pub(crate) trait EffectWrite {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()>;
}

impl dyn Effect {
    pub(crate) fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        use EffectMode::*;
        // Not the greatest fan of the lack of dynamic dispatch here.
        match self.mode() {
            Bitcrush => self.as_bitcrush().unwrap().write(writer, snapin),
            CarveEq => self.as_carve_eq().unwrap().write(writer, snapin),
            ChannelMixer => self.as_channel_mixer().unwrap().write(writer, snapin),
            Chorus => self.as_chorus().unwrap().write(writer, snapin),
            CombFilter => self.as_comb_filter().unwrap().write(writer, snapin),
            Compressor => self.as_compressor().unwrap().write(writer, snapin),
            Convolver => self.as_convolver().unwrap().write(writer, snapin),
            Delay => self.as_delay().unwrap().write(writer, snapin),
            Disperser => self.as_disperser().unwrap().write(writer, snapin),
            Distortion => self.as_distortion().unwrap().write(writer, snapin),
            DualDelay => self.as_dual_delay().unwrap().write(writer, snapin),
            Dynamics => self.as_dynamics().unwrap().write(writer, snapin),
            Ensemble => self.as_ensemble().unwrap().write(writer, snapin),
            Faturator => self.as_faturator().unwrap().write(writer, snapin),
            Filter => self.as_filter().unwrap().write(writer, snapin),
            Flanger => self.as_flanger().unwrap().write(writer, snapin),
            FormantFilter => self.as_formant_filter().unwrap().write(writer, snapin),
            FrequencyShifter => self.as_frequency_shifter().unwrap().write(writer, snapin),
            Gain => self.as_gain().unwrap().write(writer, snapin),
            Gate => self.as_gate().unwrap().write(writer, snapin),
            Group => self.as_group().unwrap().write(writer, snapin),
            Haas => self.as_haas().unwrap().write(writer, snapin),
            LadderFilter => self.as_ladder_filter().unwrap().write(writer, snapin),
            Limiter => self.as_limiter().unwrap().write(writer, snapin),
            Multipass => self.as_multipass().unwrap().write(writer, snapin),
            NonlinearFilter => self.as_nonlinear_filter().unwrap().write(writer, snapin),
            PhaseDistortion => self.as_phase_distortion().unwrap().write(writer, snapin),
            Phaser => self.as_phaser().unwrap().write(writer, snapin),
            PitchShifter => self.as_pitch_shifter().unwrap().write(writer, snapin),
            Resonator => self.as_resonator().unwrap().write(writer, snapin),
            Reverb => self.as_reverb().unwrap().write(writer, snapin),
            Reverser => self.as_reverser().unwrap().write(writer, snapin),
            RingMod => self.as_ring_mod().unwrap().write(writer, snapin),
            SliceEq => self.as_slice_eq().unwrap().write(writer, snapin),
            SnapHeap => self.as_snap_heap().unwrap().write(writer, snapin),
            Stereo => self.as_stereo().unwrap().write(writer, snapin),
            TapeStop => self.as_tape_stop().unwrap().write(writer, snapin),
            ThreeBandEq => self.as_three_band_eq().unwrap().write(writer, snapin),
            TranceGate => self.as_trance_gate().unwrap().write(writer, snapin),
            TransientShaper => self.as_transient_shaper().unwrap().write(writer, snapin),
        }
    }
}
