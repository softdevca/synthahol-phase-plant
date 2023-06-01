//! [Ladder Filter](https://kilohearts.com/products/ladder_filter) simulates
//! low pass filters found in classic synths.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1029           |
//! | 1.8.14              | 1029           |
//! | 2.0.16              | 1040           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::Decibels;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct LadderFilter {
    pub cutoff: f32,
    pub saturate: bool,
    pub resonance: f32,

    /// 0.0 to 45.0 dB
    pub drive: Decibels,

    pub bias: f32,

    /// If the filter is transistor or diode.
    pub diode: bool,
}

impl LadderFilter {
    pub const DRIVE_MIN: Decibels = Decibels::new(0.0);
    pub const DRIVE_MAX: Decibels = Decibels::new(45.0);
}

impl Default for LadderFilter {
    fn default() -> Self {
        Self {
            cutoff: 440.0,
            saturate: false,
            resonance: 0.0,
            drive: Decibels::ZERO,
            bias: 0.0,
            diode: false,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_ladder_filter(&self) -> Option<&LadderFilter> {
        self.downcast_ref::<LadderFilter>()
    }
}

impl Effect for LadderFilter {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::LadderFilter
    }
}

impl EffectRead for LadderFilter {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1029 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let cutoff = reader.read_f32()?;
        let resonance = reader.read_f32()?;

        let drive = Decibels::from_linear(reader.read_f32()?);
        if drive < LadderFilter::DRIVE_MIN {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Drive of {drive} is less than {}", LadderFilter::DRIVE_MIN),
            ));
        } else if drive > LadderFilter::DRIVE_MAX {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Drive of {drive} is greater than {}",
                    LadderFilter::DRIVE_MAX
                ),
            ));
        }

        let bias = reader.read_f32()?;
        let diode = reader.read_bool32()?;
        let saturate = reader.read_bool32()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "ladder_filter_unknown1")?;
        reader.expect_u32(0, "ladder_filter_unknown2")?;
        if effect_version >= 1040 {
            reader.expect_u32(0, "ladder_filter_unknown3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(LadderFilter {
                cutoff,
                saturate,
                resonance,
                drive,
                bias,
                diode,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for LadderFilter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_f32(self.cutoff)?;
        writer.write_f32(self.resonance)?;
        writer.write_f32(self.drive.linear())?;
        writer.write_f32(self.bias)?;
        writer.write_bool32(self.diode)?;
        writer.write_bool32(self.saturate)?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // ladder_filter_unknown1
        writer.write_u32(0)?; // ladder_filter_unknown2
        if self.write_version() >= 1040 {
            writer.write_u32(0)?; // ladder_filter_unknown3
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1040
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;
    use crate::Decibels;

    use super::*;

    #[test]
    fn default() {
        let effect = LadderFilter::default();
        assert_eq!(effect.cutoff, 440.0);
        assert_eq!(effect.resonance, 0.0);
        assert_eq!(effect.drive, Decibels::ZERO);
        assert_eq!(effect.bias, 0.0);
        assert!(!effect.saturate);
        assert!(!effect.diode);
    }

    #[test]
    fn disabled() {
        let preset =
            read_effect_preset("ladder_filter", "ladder_filter-disabled-1.8.14.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = LadderFilter::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, LadderFilter::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "ladder_filter-1.8.14.phaseplant",
            "ladder_filter-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("ladder_filter", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_ladder_filter().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("ladder_filter", "ladder_filter-minimized-1.8.14.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "ladder_filter",
            "ladder_filter-cutoff220-resonance80-diode-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_ladder_filter().unwrap();
        assert_relative_eq!(effect.cutoff, 220.0, epsilon = 0.01);
        assert_relative_eq!(effect.resonance, 0.80, epsilon = 0.01);
        assert!(effect.diode);

        let preset = read_effect_preset(
            "ladder_filter",
            "ladder_filter-drive5-bias15-saturate-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_ladder_filter().unwrap();
        assert_relative_eq!(effect.drive.db(), 5.0, epsilon = 0.0001);
        assert_relative_eq!(effect.bias, 0.15, epsilon = 0.0001);
        assert!(effect.saturate);
    }

    #[test]
    fn parts_version_2() {
        let preset = read_effect_preset(
            "ladder_filter",
            "ladder_filter-drive45-resonance65-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_ladder_filter().unwrap();
        assert_relative_eq!(effect.drive.db(), 45.0, epsilon = 0.1);
        assert_relative_eq!(effect.resonance, 0.65, epsilon = 0.01);
    }
}
