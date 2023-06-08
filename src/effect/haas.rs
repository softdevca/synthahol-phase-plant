//! [Haas](https://kilohearts.com/products/haas) is a stereo widening effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.14     | 1037           |
//! | 2.0.0               | 1046           |
//! | 2.0.16              | 1048           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::Time;
use uom::si::time::millisecond;

use crate::SnapinId;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Haas {
    pub right: bool,
    pub delay: Time,
}

impl Default for Haas {
    fn default() -> Self {
        Haas {
            right: true,
            delay: Time::new::<millisecond>(5.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_haas(&self) -> Option<&Haas> {
        self.downcast_ref::<Haas>()
    }
}

impl Effect for Haas {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Haas
    }
}

impl EffectRead for Haas {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1037 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let enabled = reader.read_bool32()?;
        let right = reader.read_bool32()?;
        let delay = reader.read_seconds()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "haas_unknown_1")?;
        reader.expect_u32(0, "haas_unknown_2")?;

        let group_id = if effect_version >= 1046 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(Haas { right, delay }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Haas {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
        group_id: Option<SnapinId>,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_bool32(self.right)?;
        writer.write_seconds(self.delay)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;

        if self.write_version() >= 1048 {
            writer.write_snapin_id(group_id)?;
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1048
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
        let effect = Haas::default();
        assert!(effect.right);
        assert_eq!(effect.delay.get::<millisecond>(), 5.0);
    }

    #[test]
    fn eq() {
        let effect = Haas::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Haas::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "haas-1.7.0.phaseplant",
            "haas-1.8.13.phaseplant",
            "haas-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("haas", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_haas().unwrap();
            assert!(effect.right);
            assert_relative_eq!(effect.delay.get::<millisecond>(), 5.0, epsilon = 0.01);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset =
            read_effect_preset("haas", "haas-2.5ms-left-minimized-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_haas().unwrap();
        assert!(!effect.right);
        assert_relative_eq!(effect.delay.get::<millisecond>(), 2.5, epsilon = 0.01);

        let preset = read_effect_preset("haas", "haas-small_width-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Small Width");
        assert_eq!(snapin.preset_path, vec!["factory", "Small Width.ksha"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_haas().unwrap();
        assert!(effect.right);

        let preset = read_effect_preset("haas", "haas-disabled-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }
}
