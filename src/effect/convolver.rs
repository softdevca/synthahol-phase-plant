//! [Convolver](https://kilohearts.com/products/convolver) is an effect that
//! applies an impulse response (IR) to audio.
//!
//! Convolver was added to Phase Plant in version 1.8.18.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 2.0.FIXME           | 1016           |
//! | 2.0.12              | 1017           |
//! | 2.0.16 to 2.1.0     | 1018           |

use std::any::{type_name, Any};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use uom::num::Zero;
use uom::si::f32::{Ratio, Time};
use uom::si::ratio::percent;

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Convolver {
    pub ir_name: Option<String>,
    pub ir_path: Vec<String>,

    /// Percentage of the IR length
    pub start: Ratio,
    pub end: Ratio,
    pub fade_in: Ratio,
    pub fade_out: Ratio,
    pub stretch: Ratio,

    /// Pre-delay
    pub delay: Time,

    pub sync: bool,
    pub tone: Ratio,
    pub feedback: Ratio,
    pub mix: Ratio,
    pub reverse: bool,
}

impl Default for Convolver {
    fn default() -> Self {
        Self {
            ir_name: None,
            ir_path: Vec::new(),
            start: Ratio::zero(),
            end: Ratio::new::<percent>(100.0),
            fade_in: Ratio::zero(),
            fade_out: Ratio::zero(),
            stretch: Ratio::new::<percent>(100.0),
            delay: Time::zero(),
            sync: false,
            tone: Ratio::zero(),
            feedback: Ratio::zero(),
            mix: Ratio::new::<percent>(100.0),
            reverse: false,
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_convolver(&self) -> Option<&Convolver> {
        self.downcast_ref::<Convolver>()
    }
}

impl Effect for Convolver {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::Convolver
    }
}

impl EffectRead for Convolver {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1017 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        let mix = reader.read_ratio()?;
        let stretch = reader.read_ratio()?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        reader.expect_u32(0, "convolver_unknown_5")?;
        reader.expect_u32(0, "convolver_unknown_6")?;

        let end = reader.read_ratio()?;
        let fade_out = reader.read_ratio()?;
        let feedback = reader.read_ratio()?;
        let tone = reader.read_ratio()?;
        let start = reader.read_ratio()?;
        let fade_in = reader.read_ratio()?;
        let delay = reader.read_seconds()?;

        reader.expect_u32(0, "convolver_unknown_7")?;
        reader.expect_u32(4, "convolver_unknown_8")?;

        let sync = reader.read_bool32()?;
        let reverse = reader.read_bool32()?;

        reader.expect_u32(0, "convolver_unknown_9")?;

        let ir_name = reader.read_string_and_length()?;
        let mut ir_path = Vec::new();
        let path_header = reader.read_block_header()?;
        if path_header.is_used() {
            ir_path = match reader.read_string_and_length()? {
                None => Vec::new(),
                Some(path) => vec![path],
            };
            let header_mode_id = path_header.mode_id().expect("convolver IR header mode");
            match header_mode_id {
                3 => reader.expect_u8(0, "convolver_block_unknown_1")?,
                2 => (),
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        format!("Unsupported convolver IR block mode {header_mode_id}"),
                    ))
                }
            }
        }

        let effect = Convolver {
            ir_name,
            ir_path,
            start,
            end,
            fade_in,
            fade_out,
            stretch,
            delay,
            sync,
            tone,
            feedback,
            mix,
            reverse,
        };
        Ok(EffectReadReturn::new(Box::new(effect), enabled, minimized))
    }
}

impl EffectWrite for Convolver {
    fn write<W: Write + Seek>(
        &self,
        _writer: &mut PhasePlantWriter<W>,
        _enabled: bool,
        _minimized: bool,
    ) -> io::Result<()> {
        todo!()
    }

    fn write_version(&self) -> u32 {
        1018
    }
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;
    use uom::si::f32::{Ratio, Time};
    use uom::si::ratio::percent;
    use uom::si::time::millisecond;

    use crate::effect::Filter;
    use crate::test::read_effect_preset;

    use super::*;

