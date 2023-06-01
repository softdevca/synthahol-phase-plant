use std::fmt::{Display, Formatter};
use std::io::{Error, ErrorKind};

use strum_macros::FromRepr;

// TODO: Why are there separate curve points for LFOs and Curves?

/// Point on a curve, such as those used for LFOs and curves.
#[derive(Clone, Debug, PartialEq)]
pub struct CurvePoint {
    pub mode: CurvePointMode,
    pub x: f32,
    pub y: f32,
    pub curve_x: f32,
    pub curve_y: f32,
}

impl CurvePoint {
    pub fn new_sharp(x: f32, y: f32, curve_x: f32, curve_y: f32) -> Self {
        Self {
            mode: CurvePointMode::Sharp,
            x,
            y,
            curve_x,
            curve_y,
        }
    }

    pub fn new_smooth(x: f32, y: f32, curve_x: f32, curve_y: f32) -> Self {
        Self {
            mode: CurvePointMode::Smooth,
            x,
            y,
            curve_x,
            curve_y,
        }
    }

    pub fn is_sharp(&self) -> bool {
        self.mode == CurvePointMode::Sharp
    }

    pub fn is_smooth(&self) -> bool {
        self.mode == CurvePointMode::Smooth
    }
}

/// Ordinals match the file format. The value for 2 for smooth and 3 for sharp
/// appear to legacy from older versions of Phase Plant.
#[derive(Clone, Copy, Debug, FromRepr, PartialEq)]
#[repr(u32)]
pub enum CurvePointMode {
    Smooth = 0,

    /// The Phase Plant documentation calls this mode "Hard".
    #[doc(alias = "hard")]
    Sharp = 1,
}

impl CurvePointMode {
    pub(crate) fn from_id(id: u32) -> Result<Self, Error> {
        // Convert from legacy values.
        if id == 2 {
            Ok(CurvePointMode::Smooth)
        } else if id == 3 {
            Ok(CurvePointMode::Sharp)
        } else {
            Self::from_repr(id).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidData,
                    format!("Unknown curve point mode {id}"),
                )
            })
        }
    }
}

impl Display for CurvePointMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            CurvePointMode::Sharp => "Sharp",
            CurvePointMode::Smooth => "Smooth",
        };
        f.write_str(msg)
    }
}

/// Returns a string representation of the coordinates of the point. The mode
/// is not included.
///
/// ```
/// use synthahol_phase_plant::CurvePoint;
///
/// let point = CurvePoint::new_sharp(1.1, 2.2, 3.0, 4.0);
/// assert_eq!(point.to_string(), "(1.1, 2.2) - (3, 4)");
/// ```
impl Display for CurvePoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}) - ({}, {})",
            self.x, self.y, self.curve_x, self.curve_y
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors() {
        assert!(CurvePoint::new_sharp(1.0, 2.0, 3.0, 4.0).is_sharp());
        assert!(!CurvePoint::new_smooth(1.0, 2.0, 3.0, 4.0).is_sharp());
        assert!(CurvePoint::new_smooth(1.0, 2.0, 3.0, 4.0).is_smooth());
        assert!(!CurvePoint::new_sharp(1.0, 2.0, 3.0, 4.0).is_smooth());
    }
}
