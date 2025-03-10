//! [Distortion](https://kilohearts.com/products/distortion) is a distortion
//! effect with multiple distortion shapes.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.6.9               | 1037           |
//! | 1.8.5 to 1.8.14     | 1038           |
//! | 2.0.12              | 1049           |
//! | 2.0.16              | 1050           |

use std::any::{Any, type_name};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum_macros::FromRepr;
use uom::num::Zero;
use uom::si::f32::Ratio;
use uom::si::ratio::percent;

use crate::effect::EffectVersion;
use crate::{Decibels, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum DistortionMode {
    // The discriminants correspond to the file format.
    Overdrive = 0,
    Saturate = 1,
    Foldback = 2,
    Sine = 3,
    HardClip = 4,

    /// Quantize was added in Phase Plant 1.8.0
    Quantize = 5,
}

impl DistortionMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown distortion mode {id}"),
            )
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Distortion {
    pub mode: DistortionMode,
    pub drive: Decibels,
    pub dynamics: Ratio,
    pub bias: Ratio,
    pub spread: Ratio,

    // DC Filter was added in Phase Plant version 2.
    pub dc_filter: bool,

    pub mix: Ratio,
}

impl Distortion {
    pub fn default_version() -> EffectVersion {
        1050
    }

    pub fn new() -> Self {
        Self::default()
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_distortion(&self) -> Option<&Distortion> {
        self.downcast_ref::<Distortion>()
    }
}

impl Clone for Distortion {
    fn clone(&self) -> Self {
        Self { ..*self }
    }
}

impl Default for Distortion {
    fn default() -> Self {
        Self {
            mode: DistortionMode::Overdrive,
            drive: Decibels::new(6.0),
            dynamics: Ratio::new::<percent>(50.0),
            bias: Ratio::zero(),
            spread: Ratio::zero(),
            dc_filter: true,
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

impl Effect for Distortion {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Distortion
    }
}

impl EffectRead for Distortion {
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
        let drive = reader.read_decibels_linear()?;
        let bias = reader.read_ratio()?;
        let spread = reader.read_ratio()?;
        let mode = DistortionMode::from_id(reader.read_u32()?)?;
        let dynamics = reader.read_ratio()?;
        let mix = reader.read_ratio()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "distortion_unknown_1")?;
        reader.expect_u32(0, "distortion_unknown_2")?;

        let mut dc_filter = true;
        let mut group_id = None;
        if effect_version > 1038 {
            group_id = reader.read_snapin_position()?;
            dc_filter = reader.read_bool32()?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Distortion {
                mode,
                drive,
                dynamics,
                bias,
                spread,
                dc_filter,
                mix,
            }),
            enabled,
            minimized,
            group_id,
        ))
    }
}

impl EffectWrite for Distortion {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        snapin: &Snapin,
    ) -> io::Result<()> {
        writer.write_bool32(snapin.enabled)?;
        writer.write_decibels_linear(self.drive)?;
        writer.write_ratio(self.bias)?;
        writer.write_ratio(self.spread)?;
        writer.write_u32(self.mode as u32)?;
        writer.write_ratio(self.dynamics)?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(snapin.minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;

        if snapin.effect_version > 1038 {
            writer.write_snapin_id(snapin.group_id)?;
            writer.write_bool32(self.dc_filter)?;
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
        let effect = Distortion::default();
        assert_eq!(effect.mode, DistortionMode::Overdrive);
        assert_relative_eq!(effect.drive.db(), 6.0, epsilon = 0.001);
        assert_relative_eq!(effect.dynamics.get::<percent>(), 50.0, epsilon = 0.005);
        assert_eq!(effect.bias.get::<percent>(), 0.0);
        assert_eq!(effect.spread.get::<percent>(), 0.0);
        assert!(effect.dc_filter);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
    }

    #[test]
    fn eq() {
        let effect = Distortion::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Distortion::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "distortion-1.8.14.phaseplant",
            "distortion-2.0.12.phaseplant",
        ] {
            let preset = read_effect_preset("distortion", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_distortion().unwrap();

            // Cannot compare directly against the default because of floating point rounding
            assert_eq!(effect.mode, DistortionMode::Overdrive);
            assert_relative_eq!(effect.drive.db(), 6.0, epsilon = 0.01);
            assert_relative_eq!(effect.dynamics.get::<percent>(), 50.0, epsilon = 0.5);
            assert_eq!(effect.bias.get::<percent>(), 0.0);
            assert_eq!(effect.spread.get::<percent>(), 0.0);
            assert!(effect.dc_filter);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "distortion",
            "distortion-foldback-dynamics75-minimized-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_distortion().unwrap();
        assert_eq!(effect.mode, DistortionMode::Foldback);
        assert_eq!(effect.dynamics.get::<percent>(), 75.0);

        let preset = read_effect_preset(
            "distortion",
            "distortion-saturate-drive2-disabled-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_distortion().unwrap();
        assert_eq!(effect.mode, DistortionMode::Saturate);
        assert_relative_eq!(effect.drive.db(), 2.0, epsilon = 0.001);

        let preset = read_effect_preset(
            "distortion",
            "distortion-sine-bias25-spread66-mix70-1.8.14.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_distortion().unwrap();
        assert_eq!(effect.mode, DistortionMode::Sine);
        assert_eq!(effect.bias.get::<percent>(), 25.0);
        assert_eq!(effect.spread.get::<percent>(), 66.0);
        assert_eq!(effect.mix.get::<percent>(), 70.0);
    }
}
