//! Phase Plant version numbers

use crate::PhasePlantRelease::V1_6_9;
use std::fmt::{Display, Formatter};

/// Not all versions are listed. Only versions that indicate the the start or
/// end of a new init preset are included.
pub enum PhasePlantRelease {
    /// Some factory presets were created with his versions before the public
    /// release.
    V1_6_9,
    V1_6_10,

    /// The first public release.
    V1_7_0,

    V1_7_1,
    V1_7_3,
    V1_7_4,
    V1_7_5,
    V1_7_11,
    V1_8_0,
    V1_8_4,
    V1_8_5,
    V1_8_9,
    V1_8_11,
    V1_8_28,
    V2_0_0,
    V2_0_11,
    V2_0_12,
    V2_0_13,
    V2_0_16,
    V2_1_0,
}

impl PhasePlantRelease {
    pub fn version(&self) -> Version<u8> {
        use PhasePlantRelease::*;
        match self {
            V1_6_9 => Version::new(1, 6, 9, 0),
            V1_6_10 => Version::new(1, 6, 9, 0),
            V1_7_0 => Version::new(1, 7, 0, 0),
            V1_7_1 => Version::new(1, 7, 1, 0),
            V1_7_3 => Version::new(1, 7, 3, 0),
            V1_7_4 => Version::new(1, 7, 4, 0),
            V1_7_5 => Version::new(1, 7, 5, 0),
            V1_7_11 => Version::new(1, 7, 11, 0),
            V1_8_0 => Version::new(1, 8, 0, 0),
            V1_8_4 => Version::new(1, 8, 4, 0),
            V1_8_5 => Version::new(1, 8, 5, 0),
            V1_8_9 => Version::new(1, 8, 9, 0),
            V1_8_11 => Version::new(1, 8, 11, 0),
            V1_8_28 => Version::new(1, 8, 28, 0),
            V2_0_0 => Version::new(2, 0, 0, 0),
            V2_0_11 => Version::new(2, 0, 11, 0),
            V2_0_12 => Version::new(2, 0, 12, 0),
            V2_0_13 => Version::new(2, 0, 13, 0),
            V2_0_16 => Version::new(2, 0, 16, 0),
            V2_1_0 => Version::new(2, 1, 0, 0),
        }
    }

    /// Each release of Phase Plant creates presets in a specific and different
    /// file format version.
    pub const fn format_version(&self) -> Version<u32> {
        use PhasePlantRelease::*;
        match self {
            V1_6_9 => Version::new(5, 2, 1010, 0),
            V1_6_10 => Version::new(5, 2, 1011, 0),
            V1_7_0 => Version::new(5, 2, 1012, 0),
            V1_7_1 => Version::new(5, 2, 1012, 0),
            V1_7_3 => Version::new(5, 2, 1013, 0),
            V1_7_4 => Version::new(5, 2, 1015, 0),
            V1_7_5 => Version::new(5, 2, 1016, 0),
            V1_7_11 => Version::new(5, 2, 1016, 0),
            V1_8_0 => Version::new(6, 2, 1019, 0),
            V1_8_4 => Version::new(6, 2, 1019, 0),
            V1_8_5 => Version::new(6, 2, 1024, 0),
            V1_8_9 => Version::new(6, 2, 1024, 0),
            V1_8_11 => Version::new(6, 2, 1024, 0),
            V1_8_28 => Version::new(6, 2, 1024, 0),
            V2_0_0 => Version::new(6, 2, 1036, 0),
            V2_0_11 => Version::new(6, 2, 1036, 0),
            V2_0_12 => Version::new(6, 2, 1037, 0),
            V2_0_13 => Version::new(6, 2, 1038, 0),
            V2_0_16 => Version::new(6, 2, 1038, 0),
            V2_1_0 => Version::new(6, 2, 1040, 0),
        }
    }

    /// Determine if a preset format version is probably one used by Phase Plant.
    pub fn is_likely_format_version(format_version: &Version<u32>) -> bool {
        format_version.minor == 2 // All start with two
            && format_version.patch >= V1_6_9.format_version().patch // Patch level is always increasing
            && format_version.is_at_least(&V1_6_9.format_version())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Version<T: PartialOrd> {
    pub major: T,
    pub minor: T,
    pub patch: T,
    pub extra: T,
}

impl<T: PartialOrd> Version<T> {
    pub const fn new(major: T, minor: T, patch: T, extra: T) -> Version<T> {
        Self {
            major,
            minor,
            patch,
            extra,
        }
    }

    pub fn is_at_least(&self, other: &Version<T>) -> bool {
        self.major > other.major
            || self.major == other.major && self.minor > other.minor
            || self.major == other.major && self.minor == other.minor && self.patch > other.patch
            || self.major == other.major
                && self.minor == other.minor
                && self.patch == other.patch
                && self.extra >= other.extra
    }
}

impl<T: Default + PartialOrd> Version<T> {
    pub fn is_zero(&self) -> bool {
        self.major == Default::default()
            && self.minor == Default::default()
            && self.patch == Default::default()
            && self.extra == Default::default()
    }
}

impl<T: Display + Default + PartialEq + PartialOrd> Display for Version<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if self.extra != T::default() {
            write!(f, "-{}", self.extra)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::version::Version;
    use crate::PhasePlantRelease;

    #[test]
    fn at_least() {
        assert!(Version::new(6, 2, 1038, 0).is_at_least(&Version::new(5, 2, 1012, 0)));
    }

    #[test]
    fn extra() {
        let version = Version::new(1, 8, 17, 0x2a);
        assert_eq!("1.8.17-42", format!("{}", version));
    }

    #[test]
    fn phase_plant_is_likely() {
        assert!(PhasePlantRelease::is_likely_format_version(&Version::new(
            5, 2, 1020, 0,
        )));
        assert!(PhasePlantRelease::is_likely_format_version(&Version::new(
            6, 2, 1020, 0,
        )));
        assert!(PhasePlantRelease::is_likely_format_version(&Version::new(
            6, 2, 1050, 0,
        )));

        assert!(!PhasePlantRelease::is_likely_format_version(&Version::new(
            4, 2, 1020, 0,
        )));
        assert!(!PhasePlantRelease::is_likely_format_version(&Version::new(
            6, 0, 1020, 0,
        )));
        assert!(!PhasePlantRelease::is_likely_format_version(&Version::new(
            6, 2, 42, 0,
        )));
    }

    #[test]
    fn regular() {
        let version = Version::new(1, 8, 17, 0);
        assert_eq!("1.8.17", format!("{}", version));
    }

    #[test]
    fn zero() {
        assert!(Version::new(0, 0, 0, 0).is_zero());
        assert!(!Version::new(1, 8, 17, 0).is_zero());
        assert!(!Version::new(0, 0, 0, 42).is_zero());
    }
}
