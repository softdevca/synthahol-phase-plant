//! [Slice EQ](https://kilohearts.com/docs/slice_eq) is a parametric equalizer.
//!
//! | Phase Plant Version | Effect Version |
//! |---------------------|----------------|
//! | 1.7.0 to 1.7.9      | 1019           |
//! | 1.8.0               | 1020           |
//! | 1.8.14              | 1021           |
//! | 2.0.16 to 2.1.0     | 1032           |

// Phase Plant 1.8.14 added saving the zoom and pan settings of the view.

use std::any::{type_name, Any};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, Write};

use log::trace;
use strum_macros::{Display, FromRepr};
use uom::si::f32::{Frequency, Ratio};
use uom::si::frequency::hertz;
use uom::si::ratio::percent;

use crate::effect::{EffectVersion, FalloffSpeed, FrequencyResolution, SpectrumView, StereoMode};
use crate::version::Version;
use crate::{Decibels, PhasePlantRelease, Snapin};

use super::super::io::*;
use super::{Effect, EffectMode};

#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum SliceEqFilterMode {
    // The discriminants correspond to the file format.
    LowCut = 1,
    LowShelf = 3,
    Peak = 4,
    Notch = 2,
    HighShelf = 5,
    HighCut = 0,
}

impl SliceEqFilterMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown Slice EQ filter mode {id}"),
            )
        })
    }
}

/// In Phase Plant the "high pass" and "low pass" modes of the
/// [Filter](crate::effect::Filter) effect are called "low cut" and "high cut" in
/// Slice EQ respectively.
impl Display for SliceEqFilterMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use SliceEqFilterMode::*;
        let name = match self {
            LowCut => "Low cut",
            LowShelf => "Low shelf",
            Peak => "Peak",
            Notch => "Notch",
            HighShelf => "High shelf",
            HighCut => "High cut",
        };
        f.write_str(name)
    }
}

#[derive(Copy, Clone, Debug, Display, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum ChannelMode {
    // The discriminants correspond to the file format.
    Both = 0,
    Mid = 1,
    Side = 2,
}

impl ChannelMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown channel mode {id}")))
    }
}

#[derive(Clone, Copy, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum OversampleMode {
    // The discriminants correspond to the file format.
    Off = 0,
    TimesTwo = 1,
    Auto = 2,
}

impl OversampleMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Unknown oversample mode {id}"),
            )
        })
    }
}

impl Display for OversampleMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            OversampleMode::Off => "Off",
            OversampleMode::TimesTwo => "2X",
            OversampleMode::Auto => "Auto",
        };
        f.write_str(msg)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliceEqFilter {
    pub id: u32,
    pub enabled: bool,
    pub channel_mode: ChannelMode,
    pub filter_mode: SliceEqFilterMode,
    pub cutoff_frequency: Frequency,
    pub gain: Decibels,
    pub q: f32,

    // TODO: Better wording for comment, or make code.
    /// Filter order times 6 is in db/Oct
    pub order: u32,
}

impl SliceEqFilter {
    pub const RESONANCE_MIN: f32 = 0.025;
    pub const RESONANCE_MAX: f32 = 40.0;

    /// This identifier is assigned when a filter is deleted.  After it's
    /// assigned the ID maybe modified by operations, such as deleting another
    /// filter.
    pub const ID_WHEN_DELETED: u32 = 99;

    pub const ORDER_TO_DB_PER_OCTAVE: [i32; 8] = [6, 12, 18, 24, 36, 48, 72, 96];

    pub fn order_db_per_octave(&self) -> Decibels {
        Decibels::new(Self::ORDER_TO_DB_PER_OCTAVE[self.order as usize] as f32)
    }
}

