//! [Carve EQ](https://kilohearts.com/docs/carve_eq) is a 31-band equalizer.
//!
//! The user interface scale is not stored with the preset.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.7.0 to 1.8.12     | 1022           |
//! | 1.8.14              | 1023           |
//! | 2.0.16 to 2.1.0     | 1034           |

// Phase Plant 1.8.14 added saving the zoom and pan settings of the view.

use std::any::{type_name, Any};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use log::trace;
use strum_macros::FromRepr;
use uom::num::Zero;
use uom::si::f32::{Frequency, Ratio};
use uom::si::frequency::hertz;
use uom::si::ratio::{percent, ratio};

use crate::version::Version;
use crate::Decibels;

use super::super::io::*;
use super::{Effect, EffectMode};

pub type CarveEqShape = [[f32; CarveEq::BAND_COUNT]; CarveEq::CHANNEL_COUNT];

#[derive(Clone, Copy, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum StereoMode {
    // The discriminants correspond to the file format.
    LeftRight = 0,
    MidSide = 1,
}

impl StereoMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown stereo mode {id}")))
    }
}

impl Display for StereoMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            StereoMode::MidSide => "Mid/Side",
            StereoMode::LeftRight => "Left/Right",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Copy, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum FalloffSpeed {
    // The discriminants correspond to the file format.
    Off = 0,
    Slow = 1,
    Medium = 2,
    Fast = 3,
}

impl FalloffSpeed {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown Carve EQ falloff speed {id}"),
            )
        })
    }
}

impl Display for FalloffSpeed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            FalloffSpeed::Off => "Off",
            FalloffSpeed::Slow => "Slow",
            FalloffSpeed::Medium => "Medium",
            FalloffSpeed::Fast => "Fast",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Copy, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum FrequencyResolution {
    // The discriminants correspond to the file format.
    Exact = 0,
    Semitone = 1,
    ThirdOfOctave = 2,
    Octave = 3,
}

impl FrequencyResolution {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown Carve EQ frequency resolution {id}"),
            )
        })
    }
}

impl Display for FrequencyResolution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            FrequencyResolution::Exact => "Exact",
            FrequencyResolution::Semitone => "Semitone",
            FrequencyResolution::ThirdOfOctave => "1/3 Octave",
            FrequencyResolution::Octave => "Octave",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumView {
    pub frequency_resolution: FrequencyResolution,
    pub falloff_speed: FalloffSpeed,
    pub x_min: Frequency,
    pub x_max: Frequency,
    pub y_min: Decibels,
    pub y_max: Decibels,
}

impl SpectrumView {
    // Frequency::new is not a const fn yet.
    // pub const MIN_X: Frequency = Frequency::new::<hertz>(19.0);
    // pub const MAX_X: Frequency = Frequency::new::<hertz>(21000.0);
    pub const MIN_X_FREQUENCY: f32 = 19.0;
    pub const MAX_X_FREQUENCY: f32 = 21000.0;
    pub const MIN_Y: Decibels = Decibels::new(-31.5);
    pub const MAX_Y: Decibels = Decibels::new(31.5);

    /// Swap the X and Y ranges if to ensure the largest is the max.
    pub fn normalize(&mut self) {
        if self.x_min > self.x_max {
            std::mem::swap(&mut self.x_min, &mut self.x_max);
        }
        if self.y_min > self.y_max {
            std::mem::swap(&mut self.y_min, &mut self.y_max);
        }
    }
}

