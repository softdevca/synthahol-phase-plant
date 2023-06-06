//! [Bitcrush](https://kilohearts.com/products/bitcrush) simulates lo-fi sound
//! sources. It is spelled with a lowercase "C".
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.8.5 to 1.8.13     | 1038           |
//! | 2.0.12              | 1048           |
//! | 2.0.16              | 1049           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::{Frequency, Ratio};
use uom::si::frequency::hertz;
use uom::si::ratio::percent;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Bitcrush {
    pub frequency: Frequency,
    pub quantize: Ratio,
    pub bits: f32,
    pub dither: Ratio,
    pub adc_quality: Ratio,
    pub dac_quality: Ratio,
    pub mix: Ratio,
}

impl Bitcrush {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Bitcrush {
    fn default() -> Self {
        Self {
            frequency: Frequency::new::<hertz>(6000.0),
            quantize: Ratio::new::<percent>(100.0),
            bits: 16.0,
            dither: Ratio::zero(),
            adc_quality: Ratio::new::<percent>(100.0),
            dac_quality: Ratio::zero(),
            mix: Ratio::new::<percent>(100.0),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_bitcrush(&self) -> Option<&Bitcrush> {
        self.downcast_ref::<Bitcrush>()
    }
}

impl Effect for Bitcrush {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Bitcrush
    }
}

impl EffectRead for Bitcrush {
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

        let bits = reader.read_f32()?;
        if bits < 0.0 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unexpected number of bits ({bits})"),
            ));
        }

        let adc_quality = reader.read_ratio()?;
        let dac_quality = reader.read_ratio()?;
        let dither = reader.read_ratio()?;
        let quantize = reader.read_ratio()?;
        let mix = reader.read_ratio()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "bitcrush_unknown_1")?;
        reader.expect_u32(0, "bitcrush_unknown_2")?;
        if effect_version >= 1048 {
            reader.expect_u32(0, "bitcrush_unknown_3")?;
        }

        let effect = Box::new(Bitcrush {
            frequency,
            quantize,
            bits,
            dither,
            adc_quality,
            dac_quality,
            mix,
        });

        Ok(EffectReadReturn::new(effect, enabled, minimized))
    }
}

impl EffectWrite for Bitcrush {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool32(enabled)?;
        writer.write_hertz(self.frequency)?;
        writer.write_f32(self.bits)?;
        writer.write_ratio(self.adc_quality)?;
        writer.write_ratio(self.dac_quality)?;
        writer.write_ratio(self.dither)?;
        writer.write_ratio(self.quantize)?;
        writer.write_ratio(self.mix)?;
        writer.write_bool32(minimized)?;

        writer.write_u32(0)?;
        writer.write_u32(0)?;
        if self.write_version() >= 1048 {
            writer.write_u32(0)?;
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

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn defaults() {
        let effect = Bitcrush::default();
        assert_relative_eq!(effect.frequency.get::<hertz>(), 6000.0, epsilon = 3.0);
        assert_eq!(effect.quantize.get::<percent>(), 100.0);
        assert_eq!(effect.bits, 16.0);
        assert_eq!(effect.dither.get::<percent>(), 0.0);
        assert_eq!(effect.adc_quality.get::<percent>(), 100.0);
        assert_eq!(effect.dac_quality.get::<percent>(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
    }

    #[test]
    fn disabled() {
        let preset = read_effect_preset("bitcrush", "bitcrush-disabled-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = Bitcrush::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Bitcrush::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &["bitcrush-1.8.13.phaseplant", "bitcrush-2.0.12.phaseplant"] {
            let preset = read_effect_preset("bitcrush", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            assert_eq!(snapin.position, 1);
            let effect = snapin.effect.as_bitcrush().unwrap();
            assert_relative_eq!(effect.frequency.get::<hertz>(), 6000.0, epsilon = 3.0);
            assert_eq!(effect.quantize.get::<percent>(), 100.0);
            assert_eq!(effect.bits, 16.0);
            assert_eq!(effect.dither.get::<percent>(), 0.0);
            assert_eq!(effect.adc_quality.get::<percent>(), 100.0);
            assert_eq!(effect.dac_quality.get::<percent>(), 0.0);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("bitcrush", "bitcrush-minimized-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts_version_1() {
        let preset = read_effect_preset(
            "bitcrush",
            "bitcrush-172hz-quant50%-8bits-1.8.13.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_bitcrush().unwrap();
        assert_relative_eq!(effect.frequency.get::<hertz>(), 172.0, epsilon = 0.3);
        assert_eq!(effect.quantize.get::<percent>(), 50.0);
        assert_eq!(effect.bits, 8.0);

        let preset =
            read_effect_preset("bitcrush", "bitcrush-dacq25%-mix75%-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_bitcrush().unwrap();
        assert_eq!(effect.mix.get::<percent>(), 75.0);
        assert_eq!(effect.dac_quality.get::<percent>(), 25.0);

        let preset =
            read_effect_preset("bitcrush", "bitcrush-dither10%-adcq66%-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_bitcrush().unwrap();
        assert_eq!(effect.dither.get::<percent>(), 10.0);
        assert_eq!(effect.adc_quality.get::<percent>(), 66.0);

        let preset =
            read_effect_preset("bitcrush", "bitcrush-emulation-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Emulation");
        assert_eq!(snapin.preset_path, vec!["factory", "Emulation.ksbc"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_bitcrush().unwrap();
        assert_eq!(effect.quantize.get::<percent>(), 100.0);
    }

    #[test]
    fn parts_version_2() {
        let preset = read_effect_preset(
            "bitcrush",
            "bitcrush-172hz-quant50%-8bits-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_bitcrush().unwrap();
        assert_relative_eq!(effect.frequency.get::<hertz>(), 172.0, epsilon = 0.3);
        assert_eq!(effect.quantize.get::<percent>(), 50.0);
        assert_eq!(effect.bits, 8.0);
    }
}