/// The default for a new filter as it is created in Phase Plant. The empty
/// slots in the preset have slightly different values.
impl Default for SliceEqFilter {
    fn default() -> Self {
        Self {
            id: 1,
            channel_mode: ChannelMode::Both,
            filter_mode: SliceEqFilterMode::LowCut,
            enabled: false,
            cutoff_frequency: Frequency::new::<hertz>(440.0),
            gain: Decibels::ZERO,
            q: 0.5,
            order: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliceEq {
    /// May contain up to [`SliceEq::FILTER_COUNT_MAX`] filters.
    pub filters: Vec<SliceEqFilter>,

    /// The global frequency offset was added in Phase Plant 1.8.0
    pub offset_semitones: f32,

    pub gain: Decibels,
    pub mix: Ratio,
    pub oversample_mode: OversampleMode,
    pub edit_mode: ChannelMode,
    pub stereo_mode: StereoMode,
    pub spectrum_view: SpectrumView,
}

impl SliceEq {
    pub const FILTER_COUNT_MAX: usize = 32;

    pub fn default_version() -> EffectVersion {
        1032
    }
}

impl Default for SliceEq {
    fn default() -> Self {
        Self {
            filters: Vec::with_capacity(Self::FILTER_COUNT_MAX),
            offset_semitones: 0.0,
            gain: Decibels::ZERO,
            mix: Ratio::new::<percent>(100.0),
            oversample_mode: OversampleMode::Auto,
            edit_mode: ChannelMode::Both,
            stereo_mode: StereoMode::MidSide,
            spectrum_view: Default::default(),
        }
    }
}

impl dyn Effect {
    #[must_use]
    pub fn as_slice_eq(&self) -> Option<&SliceEq> {
        self.downcast_ref::<SliceEq>()
    }
}

impl Effect for SliceEq {
    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> EffectMode {
        EffectMode::SliceEq
    }
}

impl EffectRead for SliceEq {
    fn read<R: Read + Seek>(
        reader: &mut PhasePlantReader<R>,
        effect_version: u32,
    ) -> io::Result<EffectReadReturn> {
        if effect_version < 1019 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Version {effect_version} of {} is not supported",
                    type_name::<Self>()
                ),
            ));
        }

        // 1210 for Phase Plant 1.7.9, 1218 for Phase Plant 1.8.0
        let _header_length = reader.read_u32()?; // FIXME: GUESS
        let _version_a_major = reader.read_u32()?;

        // TODO: Is this a SliceEQ preset file stored in the snapin?
        let version_b_patch = reader.read_u32()?;
        let version_b_minor = reader.read_u32()?;
        let version_b_major = reader.read_u32()?;
        let version_b = Version::new(version_b_major, version_b_minor, version_b_patch, 0);
        trace!("SliceEQ version B {version_b}");

        trace!("Before big skip 1 pos {}", reader.pos());
        reader.skip(1194)?;
        // reader.skip(header_length as i64)?;
        trace!("After big skip 1 pos {}", reader.pos());

        if effect_version >= 1020 {
            reader.skip(8)?;
        }

        let preset_name = reader.read_string_and_length()?;
        let preset_path = reader.read_path()?;
        let preset_edited =
            reader.is_release_at_least(PhasePlantRelease::V1_8_0) && reader.read_bool32()?;
        reader.skip(1)?; // 0 or 1

        let oversample_mode = OversampleMode::from_id(reader.read_u32()?)?;
        let enabled = reader.read_bool32()?;
        let minimized = reader.read_bool32()?;

        let mut edit_mode = ChannelMode::Both;
        let mut stereo_mode = StereoMode::MidSide;
        let mut effect_gain = Decibels::ZERO;
        let mut effect_mix = Ratio::new::<percent>(100.0);
        let mut spectrum_view = SpectrumView::default();

