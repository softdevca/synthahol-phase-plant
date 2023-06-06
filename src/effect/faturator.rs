//! [Faturator](https://kilohearts.com/products/faturator) is a distortion
//! effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.6.9 to 1.8.13     | 1040           |
//! | 2.0.16 to 2.1.0     | 1051           |

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::{Frequency, Ratio};
use uom::si::frequency::hertz;
use uom::si::ratio::{percent, ratio};

use super::{Effect, EffectMode};
use super::super::io::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Faturator {
    pub drive: Ratio,
    pub fuzz: Ratio,
    pub color: Frequency,
    pub stereo_turbo: Ratio,
    pub mix: Ratio,
}

impl Default for Faturator {
    fn default() -> Self {
        Self {
            drive: Ratio::new::<percent>(51.8),
            fuzz: Ratio::new::<percent>(27.3),
            color: Frequency::new::<hertz>(50.0),
            stereo_turbo: Ratio::zero(),
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_faturator(&self) -> Option<&Faturator> {
        self.downcast_ref::<Faturator>()
    }
}

impl Effect for Faturator {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Faturator
    }
}

impl EffectRead for Faturator {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1040 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let drive = reader.read_ratio()?;
        let fuzz = reader.read_ratio()?;
        let stereo_turbo = reader.read_ratio()?;
        let color = reader.read_hertz()?;
        let mix = reader.read_ratio()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "faturator_unknown_1")?;
        reader.expect_u32(0, "faturator_unknown_2")?;
        if effect_version > 1040 {
            reader.expect_u32(0, "faturator_unknown_3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Faturator {
                drive,
                fuzz,
                color,
                stereo_turbo,
                mix,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Faturator {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_f32(self.drive.get::<ratio>())?;
        writer.write_f32(self.fuzz.get::<ratio>())?;
        writer.write_f32(self.stereo_turbo.get::<ratio>())?;
        writer.write_f32(self.color.get::<hertz>())?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // faturator_unknown_1
        writer.write_u32(0)?; // faturator_unknown_2
        writer.write_u32(0)?; // faturator_unknown_3
        Ok(())
    }

    fn write_version(&self) -> u32 {
        1051
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn bass_driver() {
        let preset =
            read_effect_preset("faturator", "faturator-bass_driver-1.7.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(
            snapin.preset_name,
            "C:\\ProgramData/Kilohearts\\presets\\kfat\\Factory Presets\\Bass Driver.kfat"
        );
        assert_eq!(snapin.preset_path, vec![""]);
        assert!(!snapin.preset_edited);

        for file in &[
            "faturator-bass_driver-1.8.13.phaseplant",
            "faturator-bass_driver-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("faturator", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert_eq!(snapin.preset_name, "Bass Driver");
            assert_eq!(snapin.preset_path, vec!["factory", "Bass Driver.kfat"]);
            assert!(!snapin.preset_edited);
        }
    }

    #[test]
    fn eq() {
        let effect = Faturator::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Faturator::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "faturator-1.7.0.phaseplant",
            "faturator-1.7.11.phaseplant",
            "faturator-1.8.13.phaseplant",
            "faturator-2.0.0.phaseplant",
            "faturator-2.0.16.phaseplant",
            "faturator-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("faturator", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_faturator().unwrap();

            // Displays as 52% in interface
            assert_relative_eq!(effect.drive.get::<percent>(), 51.8, epsilon = 0.0001);

            assert_relative_eq!(effect.fuzz.get::<percent>(), 27.3, epsilon = 0.0001);
            assert_eq!(effect.color.get::<hertz>(), 50.0);
            assert_eq!(effect.stereo_turbo.get::<percent>(), 0.0);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("faturator", "faturator-minimized-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "faturator",
            "faturator-drive25-fuzz66-color333-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_faturator().unwrap();
        assert_relative_eq!(effect.drive.get::<percent>(), 25.0, epsilon = 0.0001);
        assert_relative_eq!(effect.fuzz.get::<percent>(), 66.0, epsilon = 0.0001);
        assert_relative_eq!(effect.color.get::<hertz>(), 333.0);
        assert_eq!(effect.stereo_turbo.get::<percent>(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);

        let preset =
            read_effect_preset("faturator", "faturator-mix23-stereo-77-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_faturator().unwrap();
        assert_relative_eq!(effect.drive.get::<percent>(), 51.8, epsilon = 0.0001);
        assert_relative_eq!(effect.fuzz.get::<percent>(), 27.3, epsilon = 0.0001);
        assert_eq!(effect.color.get::<hertz>(), 50.0);
        assert_eq!(effect.stereo_turbo.get::<percent>(), -77.0);
        assert_eq!(effect.mix.get::<percent>(), 23.0);
    }
}
