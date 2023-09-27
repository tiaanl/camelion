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