        let mut filters: Vec<SliceEqFilter> = Vec::with_capacity(SliceEq::FILTER_COUNT_MAX);
        for filter_index in 0..SliceEq::FILTER_COUNT_MAX {
            trace!(
                "slice eq: filter index {filter_index}: pos {}",
                reader.pos()
            );
            let exists = reader.read_bool32()?;
            let enabled = reader.read_bool32()?;
            let id = reader.read_u32()?;
            let filter_mode = SliceEqFilterMode::from_id(reader.read_u32()?)?;

            let order = reader.read_u32()?;
            if order >= SliceEqFilter::ORDER_TO_DB_PER_OCTAVE.len() as u32 {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Filter order {order} for filter index {filter_index} is out of range",),
                ));
            }

            let cutoff_frequency = reader.read_hertz()?;
            let q = reader.read_f32()?;
            let gain = reader.read_decibels_db()?;
            let mut channel_mode = ChannelMode::Both;

            // The channel mode for the first 8 filters are appended after the
            // 8th filter, from then on the channel mode is included with each
            // filter.
            if filter_index == 7 {
                effect_mix = reader.read_ratio()?;
                stereo_mode = StereoMode::from_id(reader.read_u32()?)?;
                spectrum_view.falloff_speed = FalloffSpeed::from_id(reader.read_u32()?)?;
                spectrum_view.frequency_resolution =
                    FrequencyResolution::from_id(reader.read_u32()?)?;
                edit_mode = ChannelMode::from_id(reader.read_u32()?)?;
                effect_gain = reader.read_decibels_db()?;

                for mode_index in 0..7 {
                    let channel_mode = ChannelMode::from_id(reader.read_u32()?)?;
                    if filters.len() > mode_index {
                        filters[mode_index].channel_mode = channel_mode;
                    }
                }
            }
            if filter_index >= 7 {
                channel_mode = ChannelMode::from_id(reader.read_u32()?)?;
            }

            if exists {
                filters.push(SliceEqFilter {
                    id,
                    channel_mode,
                    filter_mode,
                    enabled,
                    cutoff_frequency,
                    gain,
                    q,
                    order,
                });
            }
        }

        reader.expect_u32(1, "slice_eq_10")?;
        reader.expect_u32(0, "slice_eq_11")?;
        reader.expect_u32(0, "slice_eq_12")?;
        reader.expect_u32(0, "slice_eq_13")?;

        let offset_semitones = reader.read_f32()?;

        if effect_version >= 1021 {
            // Added in Phase Plant 1.8.14
            spectrum_view.y_min = reader.read_decibels_db()?;
            spectrum_view.y_max = reader.read_decibels_db()?;

            // The order of these two is reversed for an initially created
            // Slice EQ. Once a zoom or pan has been made the order is reversed.
            spectrum_view.x_min = reader.read_hertz()?;
            spectrum_view.x_max = reader.read_hertz()?;

            spectrum_view.normalize();
        }

        reader.expect_u32(0, "slice_eq_22")?;

        if effect_version >= 1020 {
            reader.expect_u32(0, "slice_eq_23")?;
        }

        let group_id = if effect_version >= 1030 {
            reader.read_snapin_position()?
        } else {
            None
        };

        let effect = Box::new(SliceEq {
            filters,
            offset_semitones,
            gain: effect_gain,
            mix: effect_mix,
            edit_mode,
            stereo_mode,
            oversample_mode,
            spectrum_view,
        });
        Ok(EffectReadReturn {
            effect,
            enabled,
            minimized,
            group_id,
            metadata: Default::default(),
            preset_name,
            preset_path,
            preset_edited,
        })
    }
}

impl EffectWrite for SliceEq {
    fn write<W: Write + Seek>(
        &self,
        _writer: &mut PhasePlantWriter<W>,
        _snapin: &Snapin,
    ) -> io::Result<()> {
        // TODO: Write Slice EQ
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
        let effect = SliceEq::default();
        assert_eq!(effect.offset_semitones, 0.0);
        assert_eq!(effect.gain.db(), 0.0);
        assert_eq!(effect.mix.get::<percent>(), 100.0);
        assert_eq!(effect.stereo_mode, StereoMode::MidSide);
        assert_eq!(effect.oversample_mode, OversampleMode::Auto);

        // Spectrum view
        assert_eq!(
            effect.spectrum_view.frequency_resolution,
            FrequencyResolution::ThirdOfOctave
        );
        assert_eq!(effect.spectrum_view.falloff_speed, FalloffSpeed::Medium);
        assert_relative_eq!(effect.spectrum_view.x_min.get::<hertz>(), 19.0);
        assert_relative_eq!(effect.spectrum_view.x_max.get::<hertz>(), 21000.0);

        // Filters
        for filter in &effect.filters {
            assert!(!filter.enabled);
            assert_eq!(filter.order_db_per_octave().db(), 6.0);
            assert_eq!(filter.cutoff_frequency, Frequency::new::<hertz>(440.0));
            assert_eq!(filter.gain, Decibels::new(0.0));
            assert_eq!(filter.q, 0.5);
        }
    }

    #[test]
    fn eq() {
        let effect = SliceEq::default();
        assert_eq!(effect, effect);
        assert_eq!(effect, SliceEq::default());
        assert!(!effect.box_eq(&Filter::default()));
    }

