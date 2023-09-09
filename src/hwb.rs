//! Model a color with the HWB notation in the sRGB color space.

use crate::color::ComponentDetails;
use crate::{Component, Flags, Space};

/// A color specified with the HWB notation in the sRGB color space.
pub struct Hwb {
    /// The hue component of the color.
    pub hue: Component,
    /// The whiteness component of the color.
    pub whiteness: Component,
    /// The blackness component of the color.
    pub blackness: Component,
    /// The alpha component of the color.
    pub alpha: Component,
    /// Holds any flags that might be enabled for this color.
    pub flags: Flags,
    _space: Space,
}

impl Hwb {
    /// Create a new color with RGB (red, green, blue) components.
    pub fn new(
        hue: impl Into<ComponentDetails>,
        whiteness: impl Into<ComponentDetails>,
        blackness: impl Into<ComponentDetails>,
        alpha: impl Into<ComponentDetails>,
    ) -> Self {
        let mut flags = Flags::empty();

        let hue = hue.into().value_and_flag(&mut flags, Flags::C0_IS_NONE);
        let whiteness = whiteness
            .into()
            .value_and_flag(&mut flags, Flags::C1_IS_NONE);
        let blackness = blackness
            .into()
            .value_and_flag(&mut flags, Flags::C2_IS_NONE);
        let alpha = alpha
            .into()
            .value_and_flag(&mut flags, Flags::ALPHA_IS_NONE);

        Self {
            hue,
            whiteness,
            blackness,
            alpha,
            flags,
            _space: Space::Hwb,
        }
    }
}
