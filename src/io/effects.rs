use std::io;
use std::io::{Read, Seek, Write};

use crate::effect::EffectMode;
use crate::*;

pub struct EffectReadReturn {
    pub effect: Box<dyn Effect>,
    pub enabled: bool,
    pub minimized: bool,
    pub metadata: Metadata,
    pub preset_name: Option<String>,
    pub preset_path: Vec<String>,
    pub preset_edited: bool,
}

impl EffectReadReturn {
    pub(crate) fn new(effect: Box<dyn Effect>, enabled: bool, minimized: bool) -> Self {
        Self {
            effect,
            enabled,
            minimized,
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
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()>;

    #[must_use]
    fn write_version(&self) -> u32;
}

impl dyn Effect {
    pub(crate) fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        use EffectMode::*;
        // Not the greatest fan of the lack of dynamic dispatch here.
        match self.mode() {
            Bitcrush => self
                .as_bitcrush()
                .unwrap()
                .write(writer, enabled, minimized),
            CarveEq => self
                .as_carve_eq()
                .unwrap()
                .write(writer, enabled, minimized),
            ChannelMixer => self
                .as_channel_mixer()
                .unwrap()
                .write(writer, enabled, minimized),
            Chorus => self.as_chorus().unwrap().write(writer, enabled, minimized),
            CombFilter => self
                .as_comb_filter()
                .unwrap()
                .write(writer, enabled, minimized),
            Compressor => self
                .as_compressor()
                .unwrap()
                .write(writer, enabled, minimized),
            Convolver => self
                .as_convolver()
                .unwrap()
                .write(writer, enabled, minimized),
            Delay => self.as_delay().unwrap().write(writer, enabled, minimized),
            Disperser => self
                .as_disperser()
                .unwrap()
                .write(writer, enabled, minimized),
            Distortion => self
                .as_distortion()
                .unwrap()
                .write(writer, enabled, minimized),
            DualDelay => self
                .as_dual_delay()
                .unwrap()
                .write(writer, enabled, minimized),
            Dynamics => self
                .as_dynamics()
                .unwrap()
                .write(writer, enabled, minimized),
            Ensemble => self
                .as_ensemble()
                .unwrap()
                .write(writer, enabled, minimized),
            Faturator => self
                .as_faturator()
                .unwrap()
                .write(writer, enabled, minimized),
            Filter => self.as_filter().unwrap().write(writer, enabled, minimized),
            Flanger => self.as_flanger().unwrap().write(writer, enabled, minimized),
            FormantFilter => self
                .as_formant_filter()
                .unwrap()
                .write(writer, enabled, minimized),
            FrequencyShifter => self
                .as_frequency_shifter()
                .unwrap()
                .write(writer, enabled, minimized),
            Gain => self.as_gain().unwrap().write(writer, enabled, minimized),
            Gate => self.as_gate().unwrap().write(writer, enabled, minimized),
            Group => self.as_group().unwrap().write(writer, enabled, minimized),
            Haas => self.as_haas().unwrap().write(writer, enabled, minimized),
            LadderFilter => self
                .as_ladder_filter()
                .unwrap()
                .write(writer, enabled, minimized),
            Limiter => self.as_limiter().unwrap().write(writer, enabled, minimized),
            Multipass => self
                .as_multipass()
                .unwrap()
                .write(writer, enabled, minimized),
            NonlinearFilter => self
                .as_nonlinear_filter()
                .unwrap()
                .write(writer, enabled, minimized),
            PhaseDistortion => self
                .as_phase_distortion()
                .unwrap()
                .write(writer, enabled, minimized),
            Phaser => self.as_phaser().unwrap().write(writer, enabled, minimized),
            PitchShifter => self
                .as_pitch_shifter()
                .unwrap()
                .write(writer, enabled, minimized),
            Resonator => self
                .as_resonator()
                .unwrap()
                .write(writer, enabled, minimized),
            Reverb => self.as_reverb().unwrap().write(writer, enabled, minimized),
            Reverser => self
                .as_reverser()
                .unwrap()
                .write(writer, enabled, minimized),
            RingMod => self
                .as_ring_mod()
                .unwrap()
                .write(writer, enabled, minimized),
            SliceEq => self
                .as_slice_eq()
                .unwrap()
                .write(writer, enabled, minimized),
            SnapHeap => self
                .as_snap_heap()
                .unwrap()
                .write(writer, enabled, minimized),
            Stereo => self.as_stereo().unwrap().write(writer, enabled, minimized),
            TapeStop => self
                .as_tape_stop()
                .unwrap()
                .write(writer, enabled, minimized),
            ThreeBandEq => self
                .as_three_band_eq()
                .unwrap()
                .write(writer, enabled, minimized),
            TranceGate => self
                .as_trance_gate()
                .unwrap()
                .write(writer, enabled, minimized),
            TransientShaper => self
                .as_transient_shaper()
                .unwrap()
                .write(writer, enabled, minimized),
        }
    }

    pub(crate) fn write_version(&self) -> u32 {
        use EffectMode::*;
        match self.mode() {
            Bitcrush => self.as_bitcrush().unwrap().write_version(),
            CarveEq => self.as_carve_eq().unwrap().write_version(),
            ChannelMixer => self.as_channel_mixer().unwrap().write_version(),
            Chorus => self.as_chorus().unwrap().write_version(),
            CombFilter => self.as_comb_filter().unwrap().write_version(),
            Compressor => self.as_compressor().unwrap().write_version(),
            Convolver => self.as_convolver().unwrap().write_version(),
            Delay => self.as_delay().unwrap().write_version(),
            Disperser => self.as_disperser().unwrap().write_version(),
            Distortion => self.as_distortion().unwrap().write_version(),
            DualDelay => self.as_dual_delay().unwrap().write_version(),
            Dynamics => self.as_dynamics().unwrap().write_version(),
            Ensemble => self.as_ensemble().unwrap().write_version(),
            Faturator => self.as_faturator().unwrap().write_version(),
            Filter => self.as_filter().unwrap().write_version(),
            Flanger => self.as_flanger().unwrap().write_version(),
            FormantFilter => self.as_formant_filter().unwrap().write_version(),
            FrequencyShifter => self.as_frequency_shifter().unwrap().write_version(),
            Gain => self.as_gain().unwrap().write_version(),
            Gate => self.as_gate().unwrap().write_version(),
            Group => self.as_group().unwrap().write_version(),
            Haas => self.as_haas().unwrap().write_version(),
            LadderFilter => self.as_ladder_filter().unwrap().write_version(),
            Limiter => self.as_limiter().unwrap().write_version(),
            Multipass => self.as_multipass().unwrap().write_version(),
            NonlinearFilter => self.as_nonlinear_filter().unwrap().write_version(),
            PhaseDistortion => self.as_phase_distortion().unwrap().write_version(),
            Phaser => self.as_phaser().unwrap().write_version(),
            PitchShifter => self.as_pitch_shifter().unwrap().write_version(),
            Resonator => self.as_resonator().unwrap().write_version(),
            Reverb => self.as_reverb().unwrap().write_version(),
            Reverser => self.as_reverser().unwrap().write_version(),
            RingMod => self.as_ring_mod().unwrap().write_version(),
            SliceEq => self.as_slice_eq().unwrap().write_version(),
            SnapHeap => self.as_snap_heap().unwrap().write_version(),
            Stereo => self.as_stereo().unwrap().write_version(),
            TapeStop => self.as_tape_stop().unwrap().write_version(),
            ThreeBandEq => self.as_three_band_eq().unwrap().write_version(),
            TranceGate => self.as_trance_gate().unwrap().write_version(),
            TransientShaper => self.as_transient_shaper().unwrap().write_version(),
        }
    }
}
