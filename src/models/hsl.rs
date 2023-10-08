//! Model a color with the HSL notation in the sRGB color space.

use crate::color::{Component, HasSpace, Space};

camelion_macros::gen_model! {
    /// A color specified with the HSL notation in the sRGB color space.
    pub struct Hsl {
        /// The hue component of the color.
        pub hue: Component,
        /// The saturation component of the color.
        saturation: Component,
        /// The lightness component of the color.
        lightness: Component,
    }
}

impl HasSpace for Hsl {
    const SPACE: Space = Space::Hsl;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::Model, Flags};

    #[test]
    fn nan_components_are_missing() {
        let c = Hsl::new(Component::NAN, Component::NAN, Component::NAN).to_color(None);
        assert_eq!(
            c.flags,
            Flags::C0_IS_NONE | Flags::C1_IS_NONE | Flags::C2_IS_NONE | Flags::ALPHA_IS_NONE
        );
    }
}
