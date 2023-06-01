use uom::num::Zero;
use uom::si::f32::{Ratio, Time};
use uom::si::time::second;

/// ADSR-style envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct Envelope {
    pub delay: Time,
    pub attack: Time,

    #[doc(alias = "attack_slope")]
    pub attack_curve: f32,

    pub hold: Time,
    pub decay: Time,

    #[doc(alias = "decay_slope")]
    pub decay_falloff: f32,

    /// A percentage, not milliseconds
    pub sustain: Ratio,

    pub release: Time,

    #[doc(alias = "release_slope")]
    pub release_falloff: f32,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            delay: Time::zero(),
            attack: Time::new::<second>(-1.01),
            attack_curve: -1.0,
            hold: Time::zero(),
            decay: Time::new::<second>(-1.1),
            decay_falloff: -1.0,
            sustain: Ratio::zero(),
            release: Time::new::<second>(-1.1),
            release_falloff: -1.0,
        }
    }
}
