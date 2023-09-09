//! Model a color in the sRGB color space.

use std::marker::PhantomData;

use crate::color::ComponentDetails;
use crate::{Component, Flags, Space};

mod tag {
    use crate::Space;

    /// This trait is used to identify tags that specify a color space/notation.
    pub trait SpaceTag {
        const SPACE: Space;
    }

    /// Tag for the sRGB color space.
    pub struct Srgb;
    impl SpaceTag for Srgb {
        const SPACE: Space = Space::Srgb;
    }
}

/// A color specified in the sRGB color space.
pub struct Rgb<S: tag::SpaceTag> {
    /// The red component of the color.
    pub red: Component,
    /// The green component of the color.
    pub green: Component,
    /// The blue component of the color.
    pub blue: Component,
    /// The alpha component of the color.
    pub alpha: Component,
    /// Holds any flags that might be enabled for this color.
    pub flags: Flags,
    _space: Space,
    _space_tag: PhantomData<S>,
}

impl<S: tag::SpaceTag> Rgb<S> {
    /// Create a new color with RGB (red, green, blue) components.
    pub fn new(
        red: impl Into<ComponentDetails>,
        green: impl Into<ComponentDetails>,
        blue: impl Into<ComponentDetails>,
        alpha: impl Into<ComponentDetails>,
    ) -> Self {
        let mut flags = Flags::empty();

        let red = red.into().value_and_flag(&mut flags, Flags::C0_IS_NONE);
        let green = green.into().value_and_flag(&mut flags, Flags::C1_IS_NONE);
        let blue = blue.into().value_and_flag(&mut flags, Flags::C2_IS_NONE);
        let alpha = alpha
            .into()
            .value_and_flag(&mut flags, Flags::ALPHA_IS_NONE);

        Self {
            red,
            green,
            blue,
            alpha,
            flags,
            _space: S::SPACE,
            _space_tag: PhantomData,
        }
    }
}

/// Model for a color in the sRGB color space.
pub type Srgb = Rgb<tag::Srgb>;
