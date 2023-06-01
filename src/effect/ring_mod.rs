//! [Ring Mod](https://kilohearts.com/products/ring_mod) simulates an analog
//! circuit that uses four diodes arranged in a ring.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5               | 1032           |
//! | 1.8.13              | 1032           |
//! | 2.0.16              | 1043           |

use std::any::{type_name, Any};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use uom::si::f32::Ratio;
use uom::si::ratio::{percent, ratio};

use super::super::io::*;
use super::{Effect, EffectMode};

/// The file format stores the names rather than a discriminant.
#[derive(Copy, Clone, Debug, EnumIter, Eq, PartialEq)]
#[repr(u32)]
pub enum ModulationMode {
    SineOscillator,
    LowPassNoise,
    BandPassNoise,

    /// Also known as "Self". The name was changed to avoid errors caused
    /// by conflicts with `Self` keyword
    OriginalSelf,
    Sideband,
}

impl ModulationMode {
    fn from_str(name: &str) -> Result<ModulationMode, Error> {
        // Case-sensitive
        match ModulationMode::iter().find(|mode| mode.to_string() == name) {
            Some(mode) => Ok(mode),
            None => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Ring modulator modulation mode '{name}' not found"),
            )),
        }
    }
}

impl Display for ModulationMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ModulationMode::*;
        let msg = match self {
            SineOscillator => "Sine Oscillator",
            LowPassNoise => "Low-pass Noise",
            BandPassNoise => "Band-pass Noise",
            OriginalSelf => "Self",
            Sideband => "Sideband",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Debug)]
pub struct RingMod {
    pub bias: f32,
    pub rectify: f32,
    pub frequency: f32,
    pub spread: f32,
    pub mix: Ratio,
    pub modulation_mode: ModulationMode,
    unknown2: u32,
    unknown3: u32,
}

impl Eq for RingMod {}

impl PartialEq for RingMod {
    fn eq(&self, other: &Self) -> bool {
        self.bias == other.bias
            && self.rectify == other.rectify
            && self.frequency == other.frequency
            && self.spread == other.spread
            && self.mix == other.mix
            && self.modulation_mode == other.modulation_mode
    }
}

impl Default for RingMod {
    fn default() -> Self {
        RingMod {
            bias: 0.0,
            rectify: 0.0,
            frequency: 440.0,
            spread: 0.0,
            mix: Ratio::new::<percent>(100.0),
            modulation_mode: ModulationMode::SineOscillator,
            unknown2: 0,
            unknown3: 0,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_ring_mod(&self) -> Option<&RingMod> {
        self.downcast_ref::<RingMod>()
    }
}

impl Effect for RingMod {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::RingMod
    }
}

impl EffectRead for RingMod {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1032 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let enabled = reader.read_bool32()?;
        let frequency = reader.read_f32()?;
        let spread = reader.read_f32()?;
        let mix = Ratio::new::<ratio>(reader.read_f32()?);
        let bias = reader.read_f32()?;
        let rectify = reader.read_f32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "ring_mod_unknown3")?;
        reader.expect_u32(0, "ring_mod_unknown4")?;
        let unknown3 = reader.read_u32()?;

        let mut unknown2 = 0;
        if effect_version > 1032 {
            unknown2 = reader.read_u32()?;
        }

        let mode_str = reader.read_string_and_length()?;
        let modulation_mode = ModulationMode::from_str(&mode_str.unwrap_or_default())?;

        Ok(EffectReadReturn::new(
            Box::new(RingMod {
                bias,
                rectify,
                frequency,
                spread,
                mix,
                modulation_mode,
                unknown2,
                unknown3,
            }),
            enabled,
            minimized,
        ))
    }
}

impl EffectWrite for RingMod {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_f32(self.frequency)?;
        writer.write_f32(self.spread)?;
        writer.write_f32(self.mix.get::<ratio>())?;
        writer.write_f32(self.bias)?;
        writer.write_f32(self.rectify)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        writer.write_u32(self.unknown3)?;

