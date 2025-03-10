//! [Comb Filter](https://kilohearts.com/products/comb_filter) is a 31-band
//! graphic equalizer.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.14     | 1038           |
//! | 2.0.0               | 1047           |
//! | 2.0.12              | 1048           |
//! | 2.0.16              | 1049           |

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::effect::EffectVersion;
use uom::si::f32::{Frequency, Ratio};
use uom::si::frequency::hertz;
use uom::si::ratio::percent;

use crate::Snapin;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct CombFilter {
    pub frequency: Frequency,

    /// Otherwise the polarity is "Plus".
    pub polarity_minus: bool,

    pub stereo: bool,
    pub mix: Ratio,
}

impl CombFilter {
    pub fn default_version() -> EffectVersion {
        1049
    }
}

impl Default for CombFilter {
    fn default() -> Self {
        Self {
            frequency: Frequency::new::<hertz>(440.0),
            polarity_minus: false,
            stereo: false,
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_comb_filter(&self) -> Option<&CombFilter> {
        self.downcast_ref::<CombFilter>()
    }
}

impl Effect for CombFilter {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::CombFilter
    }
}

impl EffectRead for CombFilter {
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
        let frequency = reader.read_hertz()?;
        let mix = reader.read_ratio()?;
        let polarity_minus = reader.read_bool32()?;
        let stereo = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "comb_filter_unknown_1")?;
        reader.expect_u32(0, "comb_filter_unknown_2")?;

        let group_id = if effect_version >= 1047 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(CombFilter {
                frequency,
                polarity_minus,
                stereo,
                mix,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for CombFilter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(snapin.enabled)?;
        writer.write_f32(self.frequency.get::<hertz>())?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(self.polarity_minus)?;
        writer.write_bool32(self.stereo)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;

        if snapin.effect_version >= 1048 {
            writer.write_snapin_id(snapin.group_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::ratio::percent;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = CombFilter::default();
        assert_eq!(effect.frequency.get::<hertz>(), 440.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert!(!effect.polarity_minus);
        assert!(!effect.stereo);
    }

    #[test]
    fn disabled() {
        let preset =
            read_effect_preset("comb_filter", "comb_filter-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = CombFilter::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, CombFilter::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in [
            "comb_filter-1.8.13.phaseplant",
            "comb_filter-2.0.12.phaseplant",
        ] {
            let preset = read_effect_preset("comb_filter", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.downcast_ref::<CombFilter>().unwrap();
            assert_eq!(effect.frequency.get::<hertz>(), 440.0);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
            assert!(!effect.polarity_minus);
            assert!(!effect.stereo);
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("comb_filter", "comb_filter-minimized-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "comb_filter",
            "comb_filter-220hz-minus-stereo_off-mix50-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.downcast_ref::<CombFilter>().unwrap();
        assert_relative_eq!(effect.frequency.get::<hertz>(), 220.0);
        assert_eq!(effect.mix.get::<percent>(), 50.0);
        assert!(effect.polarity_minus);
        assert!(!effect.stereo);

        let preset =
            read_effect_preset("comb_filter", "comb_filter-widen1-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Widen 1");
        assert_eq!(snapin.preset_path, vec!["factory", "Widen 1.kscf"]);
        assert!(!snapin.preset_edited);
    }
}
