//! [Formant Filter](https://kilohearts.com/products/formant_filter) is a
//! vocal coloring effect. A E I O U and sometimes Y.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1037           |
//! | 1.8.16              | 1037           |
//! | 2.0.12              | 1047           |
//! | 2.1.0               | 1048           |

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use crate::effect::EffectVersion;
use uom::si::f32::Frequency;
use uom::si::frequency::hertz;

use crate::Snapin;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct FormantFilter {
    pub q: f32,
    pub lows: bool,
    pub highs: bool,
    pub x: Frequency,
    pub y: Frequency,
}

impl FormantFilter {
    pub fn default_version() -> EffectVersion {
        1048
    }
}

impl Default for FormantFilter {
    fn default() -> Self {
        Self {
            q: 4.0,
            lows: true,
            highs: true,
            x: Frequency::new::<hertz>(550.0),
            y: Frequency::new::<hertz>(1500.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_formant_filter(&self) -> Option<&FormantFilter> {
        self.downcast_ref::<FormantFilter>()
    }
}

impl Effect for FormantFilter {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::FormantFilter
    }
}

impl EffectRead for FormantFilter {
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
        let x = reader.read_hertz()?;
        let y = reader.read_hertz()?;
        let q = reader.read_f32()?;
        let lows = reader.read_bool32()?;
        let highs = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "formant_filter_unknown_1")?;
        reader.expect_u32(0, "formant_filter_unknown_2")?;

        let group_id = if effect_version >= 1038 {
            reader.read_snapin_position()?
        } else {
            None
        };

        Ok(EffectReadReturn::new(
            Box::new(FormantFilter {
                q,
                lows,
                highs,
                x,
                y,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for FormantFilter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(snapin.enabled)?;
        writer.write_hertz(self.x)?;
        writer.write_hertz(self.y)?;
        writer.write_f32(self.q)?;
        writer.write_bool32(self.lows)?;
        writer.write_bool32(self.highs)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?; // formant_filter_unknown_1
        writer.write_u32(0)?; // formant_filter_unknown_2
        writer.write_snapin_id(snapin.group_id)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    pub fn default() {
        let effect = FormantFilter::default();
        assert_eq!(effect.q, 4.0);
        assert!(effect.lows);
        assert!(effect.highs);
        assert_eq!(effect.x.get::<hertz>(), 550.0);
        assert_eq!(effect.y.get::<hertz>(), 1500.0);
    }

    #[test]
    pub fn eq() {
        let effect = FormantFilter::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, FormantFilter::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    pub fn init() {
        for file in &[
            "formant_filter-2.0.12.phaseplant",
            "formant_filter-2.0.16.phaseplant",
            "formant_filter-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("formant_filter", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_formant_filter().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    pub fn parts_version_1() {
        let preset = read_effect_preset(
            "formant_filter",
            "formant_filter-500hz-1khz-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_formant_filter().unwrap();
        assert_eq!(effect.x.get::<hertz>(), 500.0);
        assert_eq!(effect.y.get::<hertz>(), 1000.0);

        let preset = read_effect_preset(
            "formant_filter",
            "formant_filter-high_off-disabled-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_formant_filter().unwrap();
        assert!(!effect.highs);
        assert!(effect.lows);

        let preset = read_effect_preset(
            "formant_filter",
            "formant_filter-low_off-q10-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_formant_filter().unwrap();
        assert!(effect.highs);
        assert!(!effect.lows);
        assert_eq!(effect.q, 10.0);
    }

    #[test]
    pub fn parts_version_2() {
        let preset = read_effect_preset(
            "formant_filter",
            "formant_filter-x200-y2500-q10-minimized-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_formant_filter().unwrap();
        assert_eq!(effect.x.get::<hertz>(), 200.0);
        assert_eq!(effect.y.get::<hertz>(), 2500.0);
        assert_eq!(effect.q, 10.0);
    }
}
