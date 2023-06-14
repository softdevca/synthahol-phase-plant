//! [Filter](https://kilohearts.com/docs/snapins/) is a resonant filter
//! effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.6.9               | 1039           |
//! | 1.8.5               | 1038           |
//! | 1.8.13 to 1.8.16    | 1040           |
//! | 2.0.16              | 1051           |

use std::any::{type_name, Any};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum_macros::FromRepr;
use uom::si::f32::Frequency;
use uom::si::frequency::hertz;

use crate::effect::EffectVersion;
use crate::{Decibels, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum FilterMode {
    // The discriminants correspond to the file format.
    LowPass = 0,
    BandPass = 1,
    HighPass = 2,
    Notch = 3,
    LowShelf = 4,
    Peak = 5,
    HighShelf = 6,
}

impl FilterMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown filter mode {id}")))
    }
}

/// The high pass and low pass modes are called "low cut" and "high cut"
/// in Slice EQ.
impl Display for FilterMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FilterMode::*;
        let name = match self {
            LowPass => "Low pass",
            BandPass => "Band pass",
            HighPass => "High pass",
            Notch => "Notch",
            LowShelf => "Low shelf",
            Peak => "Peak",
            HighShelf => "High shelf",
        };
        f.write_str(name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub filter_mode: FilterMode,
    pub cutoff: Frequency,
    pub q: f32,
    pub gain: Decibels,
    pub slope: u32,
}

impl Filter {
    pub const RESONANCE_MIN: f64 = 0.1;

    pub fn default_version() -> EffectVersion {
        1051
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_filter(&self) -> Option<&Filter> {
        self.downcast_ref::<Filter>()
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            filter_mode: FilterMode::LowPass,
            cutoff: Frequency::new::<hertz>(620.0), // Default is 440.0 in generators and 620.0 in effects
            q: 0.707,
            gain: Decibels::new(6.0), // Default is 0.0 for generators
            slope: 1,
        }
    }
}

impl Effect for Filter {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Filter
    }
}

impl EffectRead for Filter {
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
        let mode = FilterMode::from_id(reader.read_u32()?)?;
        let cutoff = reader.read_hertz()?;
        let q = reader.read_f32()?;
        let gain = reader.read_decibels_db()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "filter_unknown1")?;
        reader.expect_u32(0, "filter_unknown2")?;

        let slope = if effect_version > 1039 {
            reader.read_u32()?
        } else {
            1
        };

        let group_id = if effect_version > 1040 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(Filter {
                filter_mode: mode,
                cutoff,
                q,
                gain,
                slope,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Filter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(snapin.enabled)?;
        writer.write_u32(self.filter_mode as u32)?;
        writer.write_f32(self.cutoff.get::<hertz>())?;
        writer.write_f32(self.q)?;
        writer.write_f32(self.gain.db())?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?; // filter_unknown1
        writer.write_u32(0)?; // filter_unknown2

        writer.write_u32(self.slope)?;

        if snapin.effect_version > 1040 {
            writer.write_snapin_id(snapin.group_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::{Bitcrush, Filter};
    use crate::test::read_effect_preset;
    use crate::Decibels;

    use super::*;

    #[test]
    fn default() {
        let effect = Filter::default();
        assert_eq!(effect.filter_mode, FilterMode::LowPass);
        assert_eq!(effect.cutoff.get::<hertz>(), 620.0);
        assert_relative_eq!(effect.q, 0.707, epsilon = 0.0001);
        assert_relative_eq!(effect.gain.db(), 6.0, epsilon = 0.0001);
        assert_eq!(effect.slope, 1);
    }

    #[test]
    fn eq() {
        let effect = Filter::default();
        assert_eq!(effect, Filter::default());
        assert!(!effect.box_eq(&Bitcrush::default()));
    }

    #[test]
    fn init() {
        for file in &["filter-1.8.13.phaseplant", "filter-2.0.16.phaseplant"] {
            let preset = read_effect_preset("filter", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            assert_eq!(snapin.id, 1);
            let effect = snapin.effect.as_filter().unwrap();
            assert_eq!(effect.filter_mode, FilterMode::LowPass);
            assert_eq!(effect.cutoff.get::<hertz>(), 620.0);
            assert_relative_eq!(effect.q, 0.707, epsilon = 0.0001);
            assert_relative_eq!(effect.gain.db(), 6.0, epsilon = 0.0001);
            assert_eq!(effect.slope, 1);
        }
    }

    #[test]
    fn modes() {
        let preset = read_effect_preset("filter", "filter-all_modes-2.1.0.phaseplant").unwrap();
        let snapins = &preset.lanes[0].snapins;
        let effect = snapins[0].effect.as_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::LowPass);
        let effect = snapins[1].effect.as_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::BandPass);
        let effect = snapins[2].effect.as_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::HighPass);
        let effect = snapins[3].effect.as_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::Notch);
        let effect = snapins[4].effect.as_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::LowShelf);
        let effect = snapins[5].effect.as_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::Peak);
        let effect = snapins[6].effect.as_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::HighShelf);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset("filter", "filter-bandpass-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::BandPass);

        let preset =
            read_effect_preset("filter", "filter-cutoff440-q1.1-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.downcast_ref::<Filter>().unwrap();
        assert_relative_eq!(effect.cutoff.get::<hertz>(), 440.0, epsilon = 0.0001);
        assert_relative_eq!(effect.q, 1.1, epsilon = 0.0001);

        let preset =
            read_effect_preset("filter", "filter-gain-5-minimized-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_filter().unwrap();
        assert_eq!(effect.gain, Decibels::new(-5.0));

        let preset = read_effect_preset("filter", "filter-gain3-slope3-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_filter().unwrap();
        assert_eq!(effect.gain, Decibels::new(3.0));
        assert_eq!(effect.slope, 3);

        let preset =
            read_effect_preset("filter", "filter-slope6-disabled-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_filter().unwrap();
        assert_eq!(effect.slope, 6);
    }
}
