use std::any::Any;

use crate::modulator::{Modulator, ModulatorMode};
use crate::*;

#[derive(Debug, PartialEq)]
pub struct BlankModulator {}

impl Modulator for BlankModulator {
    fn as_block(&self) -> ModulatorBlock {
        self.into()
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }

    fn mode(&self) -> ModulatorMode {
        ModulatorMode::Blank
    }
}