    #[test]
    fn filters() {
        let preset =
            read_effect_preset("slice_eq", "slice_eq-two_disabled_filters-2.1.0.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_slice_eq().unwrap();
        assert_eq!(effect.filters.len(), 2);
        for filter in &effect.filters {
            assert!(!filter.enabled);
        }

        let preset = read_effect_preset(
            "slice_eq",
            "slice_eq-filter_cut440-gain10-q3-12db_oct-mid-2.1.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_slice_eq().unwrap();
        assert_eq!(effect.filters.len(), 1);
        let filter = effect.filters[0];
        assert!(filter.enabled);
        assert_relative_eq!(
            filter.cutoff_frequency.get::<hertz>(),
            440.0,
            epsilon = 0.0001
        );
        assert_relative_eq!(filter.gain.db(), 10.0, epsilon = 0.0001);
        assert_relative_eq!(filter.q, 0.6489087);
        assert_relative_eq!(filter.order_db_per_octave().db(), 12.0);
        assert_eq!(filter.channel_mode, ChannelMode::Mid);
    }

    #[test]
    fn filter_channel_modes() {
        let preset =
            read_effect_preset("slice_eq", "slice_eq-filter_channel_modes-2.1.0.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_slice_eq().unwrap();
        assert_eq!(effect.filters[0].channel_mode, ChannelMode::Both);
        assert_eq!(effect.filters[1].channel_mode, ChannelMode::Mid);
        assert_eq!(effect.filters[2].channel_mode, ChannelMode::Side);
    }

    #[test]
    fn filter_modes() {
        let preset =
            read_effect_preset("slice_eq", "slice_eq-filter_modes-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_slice_eq().unwrap();
        assert_eq!(effect.filters[0].filter_mode, SliceEqFilterMode::LowCut);
        assert_eq!(effect.filters[1].filter_mode, SliceEqFilterMode::LowShelf);
        assert_eq!(effect.filters[2].filter_mode, SliceEqFilterMode::Peak);
        assert_eq!(effect.filters[3].filter_mode, SliceEqFilterMode::Notch);
        assert_eq!(effect.filters[4].filter_mode, SliceEqFilterMode::HighShelf);
        assert_eq!(effect.filters[5].filter_mode, SliceEqFilterMode::HighCut);
    }

    #[test]
    fn filter_q() {
        let preset = read_effect_preset("slice_eq", "slice_eq-filter_q-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_slice_eq().unwrap();

        // TODO: Trying to reverse engineer the denormalization
        // 0..=40 with midpoint 1.0 skew = 0.18790182470910757, same as JUCE
        // let max_linear = Decibels::new(40.0).linear(); // 100
        // let one_linear = Decibels::ONE.linear(); //  1.1220184543019633
        // let midpoint = one_linear;
        // let interval = NormalizedInterval::new(0.025, max_linear).with_skew_for_midpoint(midpoint);

        // const EXPECTED_Q: [f64; 13] = [0.025, 0.25, 0.75, 1.0, 1.5, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0];

        for index in 0..effect.filters.len() {
            let filter = &effect.filters[index];
            let _q = filter.q;
            // assert_relative_eq!(interval.denormalize(q as f64), EXPECTED_Q[index]);
        }
    }

    #[test]
    fn init() {
        for file in &[
            "slice_eq-1.7.0.phaseplant",
            "slice_eq-1.7.9.phaseplant",
            "slice_eq-1.8.0.phaseplant",
            "slice_eq-1.8.14.phaseplant",
            "slice_eq-2.0.16.phaseplant",
            "slice_eq-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("slice_eq", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.enabled);
            assert!(!snapin.minimized);
            let effect = snapin.effect.as_slice_eq().unwrap();

            // The effect can't be compared against the default because there
            // are slightly different values stored in the unused slots in the
            // preset.
            assert_relative_eq!(effect.offset_semitones, 0.0, epsilon = 0.000001);
            assert_relative_eq!(effect.gain.db(), 0.0, epsilon = 0.000001);
            assert_eq!(effect.mix.get::<percent>(), 100.0);
            assert_eq!(effect.stereo_mode, StereoMode::MidSide);
            assert_eq!(effect.oversample_mode, OversampleMode::Auto);

            // Spectrum view
            assert_eq!(
                effect.spectrum_view.frequency_resolution,
                FrequencyResolution::ThirdOfOctave
            );
            assert_eq!(effect.spectrum_view.falloff_speed, FalloffSpeed::Medium);
            assert_relative_eq!(effect.spectrum_view.x_min.get::<hertz>(), 19.0);
            assert_relative_eq!(effect.spectrum_view.x_max.get::<hertz>(), 21000.0);

            // Filters
            for filter in &effect.filters {
                assert!(!filter.enabled);
                assert_eq!(filter.cutoff_frequency, Frequency::new::<hertz>(440.0));
                assert_eq!(filter.gain, Decibels::new(0.0));
                assert_eq!(filter.q, 0.5);
            }
        }
    }

    #[test]
    fn filter_orders() {
        let preset =
            read_effect_preset("slice_eq", "slice_eq-filter_orders-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_slice_eq().unwrap();
        let filters = &effect.filters;
        assert_eq!(filters[0].order_db_per_octave().db(), 6.0);
        assert_eq!(filters[1].order_db_per_octave().db(), 12.0);
        assert_eq!(filters[2].order_db_per_octave().db(), 18.0);
        assert_eq!(filters[3].order_db_per_octave().db(), 24.0);
        assert_eq!(filters[4].order_db_per_octave().db(), 36.0);
        assert_eq!(filters[5].order_db_per_octave().db(), 48.0);
        assert_eq!(filters[6].order_db_per_octave().db(), 72.0);
        assert_eq!(filters[7].order_db_per_octave().db(), 96.0);
    }

    #[test]
    fn parts() {
        let preset =
            read_effect_preset("slice_eq", "slice_eq-exact-fast-x2-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_slice_eq().unwrap();
        assert_eq!(
            effect.spectrum_view.frequency_resolution,
            FrequencyResolution::Exact
        );
        assert_eq!(effect.spectrum_view.falloff_speed, FalloffSpeed::Fast);
        assert_eq!(effect.oversample_mode, OversampleMode::TimesTwo);

        let preset = read_effect_preset(
            "slice_eq",
            "slice_eq-offset12-gain5-mix25-disabled-2.1.0.phaseplant",
        )
        .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(!snapin.enabled);
        assert!(!snapin.minimized);
        let effect = snapin.effect.as_slice_eq().unwrap();
        assert_relative_eq!(effect.offset_semitones, 12.0, epsilon = 0.001);
        assert_relative_eq!(effect.gain.db(), 5.0, epsilon = 0.001);
        assert_relative_eq!(effect.mix.get::<percent>(), 25.0, epsilon = 0.001);

        let preset =
            read_effect_preset("slice_eq", "slice_eq-offset120-minimized-2.1.0.phaseplant")
                .unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        assert!(snapin.enabled);
        assert!(snapin.minimized);
        let effect = snapin.effect.as_slice_eq().unwrap();
        assert_relative_eq!(effect.offset_semitones, 120.0, epsilon = 0.001);
    }

    #[test]
    fn preset_name() {
        for file in &[
            "slice_eq-preset_name-1.7.0.phaseplant",
            "slice_eq-preset_name-1.8.0.phaseplant",
            "slice_eq-preset_name-2.1.0.phaseplant",
        ] {
            let preset = read_effect_preset("slice_eq", file).unwrap();
            let snapin = &preset.lanes[0].snapins[0];
            assert!(snapin.preset_name.contains("Preset Name"));
        }
    }

    #[test]
    fn zoom_and_pan() {
        let preset =
            read_effect_preset("slice_eq", "slice_eq-zoom_and_pan_x-2.1.0.phaseplant").unwrap();
        let snapin = &preset.lanes[0].snapins[0];
        let effect = snapin.effect.as_slice_eq().unwrap();
        assert_relative_eq!(effect.spectrum_view.y_min.db(), -8.5);
        assert_relative_eq!(effect.spectrum_view.y_max.db(), 8.5);
        assert_relative_eq!(effect.spectrum_view.x_min.get::<hertz>(), 62.896355);
        assert_relative_eq!(effect.spectrum_view.x_max.get::<hertz>(), 116.85098);
    }
}
