//! [Frequency Shifter](https://kilohearts.com/products/frequency_shifter)
//! moves all frequencies in the input signal up or down.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.16     | 1037           |
//! | 2.0.12              | 1047           |
//! | 2.0.16              | 1048           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::Frequency;
use uom::si::frequency::kilohertz;

use crate::SnapinId;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct FrequencyShifter {
    pub frequency: Frequency,
}

impl FrequencyShifter {
    // TODO: Enable when uom supports const fn.
    // pub const MIN_FREQUENCY: Frequency = Frequency::new::<hertz>(-5000.0);
    // pub const MAX_FREQUENCY: Frequency = Frequency::new::<hertz>(5000.0);
}

impl Default for FrequencyShifter {
    fn default() -> Self {
        Self {
            frequency: Frequency::zero(),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_frequency_shifter(&self) -> Option<&FrequencyShifter> {
        self.downcast_ref::<FrequencyShifter>()
    }
}

impl Effect for FrequencyShifter {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::FrequencyShifter
    }
}

impl EffectRead for FrequencyShifter {
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
        let frequency = Frequency::new::<kilohertz>(reader.read_f32()?);
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "frequency_shifter_unknown_1")?;
        reader.expect_u32(0, "frequency_shifter_unknown_2")?;

        let group_id = if effect_version > 1037 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(FrequencyShifter { frequency }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for FrequencyShifter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
        group_id: Option<SnapinId>,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_f32(self.frequency.get::<kilohertz>())?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // frequency_shifter_unknown_1
        writer.write_u32(0)?; // frequency_shifter_unknown_2

        if self.write_version() > 1037 {
            writer.write_snapin_id(group_id)?;
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1047
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::f32::Frequency;
    use uom::si::frequency::hertz;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = FrequencyShifter::default();
        assert_eq!(effect.frequency, Frequency::zero());
    }

    #[test]
    fn eq() {
        let effect = FrequencyShifter::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, FrequencyShifter::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "frequency_shifter-2.0.12.phaseplant",
            "frequency_shifter-2.0.16.phaseplant",
            "frequency_shifter-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("frequency_shifter", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_frequency_shifter().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    fn minimized() {
        let preset = read_effect_preset(
            "frequency_shifter",
            "frequency_shifter-minimized-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts() {
        let preset = read_effect_preset(
            "frequency_shifter",
            "frequency_shifter-1khz-disabled-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_frequency_shifter().unwrap();
        assert_relative_eq!(effect.frequency.get::<hertz>(), 1000.0, epsilon = 1.0);
    }
}
