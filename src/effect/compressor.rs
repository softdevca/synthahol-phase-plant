//! [Compressor](https://kilohearts.com/products/compressor) #932.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1039           |
//! | 1.8.16              | 1039           |
//! | 2.0.12              | 1049           |
//! | 2.1.0               | 1050           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum_macros::FromRepr;
use uom::num::Zero;
use uom::si::f32::{Ratio, Time};
use uom::si::ratio::percent;
use uom::si::time::millisecond;

use crate::effect::SidechainMode;
use crate::{Decibels, SnapinId};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum CompressorMode {
    // The discriminants correspond to the file format.
    #[doc(alias = "RMS")]
    RootMeanSquared = 0,
    Peak = 1,
    Fast = 2,
}

impl CompressorMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown compressor mode {id}"),
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Compressor {
    pub mode: CompressorMode,
    pub threshold: Decibels,
    pub ratio: Ratio,
    pub attack: Time,
    pub release: Time,
    pub makeup: Ratio,
    pub sidechain_mode: SidechainMode,
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            mode: CompressorMode::Peak,
            threshold: Decibels::new(-6.0),
            ratio: Ratio::new::<percent>(200.0),
            attack: Time::new::<millisecond>(23.0),
            release: Time::new::<millisecond>(23.0),
            makeup: Ratio::zero(),
            sidechain_mode: SidechainMode::Off,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_compressor(&self) -> Option<&Compressor> {
        self.downcast_ref::<Compressor>()
    }
}

impl Effect for Compressor {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Compressor
    }
}

impl EffectRead for Compressor {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1039 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let enabled = reader.read_bool32()?;
        let attack = reader.read_seconds()?;
        let release = reader.read_seconds()?;
        let mode = CompressorMode::from_id(reader.read_u32()?)?;
        let ratio = reader.read_ratio()?;
        let threshold = reader.read_decibels_linear()?;
        let makeup = reader.read_ratio()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "compressor_unknown_1")?;
        reader.expect_u32(0, "compressor_unknown_2")?;

        let group_id = if effect_version > 1039 {
            reader.read_snapin_position()?
        } else {
            None
        };

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
            Box::new(Compressor {
                mode,
                threshold,
                ratio,
                attack,
                release,
                makeup,
                sidechain_mode,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Compressor {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
        group_id: Option<SnapinId>,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_seconds(self.attack)?;
        writer.write_seconds(self.release)?;
        writer.write_u32(self.mode as u32)?;
        writer.write_ratio(self.ratio)?;
        writer.write_decibels_linear(self.threshold)?;
        writer.write_ratio(self.makeup)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;

        writer.write_snapin_id(group_id)?;
        writer.write_u32(self.sidechain_mode as u32)?;
        writer.write_string_and_length(self.sidechain_mode.to_string())
    }

    fn write_version(&self) -> u32 {
        1049
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::{percent, ratio};

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = Compressor::default();
        assert_eq!(effect.mode, CompressorMode::Peak);
        assert_relative_eq!(effect.threshold.db(), -6.0, epsilon = 0.001);
        assert_relative_eq!(effect.release.get::<millisecond>(), 23.0, epsilon = 0.001);
        assert_relative_eq!(effect.attack.get::<millisecond>(), 023.0, epsilon = 0.001);
        assert_relative_eq!(effect.ratio.get::<ratio>(), 2.0);
        assert_relative_eq!(effect.makeup.get::<percent>(), 0.0);
        assert_eq!(effect.sidechain_mode, SidechainMode::Off);
    }

    #[test]
    fn disabled() {
        let preset =
            read_effect_preset("compressor", "compressor-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = Compressor::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Compressor::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "compressor-1.8.0.phaseplant",
            "compressor-1.8.5.phaseplant",
            "compressor-1.8.13.phaseplant",
            "compressor-2.0.12.phaseplant",
        ] {
            let preset = read_effect_preset("compressor", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            let effect = snapin.effect.as_compressor().unwrap();
            assert_eq!(effect.mode, CompressorMode::Peak);
            assert_relative_eq!(effect.threshold.db(), -6.0, epsilon = 0.01);
            assert_relative_eq!(effect.release.get::<millisecond>(), 23.0, epsilon = 0.001);
            assert_relative_eq!(effect.attack.get::<millisecond>(), 23.0, epsilon = 0.001);
            assert_relative_eq!(effect.ratio.get::<ratio>(), 2.0);
            assert_relative_eq!(effect.makeup.get::<percent>(), 0.0);
            assert_eq!(effect.sidechain_mode, SidechainMode::Off);
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("compressor", "compressor-minimized-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "compressor",
            "compressor-attack11-release22-rms-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.downcast_ref::<Compressor>().unwrap();
        assert_eq!(effect.mode, CompressorMode::RootMeanSquared);
        assert_relative_eq!(effect.attack.get::<millisecond>(), 11.0);
        assert_relative_eq!(effect.release.get::<millisecond>(), 22.0);

        let preset =
            read_effect_preset("compressor", "compressor-brick_wall-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Brickwall");
        assert_eq!(snapin.preset_path, vec!["factory", "Brickwall.kscp"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_compressor().unwrap();
        assert_relative_eq!(effect.makeup.get::<percent>(), 34.67, epsilon = 0.01);

        let preset =
            read_effect_preset("compressor", "compressor-makeup25%-fast-1.8.13.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_compressor().unwrap();
        assert_eq!(effect.makeup.get::<percent>(), 25.0);
        assert_eq!(effect.mode, CompressorMode::Fast);

        let preset = read_effect_preset(
            "compressor",
            "compressor-thresh2-ratio5to1-sideband-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_compressor().unwrap();
        assert_eq!(effect.mode, CompressorMode::Peak);
        assert_relative_eq!(effect.threshold.db(), 2.0, epsilon = 0.001);
        assert_relative_eq!(effect.ratio.get::<ratio>(), 5.0, epsilon = 0.001);
        assert_eq!(effect.sidechain_mode, SidechainMode::Sideband);
    }
}
