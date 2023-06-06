//! [Stereo](https://kilohearts.com/products/stereo) is a width and panning
//! effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.13     | 1038           |
//! | 2.0.0               | 1047           |
//! | 2.0.16              | 1049           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::Ratio;
use uom::si::ratio::{percent, ratio};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Stereo {
    pub mid: Ratio,
    pub width: Ratio,
    pub pan: Ratio,
}

impl Default for Stereo {
    fn default() -> Self {
        Self {
            mid: Ratio::new::<percent>(100.0),
            width: Ratio::new::<percent>(100.0),
            pan: Ratio::zero(),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_stereo(&self) -> Option<&Stereo> {
        self.downcast_ref::<Stereo>()
    }
}

impl Effect for Stereo {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Stereo
    }
}

impl EffectRead for Stereo {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1038 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let enabled = reader.read_bool32()?;
        let width = reader.read_ratio()?;
        let pan = reader.read_ratio()?;
        let mid = reader.read_ratio()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "stereo_unknown_1")?;
        reader.expect_u32(0, "stereo_unknown_2")?;
        if effect_version >= 1047 {
            reader.expect_u32(0, "stereo_unknown_3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Stereo { mid, width, pan }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Stereo {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_f32(self.width.get::<ratio>())?;
        writer.write_f32(self.pan.get::<ratio>())?;
        writer.write_f32(self.mid.get::<ratio>())?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        writer.write_u32(0)?;
        if self.write_version() >= 1047 {
            writer.write_u32(0)?;
        }
        Ok(())
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

    use super::*;

    #[test]
    fn default() {
        let effect = Stereo::default();
        assert_eq!(effect.mid.get::<percent>(), 100.0);
        assert_eq!(effect.width.get::<percent>(), 100.0);
        assert_eq!(effect.pan.get::<percent>(), 0.0);
    }

    #[test]
    fn eq() {
        let effect = Stereo::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Stereo::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "stereo-1.7.7.phaseplant",
            "stereo-1.8.13.phaseplant",
            "stereo-2.0.0.phaseplant",
            "stereo-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("stereo", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_stereo().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset("stereo", "stereo-lane2-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[1].snapins[0];
        let effect = snapin.effect.as_stereo().unwrap();
        assert_eq!(effect.mid.get::<percent>(), 100.0);

        let preset = read_effect_preset("stereo", "stereo-lane3-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[2].snapins[0];
        let effect = snapin.effect.as_stereo().unwrap();
        assert_eq!(effect.width.get::<percent>(), 100.0);

        let preset = read_effect_preset("stereo", "stereo-disabled-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_stereo().unwrap();
        assert_eq!(effect.pan.get::<percent>(), 0.0);

        let preset =
            read_effect_preset("stereo", "stereo-mid50-width60-pan70-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_stereo().unwrap();
        assert_relative_eq!(effect.mid.get::<percent>(), 50.0, epsilon = 0.0001);
        assert_relative_eq!(effect.width.get::<percent>(), 60.0, epsilon = 0.0001);
        assert_relative_eq!(effect.pan.get::<percent>(), 70.0, epsilon = 0.0001);

        let preset = read_effect_preset("stereo", "stereo-5of-1.8.13.phaseplant").unwrap();
        assert_eq!(preset.lanes[0].snapins.len(), 5);
        preset.lanes[0].snapins.iter().for_each(|snapin| {
            let effect = snapin.effect.as_stereo().unwrap();
            assert_eq!(effect.mid.get::<percent>(), 100.0);
            assert_eq!(effect.width.get::<percent>(), 100.0);
            assert_eq!(effect.pan.get::<percent>(), 0.0);
        });
    }
}
