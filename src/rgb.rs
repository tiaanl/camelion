//! Model a color in the sRGB color space.

use std::marker::PhantomData;

use crate::color::ComponentDetails;
use crate::{Component, Flags, Space};

mod space {
    /// This trait is used to identify tags that specify a color space/notation.
    pub trait SpaceTag {}

    /// Tag for the sRGB color space.
    pub struct Srgb;
    impl SpaceTag for Srgb {}
}

mod encoding {
    /// This trait is used to identity tags that specify gamma encoding.
    pub trait GammaEncodingTag {}

    pub struct GammaEncoded;
    impl GammaEncodingTag for GammaEncoded {}

    pub struct LinearLight;
    impl GammaEncodingTag for LinearLight {}
}

mod model {
    use super::encoding;
    use super::space;
    use crate::Space;

    pub trait RgbModel {
        type Space: space::SpaceTag;
        type GammaEncoding: encoding::GammaEncodingTag;

        const SPACE: Space;
    }

    pub struct Srgb;
    impl RgbModel for Srgb {
        type Space = space::Srgb;
        type GammaEncoding = encoding::GammaEncoded;

        const SPACE: Space = Space::Srgb;
    }

    pub struct SrgbLinear;
    impl RgbModel for SrgbLinear {
        type Space = space::Srgb;
        type GammaEncoding = encoding::GammaEncoded;

        const SPACE: Space = Space::SrgbLinear;
    }
}

/// A color specified in the sRGB color space.
pub struct Rgb<R: model::RgbModel> {
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
    r: PhantomData<R>,
}

impl<M: model::RgbModel> Rgb<M> {
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
            _space: M::SPACE,
            r: PhantomData,
        }
    }
}

/// Model for a color in the sRGB color space with gamma encoding.
pub type Srgb = Rgb<model::Srgb>;

/// Model for a color in the sRGB color space with no gamma encoding.
pub type SrgbLinear = Rgb<model::SrgbLinear>;
