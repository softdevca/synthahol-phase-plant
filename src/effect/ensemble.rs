//! [Ensemble](https://kilohearts.com/products/ensemble) is an effect that
//! creates additional unison voices.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1003           |
//! | 1.8.13              | 1003           |
//! | 2.0.FIXME           | 1012           |
//! | 2.0.12              | 1013           |
//! | 2.0.16              | 1014           |

use std::any::Any;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum_macros::FromRepr;
use uom::si::f32::Ratio;
use uom::si::ratio::{percent, ratio};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum MotionMode {
    // The discriminants correspond to the file format.
    Random = 0,
    Symmetric = 1,
    Sine = 2,
}

impl MotionMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown ensemble motion mode {id}"),
            )
        })
    }
}

impl Display for MotionMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            MotionMode::Random => "Random",
            MotionMode::Symmetric => "Symmetric",
            MotionMode::Sine => "Sine",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ensemble {
    pub voices: u32,
    pub detune: Ratio,
    pub spread: Ratio,
    pub mix: Ratio,
    pub motion_mode: MotionMode,
}

impl Default for Ensemble {
    fn default() -> Self {
        Ensemble {
            voices: 6,
            detune: Ratio::new::<percent>(25.0),
            spread: Ratio::new::<percent>(50.0),
            mix: Ratio::new::<percent>(100.0),
            motion_mode: MotionMode::Symmetric,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_ensemble(&self) -> Option<&Ensemble> {
        self.downcast_ref::<Ensemble>()
    }
}

impl Effect for Ensemble {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Ensemble
    }
}

impl EffectRead for Ensemble {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1003 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Ensemble effect version {effect_version}"),
            ));
        }

        let voices = reader.read_u32()?;
        let detune = Ratio::new::<ratio>(reader.read_f32()?);
        let spread = Ratio::new::<ratio>(reader.read_f32()?);
        let mix = Ratio::new::<ratio>(reader.read_f32()?);
        let enabled = reader.read_bool32()?;

        let minimized = reader.read_bool32()?;
        reader.expect_u32(0, "ensemble_u3")?;

        // FIXME: VERIFY THIS IS MOTION MODE WitH SINE TEST
        let motion_mode = MotionMode::from_id(reader.read_u32()?)?;

        reader.expect_u32(0, "ensemble_u4")?;
        if effect_version >= 1012 {
            reader.expect_u32(0, "ensemble_u4")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Ensemble {
                voices,
                detune,
                spread,
                mix,
                motion_mode,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Ensemble {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_u32(self.voices)?;
        writer.write_f32(self.detune.get::<ratio>())?;
        writer.write_f32(self.spread.get::<ratio>())?;
        writer.write_f32(self.mix.get::<ratio>())?;
        writer.write_bool32(enabled)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;

        writer.write_u32(self.motion_mode as u32)?;

        writer.write_u32(0)?;
        if self.write_version() >= 1013 {
            writer.write_u32(0)?;
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1014
    }
}

#[cfg(test)]
mod test {
    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = Ensemble::default();
        assert_eq!(effect.voices, 6);
        assert_eq!(effect.detune.get::<percent>(), 25.0);
        assert_eq!(effect.spread.get::<percent>(), 50.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(effect.motion_mode, MotionMode::Symmetric);
    }

    #[test]
    fn eq() {
        let effect = Ensemble::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Ensemble::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "ensemble-1.8.13.phaseplant",
            "ensemble-2.0.12.phaseplant",
            "ensemble-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("ensemble", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_ensemble().unwrap();
            assert_eq!(effect, &Ensemble::default());
        }
    }

    #[test]
    fn detune() {
        let preset =
            read_effect_preset("ensemble", "ensemble-heavy_detune-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Heavy Detune");
        assert_eq!(snapin.preset_path, vec!["factory", "Heavy Detune.ksun"]);
        assert!(!snapin.preset_edited);
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("ensemble", "ensemble-minimized-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn sine() {
        let preset = read_effect_preset("ensemble", "ensemble-sine-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        assert_eq!(snapin.preset_name, ""); // Sine is the default.
        assert!(snapin.preset_path.is_empty());
    }

    #[test]
    fn voices_random() {
        let preset =
            read_effect_preset("ensemble", "ensemble-16voices-random-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_ensemble().unwrap();
        assert_eq!(effect.voices, 16);
        assert_eq!(effect.motion_mode, MotionMode::Random);
    }
}
