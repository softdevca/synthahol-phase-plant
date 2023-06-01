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

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::effect::SidechainMode;
use crate::Decibels;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Copy, Clone, Debug, EnumIter, Eq, PartialEq)]
#[repr(u32)]
pub enum CompressorMode {
    // The discriminants correspond to the file format.
    #[doc(alias = "RMS")]
    RootMeanSquared = 0,
    Peak = 1,
    Fast = 2,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Compressor {
    pub mode: CompressorMode,
    pub threshold: f32,
    pub ratio: f32,
    pub attack: f32,
    pub release: f32,
    pub makeup: f32,
    pub sidechain_mode: SidechainMode,
}

impl Default for Compressor {
    fn default() -> Self {
        Compressor {
            mode: CompressorMode::Peak,
            threshold: Decibels::new(-6.0).linear(),
            ratio: 2.0,
            attack: 0.023,
            release: 0.023,
            makeup: 0.0,
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
        let attack = reader.read_f32()?;
        let release = reader.read_f32()?;

        let mode_id = reader.read_u32()?;
        let mode_opt = CompressorMode::iter().find(|mode| *mode as u32 == mode_id);
        let mode = match mode_opt {
            Some(mode) => mode,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Compressor mode {mode_id} not found"),
                ));
            }
        };

        let ratio = reader.read_f32()?;
        let threshold = reader.read_f32()?;
        let makeup = reader.read_f32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "compressor_unknown1")?;
        reader.expect_u32(0, "compressor_unknown2")?;

        if effect_version > 1039 {
            reader.expect_u32(0, "compressor_unknown3")?;
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
        ))
    }
}

impl EffectWrite for Compressor {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_f32(self.attack)?;
        writer.write_f32(self.release)?;
        writer.write_u32(self.mode as u32)?;
        writer.write_f32(self.ratio)?;
        writer.write_f32(self.threshold)?;
        writer.write_f32(self.makeup)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        writer.write_u32(0)?;

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

    use crate::effect::Filter;
    use crate::test::read_effect_preset;
    use crate::Decibels;

    use super::*;

    #[test]
    fn default() {
        let effect = Compressor::default();
        assert_eq!(effect.mode, CompressorMode::Peak);
        assert_relative_eq!(
            effect.threshold,
            Decibels::new(-6.0).linear(),
            epsilon = 0.001
        );
        assert_relative_eq!(effect.release, 0.023, epsilon = 0.001);
        assert_relative_eq!(effect.attack, 0.023, epsilon = 0.001);
        assert_relative_eq!(effect.ratio, 2.0);
        assert_relative_eq!(effect.makeup, 0.0);
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
            "compressor-1.8.5.phaseplant",
            "compressor-1.8.13.phaseplant",
            "compressor-2.0.12.phaseplant",
        ] {
            let preset = read_effect_preset("compressor", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            let effect = snapin.effect.as_compressor().unwrap();
            assert_eq!(effect.mode, CompressorMode::Peak);
            assert_relative_eq!(
                effect.threshold,
                Decibels::new(-6.0).linear(),
                epsilon = 0.001
            );
            assert_relative_eq!(effect.release, 0.023, epsilon = 0.001);
            assert_relative_eq!(effect.attack, 0.023, epsilon = 0.001);
            assert_relative_eq!(effect.ratio, 2.0);
            assert_relative_eq!(effect.makeup, 0.0);
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
        assert_eq!(effect.attack, 0.011);
        assert_eq!(effect.release, 0.022);

        let preset =
            read_effect_preset("compressor", "compressor-brick_wall-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Brickwall");
        assert_eq!(snapin.preset_path, vec!["factory", "Brickwall.kscp"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_compressor().unwrap();
        assert_relative_eq!(effect.makeup, 0.35, epsilon = 0.01);

        let preset =
            read_effect_preset("compressor", "compressor-makeup25%-fast-1.8.13.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_compressor().unwrap();
        assert_eq!(effect.makeup, 0.25);
        assert_eq!(effect.mode, CompressorMode::Fast);

        let preset = read_effect_preset(
            "compressor",
            "compressor-thresh2-ratio5to1-sideband-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_compressor().unwrap();
        assert_relative_eq!(effect.threshold, Decibels::new(2.0).linear());
        assert_eq!(effect.mode, CompressorMode::Peak);
        assert_eq!(effect.ratio, 5.0);
        assert_eq!(effect.sidechain_mode, SidechainMode::Sideband);
    }
}
