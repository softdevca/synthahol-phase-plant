//! [Resonator](https://kilohearts.com/products/resonator) is a harmonic
//! resonance effect.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.13     | 1038           |
//! | 2.0.16              | 1049           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::si::f32::{Ratio, Time};
use uom::si::ratio::percent;
use uom::si::time::{millisecond, second};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Resonator {
    /// MIDI note number plus fractional tuning
    pub note: f32,

    /// Square unless sawtooth
    pub sawtooth: bool,

    pub decay: Time,
    pub intensity: f32,
    pub mix: Ratio,
}

impl Default for Resonator {
    fn default() -> Self {
        Self {
            note: 69.0, // Nice
            sawtooth: true,
            decay: Time::new::<millisecond>(10.0),
            intensity: 0.5,
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_resonator(&self) -> Option<&Resonator> {
        self.downcast_ref::<Resonator>()
    }
}

impl Effect for Resonator {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Resonator
    }
}

impl EffectRead for Resonator {
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
        let note = reader.read_f32()?;
        let decay = reader.read_seconds()?;
        let intensity = reader.read_f32()?;
        let sawtooth = !reader.read_bool32()?;
        let mix = reader.read_ratio()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "resonator_unknown_1")?;
        reader.expect_u32(0, "resonator_unknown_2")?;
        if effect_version > 1038 {
            reader.expect_u32(0, "resonator_unknown_3")?;
        }

        Ok(EffectReadReturn::new(
            Box::new(Resonator {
                note,
                sawtooth,
                decay,
                intensity,
                mix,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for Resonator {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_f32(self.note)?;
        writer.write_f32(self.decay.get::<second>())?;
        writer.write_f32(self.intensity)?;
        writer.write_bool32(!self.sawtooth)?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?; // resonator_unknown_1
        writer.write_u32(0)?; // resonator_unknown_2
        if self.write_version() > 1038 {
            writer.write_u32(0)?; // resonator_unknown_3
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1049
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::time::second;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn default() {
        let effect = Resonator::default();
        assert!(effect.sawtooth);
        assert_relative_eq!(effect.decay.get::<second>(), 0.010, epsilon = 0.00001);
        assert_relative_eq!(effect.intensity, 0.5, epsilon = 0.002);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(effect.note, 69.0);
    }

    #[test]
    fn disabled() {
        let preset =
            read_effect_preset("resonator", "resonator-disabled-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = Resonator::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Resonator::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &["resonator-1.8.13.phaseplant", "resonator-2.0.16.phaseplant"] {
            let preset = read_effect_preset("resonator", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_resonator().unwrap();
            assert!(effect.sawtooth);
            assert_relative_eq!(effect.decay.get::<second>(), 0.010, epsilon = 0.00001);
            assert_relative_eq!(effect.intensity, 0.5, epsilon = 0.002);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
            assert_eq!(effect.note, 69.0);
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("resonator", "resonator-minimized-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts_version_1() {
        let preset =
            read_effect_preset("resonator", "resonator-c3-square-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_resonator().unwrap();
        assert!(!effect.sawtooth);
        assert_eq!(effect.note, 48.0);

        let preset = read_effect_preset(
            "resonator",
            "resonator-d1+30-intensity25%-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_resonator().unwrap();
        assert_eq!(effect.intensity, 0.25);
        assert_relative_eq!(effect.note, 26.3, epsilon = 0.00001);
    }
}
