//! [Gain](https://kilohearts.com/products/gain) is a volume control.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.0               | 1038           |
//! | 1.8.5 to 1.8.14     | 1039           |
//! | 2.0.0               | 1048           |
//! | 2.0.16              | 1050           |

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::effect::EffectVersion;
use crate::{Decibels, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

// Phase Plant 1.8.6 added a percent fade mode in addition to existing decibel trim mode.

#[derive(Clone, Debug, PartialEq)]
pub struct Gain {
    /// Can range from 0 to 200% (-30 to +30 dB) in the Phase Plant interface.
    pub amount: f32,

    /// If the amount is decibels or a percentage.
    pub percentage: bool,
}

impl Gain {
    pub const GAIN_AMOUNT_MIN: f32 = -30.0;
    pub const GAIN_AMOUNT_MAX: f32 = 30.0;

    /// Provide the amount of gain in decibels.
    pub fn amount_db(&self) -> Decibels {
        Decibels::from_linear(self.amount)
    }

    /// The mount of gain as a percentage between [`Self::GAIN_AMOUNT_MIN`] (0%)
    /// and [`Self::GAIN_AMOUNT_MAX`] (200%).
    pub fn amount_percentage(&self) -> f32 {
        let range = Self::GAIN_AMOUNT_MAX - Self::GAIN_AMOUNT_MIN;
        (self.amount_db().db() - Self::GAIN_AMOUNT_MIN) / range * 2.0 // 0 to 200%
    }

    pub fn default_version() -> EffectVersion {
        1050
    }
}

impl Default for Gain {
    fn default() -> Self {
        Self {
            amount: 1.0,
            percentage: false,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_gain(&self) -> Option<&Gain> {
        self.downcast_ref::<Gain>()
    }
}

impl Effect for Gain {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Gain
    }
}

impl EffectRead for Gain {
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
        let amount = reader.read_f32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "gain_unknown_1")?;

        let percentage = reader.read_bool32()?;

        if effect_version > 1038 {
            reader.expect_u32(0, "gain_unknown_2")?;
        }

        let group_id = if effect_version >= 1048 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(Gain { amount, percentage }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Gain {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(snapin.enabled)?;
        writer.write_f32(self.amount)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?;

        writer.write_bool32(self.percentage)?;

        writer.write_u32(0)?;

        if snapin.effect_version >= 1050 {
            writer.write_snapin_id(snapin.group_id)?;
        }

        Ok(())
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
        let effect = Gain::default();
        assert!(!effect.percentage);
        assert_eq!(effect.amount, 1.0);
    }

    #[test]
    fn disabled() {
        let preset = read_effect_preset("gain", "gain-disabled-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = Gain::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Gain::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in [
            "gain-1.8.0.phaseplant",
            "gain-1.8.13.phaseplant",
            "gain-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("gain", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_gain().unwrap();
            assert!(!effect.percentage);
            assert_eq!(effect.amount, 1.0);
            assert_eq!(effect.amount_db().db(), 0.0);
            assert_eq!(effect.amount_percentage(), 1.0);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset("gain", "gain-125%-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        let effect = snapin.effect.as_gain().unwrap();
        assert!(effect.percentage);
        assert_relative_eq!(effect.amount, 2.3713737, epsilon = 0.00001);
        assert_relative_eq!(effect.amount_db().db(), 7.5, epsilon = 0.00001);
        assert_relative_eq!(effect.amount_percentage(), 1.25, epsilon = 0.00001);
    }

    #[test]
    fn minimized() {
        for file in [
            "gain-minimized-1.8.14.phaseplant",
            "gain-minimized-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("gain", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(snapin.minimized);
        }
    }
}
