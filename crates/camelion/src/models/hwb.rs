//! Model a color with the HWB notation in the sRGB color space.

use crate::{
    color::{CssColorSpaceId, Space},
    Component,
};

camelion_macros::gen_model! {
    /// A color specified with the HWB notation in the sRGB color space.
    pub struct Hwb {
        /// The hue component of the color.
        pub hue: Component,
        /// The whiteness component of the color.
        pub whiteness: Component,
        /// The blackness component of the color.
        pub blackness: Component,
    }
}

impl CssColorSpaceId for Hwb {
    const ID: Space = Space::Hwb;
}
