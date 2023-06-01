//! This module is not called "macro" to because it's a keyword.

use crate::modulator::OutputRange;
use std::fmt;

pub type MacroControlId = u8;

#[derive(Clone, PartialEq)]
pub struct MacroControl {
    pub name: String,
    pub value: f32,
    pub polarity: OutputRange,
}

impl fmt::Debug for MacroControl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "Macro {{ \"{}\" = {} {}}}",
            &self.name,
            &self.value,
            self.polarity.symbol(),
        ))
    }
}

impl MacroControl {
    /// Number of macros controls (knobs) in the file.
    pub const COUNT: usize = 8;

    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            value: 0.0,
            polarity: OutputRange::Unipolar,
        }
    }

    /// Create a new list of default macros.
    pub fn defaults() -> [Self; Self::COUNT] {
        [
            Self::new("Macro 1"),
            Self::new("Macro 2"),
            Self::new("Macro 3"),
            Self::new("Macro 4"),
            Self::new("Macro 5"),
            Self::new("Macro 6"),
            Self::new("Macro 7"),
            Self::new("Macro 8"),
        ]
    }
}

#[cfg(test)]
mod test {
    use crate::modulator::OutputRange;
    use crate::test::read_preset;

    #[test]
    fn name() {
        let preset = read_preset("macros", "macro-10to80%-1.8.13.phaseplant");
        assert_eq!(preset.macro_controls[0].name, "Macro 1");
        assert_eq!(preset.macro_controls[1].name, "Macro 2");
        assert_eq!(preset.macro_controls[2].name, "Macro 3");
        assert_eq!(preset.macro_controls[3].name, "Macro 4");
        assert_eq!(preset.macro_controls[4].name, "Macro 5");
        assert_eq!(preset.macro_controls[5].name, "Macro 6");
        assert_eq!(preset.macro_controls[6].name, "Macro 7");
        assert_eq!(preset.macro_controls[7].name, "Macro 8");
    }
    #[test]
    fn polarity() {
        let preset = read_preset(
            "macros",
            "macro1_unipolar-macro2_bipolar-macro3_inverted-2.1.0.phaseplant",
        );
        assert_eq!(preset.macro_controls[0].polarity, OutputRange::Unipolar);
        assert_eq!(preset.macro_controls[1].polarity, OutputRange::Bipolar);
        assert_eq!(preset.macro_controls[2].polarity, OutputRange::Inverted);
        assert_eq!(preset.macro_controls[3].polarity, OutputRange::Unipolar);
    }

    #[test]
    fn values() {
        let preset = read_preset("macros", "macro-10to80%-1.8.13.phaseplant");
        assert_eq!(preset.macro_controls[0].value, 0.1);
        assert_eq!(preset.macro_controls[1].value, 0.2);
        assert_eq!(preset.macro_controls[2].value, 0.3);
        assert_eq!(preset.macro_controls[3].value, 0.4);
        assert_eq!(preset.macro_controls[4].value, 0.5);
        assert_eq!(preset.macro_controls[5].value, 0.6);
        assert_eq!(preset.macro_controls[6].value, 0.7);
        assert_eq!(preset.macro_controls[7].value, 0.8);
    }
}