impl Default for SpectrumView {
    fn default() -> Self {
        Self {
            frequency_resolution: FrequencyResolution::ThirdOfOctave,
            falloff_speed: FalloffSpeed::Medium,
            x_min: Frequency::new::<hertz>(Self::MIN_X_FREQUENCY),
            x_max: Frequency::new::<hertz>(Self::MAX_X_FREQUENCY),
            y_min: Self::MIN_Y,
            y_max: Self::MAX_Y,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CarveEq {
    pub gain: Decibels,
    pub mix: Ratio,
    pub stereo_mode: StereoMode,
    pub spectrum_view: SpectrumView,
    pub shape: CarveEqShape,
}

impl CarveEq {
    pub const BAND_COUNT: usize = 31;
    pub const CHANNEL_COUNT: usize = 2;
}

impl Default for CarveEq {
    fn default() -> Self {
        Self {
            gain: Decibels::ZERO,
            mix: Ratio::new::<percent>(100.0),
            stereo_mode: StereoMode::MidSide,
            spectrum_view: SpectrumView {
                // Carve EQ doesn't have zooming.
                x_min: Frequency::zero(),
                x_max: Frequency::zero(),
                ..Default::default()
            },
            shape: [[0_f32; Self::BAND_COUNT]; Self::CHANNEL_COUNT],
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_carve_eq(&self) -> Option<&CarveEq> {
        self.downcast_ref::<CarveEq>()
    }
}

impl Effect for CarveEq {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::CarveEq
    }
}

impl EffectRead for CarveEq {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1022 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        // Length to the preset name.
        let header_length = reader.read_u32()?;
        let header_start_pos = reader.stream_position()?;

        // Two version numbers.
        // TODO: Unsure of the purpose. Maybe one is the version of HeartCore?
        // Snapin version vs. independent product version?

        let version_b_major = reader.read_u32()?;
        let version_b_patch = reader.read_u32()?;
        let version_b_minor = reader.read_u32()?;
        let version_b = Version::new(version_b_major, version_b_minor, version_b_patch, 0);
        trace!("carve eq: effect version {effect_version} version_b {version_b}");

        // FIXME: this "Json" has a leading NULL.  Maybe a bool8?
        let json = reader.read_string_and_length()?;
        trace!("carve eq: json {json:?}");

        reader.expect_u32(0, "carve_eq_unknown_1")?;
        reader.expect_f32(1.0, "carve_eq_unknown_2")?;
        reader.expect_bool32(true, "carve_eq_unknown_3")?;

        for index in 0..124 {
            reader.expect_u8(0, &format!("carve_eq_unknown_4_{index}"))?;
        }

        reader.expect_bool32(true, "carve_unknown_5")?;

        for index in 0..128 {
            reader.expect_u8(0, &format!("carve_eq_unknown_6_{index}"))?;
        }

        reader.expect_u32(0, "carve_eq_unknown_7")?;

        if effect_version > 1022 {
            reader.skip(4)?;
            reader.expect_u32(0, "carve_eq_unknown_9")?;
        }

        let header_remaining =
            header_length as i64 - (reader.stream_position()? - header_start_pos) as i64;
        trace!("carve eq: header remaining {header_remaining}");
        reader.skip(header_remaining)?;

        let preset_name = reader.read_string_and_length()?;
        trace!("carve eq: preset name {preset_name:?}");

        let mut preset_path = vec![];
        if version_b_major > 5 {
            // if effect_version > 1022 {
            preset_path = reader.read_path()?;
            trace!("carve eq: preset path {preset_path:?}");
        }

        let preset_edited = reader.read_bool32()?;

        reader.expect_u8(0, "carve_eq_path_1")?;

        let mix = Ratio::new::<ratio>(reader.read_f32()?);
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        let mut shape = [[0_f32; Self::BAND_COUNT]; Self::CHANNEL_COUNT];
        for band_idx in 0..Self::BAND_COUNT {
            shape[0][band_idx] = reader.read_f32()?;
        }

        let stereo_mode = StereoMode::from_id(reader.read_u32()?)?;

        let falloff_speed = FalloffSpeed::from_id(reader.read_u32()?)?;
        let frequency_resolution = FrequencyResolution::from_id(reader.read_u32()?)?;
        let mut spectrum_view = SpectrumView {
            frequency_resolution,
            falloff_speed,

            // Carve EQ doesn't have zooming.
            x_min: Frequency::zero(),
            x_max: Frequency::zero(),
            ..SpectrumView::default()
        };

        let _unknown = reader.read_u32()?;
        let gain = Decibels::new(reader.read_f32()?);

        for i in 0..Self::BAND_COUNT {
            shape[1][i] = reader.read_f32()?;
        }

        reader.skip(12 + 8)?;

        if effect_version > 1022 {
            // Added in Phase Plant 1.8.14
            spectrum_view.y_min = Decibels::new(reader.read_f32()?);
            spectrum_view.y_max = Decibels::new(reader.read_f32()?);

            // The order of these two is reversed for an initially created
            // Slice EQ. Once a zoom or pan has been made the order is reversed.
            spectrum_view.x_min = Frequency::new::<hertz>(reader.read_f32()?);
            spectrum_view.x_max = Frequency::new::<hertz>(reader.read_f32()?);

            spectrum_view.normalize();
        }

        reader.expect_u32(0, "carve_eq_unknown_22")?;
        if effect_version < 1023 {
            reader.expect_u32(0, "carve_eq_unknown_23")?;
            reader.expect_u32(0, "carve_eq_unknown_24")?;
        } else if effect_version >= 1034 {
            reader.expect_u32(0, "carve_eq_unknown_23")?;
        }

        let effect = Box::new(CarveEq {
            gain,
            mix,
            stereo_mode,
            spectrum_view,
            shape,
        });
        Ok(EffectReadReturn {
            effect,
            enabled,
            minimized,
            metadata: Default::default(),
            preset_name,
            preset_path,
            preset_edited,
        })
    }
}

impl EffectWrite for CarveEq {
    fn write<W: Write + Seek>(
        &self,
        writer: &mut PhasePlantWriter<W>,
        enabled: bool,
        _minimized: bool,
    ) -> io::Result<()> {
        writer.write_bool8(enabled)?; // FIXME: Duplicate
        writer.write_u8(0)?;
        writer.write_u8(0)?;
        writer.write_u32(0)?;
        writer.write_u32(0)?;
        writer.write_u32(0)?;
        writer.write_u8(0)?;
        writer.write_f32(self.mix.get::<ratio>())?;
        writer.write_f32(self.gain.db())?;
        writer.write_bool32(enabled)?;

        for _ in 0..32 {
            writer.skip(19)?;
        }

        Ok(())
    }

    fn write_version(&self) -> u32 {
        1034
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
        let effect = CarveEq::default();
        assert_eq!(effect.gain.db(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(
            effect.shape,
            [[0_f32; CarveEq::BAND_COUNT]; CarveEq::CHANNEL_COUNT]
        );
        assert_eq!(effect.stereo_mode, StereoMode::MidSide);
        assert_eq!(
            effect.spectrum_view.frequency_resolution,
            FrequencyResolution::ThirdOfOctave
        );
        assert_eq!(effect.spectrum_view.falloff_speed, FalloffSpeed::Medium);
    }

    #[test]
    fn disabled() {
        let preset = read_effect_preset("carve_eq", "carve_eq-disabled-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
    }

    #[test]
    fn eq() {
        let effect = CarveEq::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, CarveEq::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn gain_mix() {
        let preset =
            read_effect_preset("carve_eq", "carve_eq-gain5-mix70-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_carve_eq().unwrap();
        assert_eq!(effect.gain.db(), 5.0);
        assert_eq!(effect.mix.get::<percent>(), 70.0);
    }

    #[test]
    fn init() {
        for file in &[
            "carve_eq-1.7.0.phaseplant",
            "carve_eq-1.7.7.phaseplant",
            "carve_eq-1.7.11.phaseplant",
            "carve_eq-1.8.0.phaseplant",
            "carve_eq-1.8.13.phaseplant",
            "carve_eq-1.8.14.phaseplant",
            "carve_eq-2.0.16.phaseplant",
            "carve_eq-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("carve_eq", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_carve_eq().unwrap();
            assert_eq!(effect, &Default::default());
        }
    }

    #[test]
    fn minimized() {
        let preset =
            read_effect_preset("carve_eq", "carve_eq-minimized-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
    }

    #[test]
    fn mode_spectrum_view() {
        let preset = read_effect_preset(
            "carve_eq",
            "carve_eq-left_right-semitone-medium-2.0.16.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_carve_eq().unwrap();
        assert_eq!(effect.stereo_mode, StereoMode::LeftRight);
        assert_eq!(
            effect.spectrum_view.frequency_resolution,
            FrequencyResolution::Semitone
        );
        assert_eq!(effect.spectrum_view.falloff_speed, FalloffSpeed::Medium);
    }

    #[test]
    fn pan() {
        let preset = read_effect_preset("carve_eq", "carve_eq-pan_y-1.8.14.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_carve_eq().unwrap();
        assert_relative_eq!(effect.spectrum_view.x_min.get::<hertz>(), 0.0);
        assert_relative_eq!(effect.spectrum_view.x_max.get::<hertz>(), 0.0);
        assert_relative_eq!(effect.spectrum_view.y_min.db(), -2.96855, epsilon = 0.0001);
        assert_relative_eq!(effect.spectrum_view.y_max.db(), 14.03145, epsilon = 0.0001);
    }

    #[test]
    fn preset_name() {
        for file in &[
            "carve_eq-preset_name-1.7.0.phaseplant",
            "carve_eq-preset_name-1.7.7.phaseplant",
            "carve_eq-preset_name-1.8.0.phaseplant",
            "carve_eq-preset_name-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("carve_eq", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.preset_name.contains("Preset Name"));
        }
    }

    /// Test a shape created by hand.
    #[test]
    fn shape() {
        let preset = read_effect_preset("carve_eq", "carve_eq-shape-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "");
        assert!(snapin.preset_path.is_empty());
        assert!(snapin.preset_edited);
        let effect = snapin.effect.as_carve_eq().unwrap();
        let shape = effect.shape;
        assert_relative_eq!(shape[0][0], 0.5, epsilon = 0.1);
        assert_relative_eq!(shape[0][1], 1.0, epsilon = 0.1);
        assert_relative_eq!(shape[0][2], 1.5, epsilon = 0.1);
        assert_relative_eq!(shape[0][3], 2.0, epsilon = 0.1);
        assert_relative_eq!(shape[0][28], 5.0, epsilon = 0.1);
        assert_relative_eq!(shape[0][29], 8.0, epsilon = 0.1);
        assert_relative_eq!(shape[0][30], 10.0, epsilon = 0.1);
        assert_relative_eq!(shape[1][0], -0.5, epsilon = 0.1);
        assert_relative_eq!(shape[1][1], -1.0, epsilon = 0.1);
        assert_relative_eq!(shape[1][2], -1.5, epsilon = 0.1);
        assert_relative_eq!(shape[1][3], -2.0, epsilon = 0.1);
        assert_relative_eq!(shape[1][28], -5.0, epsilon = 0.1);
        assert_relative_eq!(shape[1][29], -8.0, epsilon = 0.1);
        assert_relative_eq!(shape[1][30], -10.0, epsilon = 0.1);
    }

    #[test]
    fn stereo_comb() {
        let preset =
            read_effect_preset("carve_eq", "carve_eq-stereo_comb-1.7.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.preset_name.contains("Stereo Comb"));
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_carve_eq().unwrap();
        assert_eq!(effect.gain.db(), 0.0);

        let preset = read_effect_preset(
            "carve_eq",
            "carve_eq-stereo_comb-mix_edited-1.7.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.preset_name.contains("Stereo Comb"));
        assert_eq!(effect.gain.db(), 0.0);
        assert!(snapin.preset_edited);

        let preset =
            read_effect_preset("carve_eq", "carve_eq-stereo_comb-1.8.13.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Stereo Comb");
        assert_eq!(snapin.preset_path, vec!["factory", "Stereo Comb.ksge"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_carve_eq().unwrap();
        assert_eq!(effect.gain.db(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert!(snapin
            .metadata
            .description
            .clone()
            .unwrap_or_default()
            .contains("Adds stereo effect"));

        let preset =
            read_effect_preset("carve_eq", "carve_eq-stereo_comb-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Stereo Comb");
        assert_eq!(snapin.preset_path, vec!["factory", "Stereo Comb.ksge"]);
        assert!(!snapin.preset_edited);
        let effect = snapin.effect.as_carve_eq().unwrap();
        assert_eq!(effect.mix.get::<percent>(), 100.0);

        let preset =
            read_effect_preset("carve_eq", "carve_eq-stereo_comb-edit-2.0.16.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Stereo Comb");
        assert_eq!(snapin.preset_path, vec!["factory", "Stereo Comb.ksge"]);
        assert!(snapin.preset_edited);

        let preset =
            read_effect_preset("carve_eq", "carve_eq-stereo_comb-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Stereo Comb");
        assert_eq!(snapin.preset_path, vec!["factory", "Stereo Comb.ksge"]);
        assert!(!snapin.preset_edited);
        assert!(snapin
            .metadata
            .description
            .clone()
            .unwrap_or_default()
            .contains("Adds stereo effect"));
        let effect = snapin.effect.as_carve_eq().unwrap();
        assert_eq!(effect.gain.db(), 0.0);

        let preset = read_effect_preset(
            "carve_eq",
            "carve_eq-stereo_comb-gain_edited-2.1.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert_eq!(snapin.preset_name, "Stereo Comb");
        assert_eq!(snapin.preset_path, vec!["factory", "Stereo Comb.ksge"]);
        assert!(snapin.preset_edited);
        let effect = snapin.effect.as_carve_eq().unwrap();
        assert_relative_eq!(effect.gain.db(), 4.476192, epsilon = 0.0001);
    }
}
