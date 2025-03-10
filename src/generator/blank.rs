//! Generator representing the lack of a generator.  It takes up space in the
//! preset file.

use std::any::Any;

use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BlankGenerator {}

impl From<&GeneratorBlock> for BlankGenerator {
    fn from(_block: &GeneratorBlock) -> Self {
        BlankGenerator {}
    }
}

impl Generator for BlankGenerator {
    fn id(&self) -> Option<GeneratorId> {
        None
    }

    fn as_block(&self) -> GeneratorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>() == Some(self)
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn mode(&self) -> GeneratorMode {
        GeneratorMode::Blank
    }

    fn name(&self) -> String {
        "Blank".to_owned()
    }
}

impl dyn Generator {
    #[must_use]
    pub fn as_blank(&self) -> Option<&BlankGenerator> {
        self.downcast_ref::<BlankGenerator>()
    }
}

#[cfg(test)]
mod test {
    use crate::generator::Generator;

    use super::BlankGenerator;

    #[test]
    fn default() {
        let generator = BlankGenerator::default();
        assert_eq!(generator.name(), "Blank".to_owned());
    }
}
