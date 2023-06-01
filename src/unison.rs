use std::io::{Error, ErrorKind};

use strum_macros::FromRepr;

#[derive(Copy, Clone, Debug, FromRepr, Eq, PartialEq)]
#[repr(u32)]
pub enum UnisonMode {
    // The discriminants correspond to the file format.

    // Unison
    Hard = 0,
    Smooth = 1,
    Synthetic = 2,

    // Creative
    FreqStack = 15,
    PitchStack = 16,
    Shepard = 17,

    // Chords
    Octaves = 3,
    Fifths = 4,
    Minor = 5,
    Minor7 = 7,
    MinorMaj7 = 9,
    Major = 6,
    Major7 = 8,
    MajorMaj7 = 10,
    Sus2 = 11,
    Sus4 = 12,
    Dim = 13,
    Harmonics = 14,
}

impl UnisonMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        Self::from_repr(id)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Unknown unison mode {id}")))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Unison {
    pub enabled: bool,
    pub voices: u32,
    pub mode: UnisonMode,

    /// In cents. The Phase Plant interface only shows on decimal digit even though it
    /// stores more.
    pub detune: f32,

    pub spread: f32,
    pub blend: f32,

    /// Also known as "balance"
    pub bias: f32,
}

impl Unison {
    pub const VOICES_MAX: u32 = 8; // As of Phase Plant 1.8.20
}

impl Default for Unison {
    fn default() -> Self {
        Unison {
            enabled: false,
            voices: 4,
            mode: UnisonMode::Smooth,
            detune: 25.0,
            spread: 0.0,
            blend: 1.0,
            bias: 0.0,
        }
    }
}
