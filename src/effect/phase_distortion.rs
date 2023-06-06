//! [Phase Distortion](https://kilohearts.com/products/phase_distortion)
//! is a non-linear phase modulation effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.1.6    | 1023           |
//! | 2.0.16              | 1034           |

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::{Frequency, Ratio};
use uom::si::frequency::hertz;
use uom::si::ratio::percent;

use crate::effect::SidechainMode;

use super::{Effect, EffectMode};
use super::super::io::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PhaseDistortion {
    pub drive: f32,
    pub normalize: f32,

    pub tone: Frequency,

    /// Percentage of 360 degrees
    pub bias: Ratio,

    /// Percentage of 360 degrees
    pub spread: Ratio,

    pub mix: Ratio,
    pub sidechain_mode: SidechainMode,
}

impl dyn Effect {
    #[must_use]
    pub fn as_phase_distortion(&self) -> Option<&PhaseDistortion> {
        self.downcast_ref::<PhaseDistortion>()
    }
}

impl Effect for PhaseDistortion {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::PhaseDistortion
    }
}

impl EffectRead for PhaseDistortion {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1023 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let drive = reader.read_f32()?;
        let spread = Ratio::new::<percent>(reader.read_f32()?);
        let mix = reader.read_ratio()?;
        let normalize = reader.read_f32()?;
        let tone = reader.read_hertz()?;
        let bias = Ratio::new::<percent>(reader.read_f32()?);
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "phase_distortion_unknown1")?;
        reader.expect_u32(0, "phase_distortion_unknown2")?;
        if effect_version >= 1034 {
            reader.expect_u32(0, "phase_distortion_unknown3")?;
        }

        let sidechain_id = reader.read_u32()?;
        let sidechain_mode_str = reader.read_string_and_length()?;
        let sidechain_mode = SidechainMode::from_name(&sidechain_mode_str.unwrap_or_default())?;
        if sidechain_mode as u32 != sidechain_id {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Sidechain ID {sidechain_id:#x} does not match mode {sidechain_mode}"),
            ));
        }

        Ok(EffectReadReturn::new(
            Box::new(PhaseDistortion {
                drive,
                normalize,
                tone,
                bias,
                spread,
                mix,
                sidechain_mode,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for PhaseDistortion {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_f32(self.drive)?;
        writer.write_f32(self.spread.get::<percent>())?;
        writer.write_ratio(self.mix)?;
        writer.write_f32(self.normalize)?;
        writer.write_f32(self.tone.get::<hertz>())?;
        writer.write_f32(self.bias.get::<percent>())?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // phase_distortion_unknown1
        writer.write_u32(0)?; // phase_distortion_unknown2
        if self.write_version() >= 1034 {
            writer.write_u32(0)?; // phase_distortion_unknown3
        }

        writer.write_u32(self.sidechain_mode as u32)?;
        writer.write_string_and_length(self.sidechain_mode.to_string())
    }

    fn write_version(&self) -> u32 {
        1034
    }
}

impl Default for PhaseDistortion {
    fn default() -> Self {
        Self {
            drive: 0.5,
            normalize: 0.5,
            tone: Frequency::new::<hertz>(640.0),
            bias: Ratio::zero(),
            spread: Ratio::zero(),
            mix: Ratio::new::<percent>(100.0),
            sidechain_mode: SidechainMode::Off,
        }
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::num::Zero;
    use uom::si::f32::Ratio;
    use uom::si::frequency::hertz;
    use uom::si::ratio::percent;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = PhaseDistortion::default();
        assert_relative_eq!(effect.drive, 0.5);
        assert_relative_eq!(effect.normalize, 0.5);
        assert_relative_eq!(effect.tone.get::<hertz>(), 640.0);
        assert_eq!(effect.bias, Ratio::zero());
        assert_eq!(effect.spread, Ratio::zero());
        assert_relative_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(effect.sidechain_mode, SidechainMode::Off);
    }

    #[test]
    fn eq() {
        let effect = PhaseDistortion::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, PhaseDistortion::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    pub fn init() {
        for file in &[
            "phase_distortion-1.8.14.phaseplant",
            "phase_distortion-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("phase_distortion", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_phase_distortion().unwrap();
            assert_eq!(effect, &PhaseDistortion::default())
        }
    }

    #[test]
    pub fn parts_version_1() {
        let preset = read_effect_preset(
            "phase_distortion",
            "phase_distortion-drive25-normalize10-disabled-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_phase_distortion().unwrap();
        assert_relative_eq!(effect.drive, 0.25);
        assert_relative_eq!(effect.normalize, 0.1);

        let preset = read_effect_preset(
            "phase_distortion",
            "phase_distortion-spread25-mix50-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_phase_distortion().unwrap();
        assert_relative_eq!(effect.spread.get::<percent>(), 25.0 / 360.0);
        assert_relative_eq!(effect.mix.get::<percent>(), 50.0);

        let preset = read_effect_preset(
            "phase_distortion",
            "phase_distortion-tone25-bias10-minimized-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_phase_distortion().unwrap();
        assert_relative_eq!(effect.tone.get::<hertz>(), 25.0);
        assert_relative_eq!(effect.bias.get::<percent>(), 10.0 / 360.0);
    }

    #[test]
    pub fn sideband() {
        let preset = read_effect_preset(
            "phase_distortion",
            "phase_distortion-sideband-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_phase_distortion().unwrap();
        assert_eq!(effect.sidechain_mode, SidechainMode::Sideband);
    }
}
