//! [Nonlinear Filter](https://kilohearts.com/products/nonlinear_filter) is a
//! colorful filtering effect.
//!
//! Nonlinear Filter was added to Phase Plant in version 1.8.15.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.16              | 1000           |
//! | 2.0.16              | 1011           |

use std::any::{type_name, Any};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum_macros::FromRepr;
use uom::si::f32::Frequency;
use uom::si::frequency::hertz;

use crate::effect::FilterMode;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum NonlinearFilterMode {
    // The discriminants correspond to the file format.
    Clean = 0,
    Saturated,
    Tubular,
    Clipped,
    Warm,
    Biased,
    Fuzzy,
    Metallic,
    Digital,
}

impl NonlinearFilterMode {
    fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown Nonlinear Filter mode {id}"),
            )
        })
    }
}

impl Display for NonlinearFilterMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use NonlinearFilterMode::*;
        let name = match self {
            Clean => "Clean",
            Saturated => "Saturated",
            Tubular => "Tubular",
            Clipped => "Clipped",
            Warm => "Warm",
            Biased => "Biased",
            Fuzzy => "Fuzzy",
            Metallic => "Metallic",
            Digital => "Digital",
        };
        f.write_str(name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NonlinearFilter {
    pub cutoff: Frequency,
    pub q: f32,
    pub drive: f32,
    pub mode: NonlinearFilterMode,
    pub filter_mode: FilterMode,
}

impl dyn Effect {
    #[must_use]
    pub fn as_nonlinear_filter(&self) -> Option<&NonlinearFilter> {
        self.downcast_ref::<NonlinearFilter>()
    }
}

impl Default for NonlinearFilter {
    fn default() -> Self {
        Self {
            cutoff: Frequency::new::<hertz>(440.0),
            q: 0.707,
            drive: 0.25,
            mode: NonlinearFilterMode::Saturated,
            filter_mode: FilterMode::LowPass,
        }
    }
}

impl Effect for NonlinearFilter {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::NonlinearFilter
    }
}

impl EffectRead for NonlinearFilter {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1000 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let filter_mode = FilterMode::from_id(reader.read_u32()?)?;
        let mode = NonlinearFilterMode::from_id(reader.read_u32()?)?;
        let cutoff = reader.read_hertz()?;
        let q = reader.read_f32()?;
        let drive = reader.read_f32()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "nonlinear_filter_unknown1")?;
        reader.expect_u32(0, "nonlinear_filter_unknown2")?;
        if effect_version > 1000 {
            reader.expect_u32(0, "nonlinear_filter_unknown3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(NonlinearFilter {
                cutoff,
                q,
                drive,
                mode,
                filter_mode,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for NonlinearFilter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_u32(self.filter_mode as u32)?;
        writer.write_u32(self.mode as u32)?;
        writer.write_f32(self.cutoff.get::<hertz>())?;
        writer.write_f32(self.q)?;
        writer.write_f32(self.drive)?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // nonlinear_filter_unknown1
        writer.write_u32(0)?; // nonlinear_filter_unknown2
        if self.write_version() > 1000 {
            writer.write_u32(0)?; // nonlinear_filter_unknown3
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1011
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::frequency::hertz;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = NonlinearFilter::default();
        assert_eq!(effect.cutoff.get::<hertz>(), 440.0);
        assert_eq!(effect.q, 0.707);
        assert_eq!(effect.drive, 0.25);
        assert_eq!(effect.mode, NonlinearFilterMode::Saturated);
        assert_eq!(effect.filter_mode, FilterMode::LowPass);
    }

    #[test]
    fn eq() {
        let effect = NonlinearFilter::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, NonlinearFilter::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "nonlinear_filter-1.8.15.phaseplant",
            "nonlinear_filter-1.8.16.phaseplant",
            "nonlinear_filter-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("nonlinear_filter", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_nonlinear_filter().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-cutoff1khz-warm-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_relative_eq!(effect.cutoff.get::<hertz>(), 1000.0, epsilon = 0.001);
        assert_eq!(effect.mode, NonlinearFilterMode::Warm);

        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-digital-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_eq!(effect.mode, NonlinearFilterMode::Digital);

        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-disabled-biased-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_eq!(effect.mode, NonlinearFilterMode::Biased);

        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-highpass-q1.2-tubular-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_eq!(effect.q, 1.2);
        assert_eq!(effect.filter_mode, FilterMode::HighPass);
        assert_eq!(effect.mode, NonlinearFilterMode::Tubular);

        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-lane2-fuzzy-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[1].snapins[0];
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_eq!(effect.mode, NonlinearFilterMode::Fuzzy);

        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-lane3-metallic-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[2].snapins[0];
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_eq!(effect.mode, NonlinearFilterMode::Metallic);

        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-minimized-clipped-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_eq!(effect.mode, NonlinearFilterMode::Clipped);

        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-notch-drive75-clean-1.8.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_eq!(effect.drive, 0.75);
        assert_eq!(effect.filter_mode, FilterMode::Notch);
        assert_eq!(effect.mode, NonlinearFilterMode::Clean);
    }

    #[test]
    fn parts_version_2() {
        let preset = read_effect_preset(
            "nonlinear_filter",
            "nonlinear_filter-bandpass-20hz-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_nonlinear_filter().unwrap();
        assert_eq!(effect.filter_mode, FilterMode::BandPass);
        assert_eq!(effect.cutoff.get::<hertz>(), 20.0);
    }
}