    #[test]
    fn art_museum() {
        let preset =
            read_effect_preset("convolver", "convolver-art_museum-2.0.12.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        assert!(!snapin.preset_edited);
        assert!(snapin.preset_name.is_empty());
        assert!(snapin.preset_path.is_empty());
        let effect = snapin.effect.as_convolver().unwrap();
        assert_eq!(effect.ir_name, Some("Art Museum".to_owned()));
        assert_eq!(
            effect.ir_path,
            vec!["factory/Impulse Responses/Spaces Real/Art Museum.flac"]
        );
        assert_eq!(effect.start, Ratio::zero());
        assert_eq!(effect.end, Ratio::new::<percent>(100.0));
        assert_eq!(effect.fade_in, Ratio::zero());
        assert_eq!(effect.fade_out, Ratio::zero());
        assert_eq!(effect.stretch, Ratio::new::<percent>(100.0));
        assert_eq!(effect.delay, Time::zero());
        assert!(!effect.sync);
        assert_eq!(effect.tone.get::<percent>(), 0.0);
        assert_eq!(effect.feedback.get::<percent>(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert!(!effect.reverse);
    }

    #[test]
    fn default() {
        let effect = Convolver::default();
        assert_eq!(effect.start, Ratio::zero());
        assert_eq!(effect.end, Ratio::new::<percent>(100.0));
        assert_eq!(effect.fade_in, Ratio::zero());
        assert_eq!(effect.fade_out, Ratio::zero());
        assert_eq!(effect.stretch, Ratio::new::<percent>(100.0));
        assert_eq!(effect.delay, Time::zero());
        assert!(!effect.sync);
        assert_eq!(effect.tone.get::<percent>(), 0.0);
        assert_eq!(effect.feedback.get::<percent>(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert!(!effect.reverse);
    }

    #[test]
    fn disabled() {
        let preset =
            read_effect_preset("convolver", "convolver-disabled-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = Convolver::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, Convolver::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn init() {
        for file in &[
            "convolver-2.0.12.phaseplant",
            "convolver-2.0.16.phaseplant",
            "convolver-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("convolver", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            assert_eq!(snapin.preset_name, "".to_string());
            assert_eq!(snapin.preset_path, Vec::<String>::new());
            let effect = snapin.effect.as_convolver().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("convolver", "convolver-minimized-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn parts() {
        let preset =
            read_effect_preset("convolver", "convolver-delay50-tone25-2.0.12.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_convolver().unwrap();
        assert_relative_eq!(effect.delay.get::<millisecond>(), 50.0);
        assert_eq!(effect.tone.get::<percent>(), 25.0);
        assert!(!effect.sync);

        let preset = read_effect_preset(
            "convolver",
            "convolver-fade_in25-stretch45-fade_out75-2.0.12.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_convolver().unwrap();
        assert_relative_eq!(effect.fade_in.get::<percent>(), 25.0, epsilon = 0.01);
        assert_relative_eq!(effect.fade_out.get::<percent>(), 75.0, epsilon = 0.01);
        assert_eq!(effect.stretch.get::<percent>(), 45.0);

        let preset =
            read_effect_preset("convolver", "convolver-feedback25-mix50-2.0.12.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_convolver().unwrap();
        assert_eq!(effect.feedback.get::<percent>(), 25.0);
        assert_eq!(effect.mix.get::<percent>(), 50.0);

        let preset = read_effect_preset(
            "convolver",
            "convolver-feedback75-delay25-reverse-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_convolver().unwrap();
        assert!(effect.reverse);
        assert_relative_eq!(effect.feedback.get::<percent>(), 75.4, epsilon = 0.1);
        assert_relative_eq!(effect.delay.get::<millisecond>(), 25.0, epsilon = 0.1);

        let preset =
            read_effect_preset("convolver", "convolver-start5-end80-2.0.12.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_convolver().unwrap();
        assert_relative_eq!(effect.start.get::<percent>(), 5.0, epsilon = 0.001);
        assert_relative_eq!(effect.end.get::<percent>(), 80.0, epsilon = 0.001);
    }

    #[test]
    fn reverse_reverb() {
        let preset =
            read_effect_preset("convolver", "convolver-reverse_reverb-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        assert!(!snapin.preset_edited);
        assert_eq!(snapin.preset_name, "Reverse Reverb");
        assert_eq!(snapin.preset_path, vec!["factory", "Reverse Reverb.ksco"]);

        let effect = snapin.effect.as_convolver().unwrap();
        assert_eq!(effect.ir_name, Some("Modern Church".to_string()));
        assert_eq!(
            effect.ir_path,
            vec!["factory/Impulse Responses/Spaces Real/Modern Church.flac"]
        );
        assert_relative_eq!(effect.start.get::<percent>(), 50.0, epsilon = 0.0001);
        assert_relative_eq!(effect.end.get::<percent>(), 100.0, epsilon = 0.0001);
        assert_relative_eq!(effect.fade_in.get::<percent>(), 10.0, epsilon = 0.0001);
        assert_relative_eq!(effect.fade_out.get::<percent>(), 10.0, epsilon = 0.0001);
        assert_relative_eq!(effect.stretch.get::<percent>(), 100.0, epsilon = 0.0001);
        assert_eq!(effect.delay, Time::zero());
        assert!(!effect.sync);
        assert_eq!(effect.tone.get::<percent>(), 0.0);
        assert_eq!(effect.feedback.get::<percent>(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert!(effect.reverse);
    }

    #[test]
    fn sync() {
        let preset = read_effect_preset("convolver", "convolver-sync-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_convolver().unwrap();
        assert!(effect.sync);
    }
}