        if self.write_version() > 1032 {
            writer.write_u32(self.unknown2)?;
        }

        writer.write_string_and_length(self.modulation_mode.to_string())
    }

    fn write_version(&self) -> u32 {
        1043
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn crunch_time() {
        let preset =
            read_effect_preset("ring_mod", "ring_mod-crunch_time-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        assert_eq!(snapin.preset_name, "Crunch Time");
        assert_eq!(snapin.preset_path, vec!["factory", "Crunch Time.ksrm"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_ring_mod().unwrap();
        assert_relative_eq!(effect.bias, 0.18667, epsilon = 0.001);
        assert_relative_eq!(effect.rectify, -0.346667, epsilon = 0.001);
        assert_relative_eq!(effect.mix.get::<percent>(), 100.0, epsilon = 0.001);
        assert_relative_eq!(effect.frequency, 3835.668, epsilon = 0.001);
        assert_relative_eq!(effect.spread, 0.998, epsilon = 0.001);
        assert_eq!(effect.modulation_mode, ModulationMode::LowPassNoise);
    }

    #[test]
    fn default() {
        let effect = RingMod::default();
        assert_eq!(effect.bias, 0.0);
        assert_eq!(effect.rectify, 0.0);
        assert_eq!(effect.frequency, 440.0);
        assert_eq!(effect.spread, 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(effect.modulation_mode, ModulationMode::SineOscillator);
    }

    #[test]
    fn disabled() {
        let preset = read_effect_preset("ring_mod", "ring_mod-disabled-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_ring_mod().unwrap();
        assert_eq!(effect, &RingMod::default())
    }

    #[test]
    fn eq() {
        let effect = RingMod::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, RingMod::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &["ring_mod-1.8.13.phaseplant", "ring_mod-2.0.16.phaseplant"] {
            let preset = read_effect_preset("ring_mod", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_ring_mod().unwrap();
            assert_eq!(effect, &RingMod::default())
        }
    }

    #[test]
    fn modulation_mode() {
        let preset = read_effect_preset(
            "ring_mod",
            "ring_mod-mode_band_pass_noise-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_ring_mod().unwrap();
        assert_eq!(effect.modulation_mode, ModulationMode::BandPassNoise);

        let preset =
            read_effect_preset("ring_mod", "ring_mod-mode_low_pass_noise-2.0.16.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_ring_mod().unwrap();
        assert_eq!(effect.modulation_mode, ModulationMode::LowPassNoise);

        let preset =
            read_effect_preset("ring_mod", "ring_mod-mode_sideband-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_ring_mod().unwrap();
        assert_eq!(effect.modulation_mode, ModulationMode::Sideband);

        let preset = read_effect_preset(
            "ring_mod",
            "ring_mod-mode_sine_oscillator-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_ring_mod().unwrap();
        assert_eq!(effect.modulation_mode, ModulationMode::SineOscillator);
    }

    #[test]
    fn parts() {
        let preset = read_effect_preset(
            "ring_mod",
            "ring_mod-bias10-rect15-mix31-disabled-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_ring_mod().unwrap();
        assert_relative_eq!(effect.bias, 0.1, epsilon = 0.005);
        assert_relative_eq!(effect.rectify, 0.15, epsilon = 0.01);
        assert_relative_eq!(effect.mix.get::<percent>(), 31.2, epsilon = 0.01);

        let preset = read_effect_preset(
            "ring_mod",
            "ring_mod-freq432-spread10-self-minimized-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_ring_mod().unwrap();
        assert_relative_eq!(effect.frequency, 432.0, epsilon = 1.0);
        assert_relative_eq!(effect.spread, 0.1, epsilon = 0.001);
        assert_eq!(effect.modulation_mode, ModulationMode::OriginalSelf);
    }
}
