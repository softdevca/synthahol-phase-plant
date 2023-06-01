//! [Pitch Shifter](https://kilohearts.com/products/pitch_shifter) performs
//! harmonic pitch shifting.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1039           |
//! | 1.8.13              | 1039           |
//! | 2.0.16              | 1050           |

use std::any::{type_name, Any};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uom::si::f32::{Ratio, Time};
use uom::si::ratio::{percent, ratio};
use uom::si::time::{millisecond, second};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Copy, Debug, EnumIter, Eq, PartialEq)]
#[repr(u8)]
pub enum CompensationMode {
    // The discriminants correspond to the file format.
    Off = 0,
    Low = 1,
    High = 2,
}

impl CompensationMode {
    fn from_id(id: u32) -> Result<CompensationMode, Error> {
        match CompensationMode::iter().find(|mode| *mode as u32 == id) {
            Some(mode) => Ok(mode),
            None => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Compensation mode {} not found", id),
            )),
        }
    }
}

impl Display for CompensationMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            CompensationMode::Off => "Off",
            CompensationMode::Low => "Low",
            CompensationMode::High => "High",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PitchShifter {
    pub pitch: f32,
    pub jitter: f32,
    pub grain_size: Time,
    pub mix: Ratio,
    pub correlate: bool,
    pub compensation_mode: CompensationMode,
}

impl Default for PitchShifter {
    fn default() -> Self {
        PitchShifter {
            pitch: 0.0,
            jitter: 0.0,
            grain_size: Time::new::<millisecond>(80.0),
            mix: Ratio::new::<percent>(100.0),
            correlate: true,
            compensation_mode: CompensationMode::Low,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_pitch_shifter(&self) -> Option<&PitchShifter> {
        self.downcast_ref::<PitchShifter>()
    }
}

impl Effect for PitchShifter {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::PitchShifter
    }
}

impl EffectRead for PitchShifter {
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
        let pitch = reader.read_f32()?;
        let jitter = reader.read_f32()?;
        let grain_size = Time::new::<second>(reader.read_f32()?);
        let mix = Ratio::new::<ratio>(reader.read_f32()?);
        let correlate = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "pitch_shifter_unknown1")?;
        reader.expect_u32(0, "pitch_shifter_unknown2")?;

        let compensation_mode = CompensationMode::from_id(reader.read_u32()?)?;

        if effect_version > 1039 {
            reader.expect_u32(0, "pitch_shifter_unknown3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(PitchShifter {
                pitch,
                jitter,
                grain_size,
                mix,
                correlate,
                compensation_mode,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for PitchShifter {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_f32(self.pitch)?;
        writer.write_f32(self.jitter)?;
        writer.write_f32(self.grain_size.get::<second>())?;
        writer.write_f32(self.mix.get::<ratio>())?;
        writer.write_bool32(self.correlate)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // pitch_shifter_unknown1
        writer.write_u32(0)?; // pitch_shifter_unknown2

        writer.write_u32(self.compensation_mode as u32)?;

        if self.write_version() > 1039 {
            writer.write_u32(0)?; // pitch_shifter_unknown3
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
    use uom::si::f32::Time;
    use uom::si::time::millisecond;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = PitchShifter::default();
        assert_eq!(effect.pitch, 0.0);
        assert_eq!(effect.jitter, 0.0);
        assert_eq!(effect.grain_size, Time::new::<millisecond>(80.0));
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert!(effect.correlate);
        assert_eq!(effect.compensation_mode, CompensationMode::Low);
    }

    #[test]
    fn eq() {
        let effect = PitchShifter::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, PitchShifter::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    pub fn init() {
        for file in &[
            "pitch_shifter-1.8.13.phaseplant",
            "pitch_shifter-2.0.16.phaseplant",
        ] {
            let preset = read_effect_preset("pitch_shifter", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_pitch_shifter().unwrap();
            assert_eq!(effect.pitch, 0.0);
            assert_eq!(effect.jitter, 0.0);
            assert_relative_eq!(effect.grain_size.get::<millisecond>(), 80.0);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
            assert!(effect.correlate);
            assert_eq!(effect.compensation_mode, CompensationMode::Low);
        }
    }

    #[test]
    fn parts() {
        let preset = read_effect_preset(
            "pitch_shifter",
            "pitch_shifter-comp_off-minimized-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_pitch_shifter().unwrap();
        assert_eq!(effect.compensation_mode, CompensationMode::Off);

        let preset = read_effect_preset(
            "pitch_shifter",
            "pitch_shifter-correlate_off-comp_high-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_pitch_shifter().unwrap();
        assert!(!effect.correlate);
        assert_eq!(effect.compensation_mode, CompensationMode::High);

        let preset = read_effect_preset(
            "pitch_shifter",
            "pitch_shifter-jitter50-grain100-mix35-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_pitch_shifter().unwrap();
        assert_relative_eq!(effect.jitter, 0.50, epsilon = 0.01);
        assert_relative_eq!(effect.grain_size.get::<millisecond>(), 100.0, epsilon = 0.1);
        assert_relative_eq!(effect.mix.get::<percent>(), 35.48, epsilon = 0.01);

        let preset = read_effect_preset(
            "pitch_shifter",
            "pitch_shifter-plus5-disabled-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_pitch_shifter().unwrap();
        assert_eq!(effect.pitch, 5.0);
    }
}
