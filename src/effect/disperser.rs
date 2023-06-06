//! [Disperser](https://kilohearts.com/products/disperser) as an all-pass
//! filter.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.16     | 1039           |
//! | 2.0.12              | 1050           |

use std::any::Any;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::Frequency;
use uom::si::frequency::hertz;

use super::{Effect, EffectMode};
use super::super::io::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Disperser {
    pub frequency: Frequency,
    pub amount: u32,
    pub pinch: f32,
    unknown2: bool,
}

impl Default for Disperser {
    fn default() -> Self {
        Self {
            frequency: Frequency::new::<hertz>(130.0),
            amount: 18,
            pinch: 0.5,
            unknown2: true,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_disperser(&self) -> Option<&Disperser> {
        self.downcast_ref::<Disperser>()
    }
}

impl Effect for Disperser {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Disperser
    }
}

impl EffectRead for Disperser {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1039 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Disperser effect version {effect_version}"),
            ));
        }

        let amount = reader.read_u32()?;
        let frequency = reader.read_hertz()?;
        let pinch = reader.read_f32()?;

        let unknown2 = reader.read_bool32()?;

        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "disperser_unknown_1")?;
        reader.expect_u32(0, "disperser_unknown_2")?;
        if effect_version > 1039 {
            reader.expect_u32(0, "disperser_unknown_3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Disperser {
                frequency,
                amount,
                pinch,
                unknown2,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Disperser {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_u32(self.amount)?;
        writer.write_hertz(self.frequency)?;
        writer.write_f32(self.pinch)?;

        writer.write_bool32(self.unknown2)?;

        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        if self.write_version() > 1039 {
            writer.write_u32(0)?;
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1050
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
        let effect = Disperser::default();
        assert_relative_eq!(effect.frequency.get::<hertz>(), 130.0, epsilon = 1.0);
        assert_eq!(effect.amount, 18);
        assert_eq!(effect.pinch, 0.50);
    }

    #[test]
    fn eq() {
        let effect = Disperser::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Disperser::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "disperser-1.8.5.phaseplant",
            "disperser-1.8.14.phaseplant",
            "disperser-2.0.12.phaseplant",
        ] {
            let preset = read_effect_preset("disperser", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_disperser().unwrap();
            assert_relative_eq!(effect.frequency.get::<hertz>(), 130.0, epsilon = 1.0);
            assert_eq!(effect.amount, 18);
            assert_eq!(effect.pinch, 0.50);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "disperser",
            "disperser-200hz-amount10-minimized-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_disperser().unwrap();
        assert_relative_eq!(effect.frequency.get::<hertz>(), 200.0, epsilon = 0.001);
        assert_eq!(effect.amount, 10);
        assert_eq!(effect.pinch, 0.50);

        let preset =
            read_effect_preset("disperser", "disperser-pinch3-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_disperser().unwrap();
        assert_relative_eq!(effect.frequency.get::<hertz>(), 130.0, epsilon = 1.0);
        assert_eq!(effect.amount, 18);
        assert_relative_eq!(effect.pinch, 3.0, epsilon = 0.1);
    }
}
