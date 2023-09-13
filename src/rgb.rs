//! Model a color in the sRGB color space.

use crate::color::ComponentDetails;
use crate::{Component, Flags, Space};
use std::marker::PhantomData;

use self::model::RgbModel;

mod space {
    /// This trait is used to identify tags that specify a color space/notation.
    pub trait SpaceTag {}

    /// Tag for the sRGB color space.
    pub struct Srgb;
    impl SpaceTag for Srgb {}

    /// Tag for the DisplayP3 color space.
    pub struct DisplayP3;
    impl SpaceTag for DisplayP3 {}
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
    use std::marker::PhantomData;

    use super::encoding;
    use super::space;
    use crate::Space;

    pub trait RgbModel {
        const SPACE: Space;
    }

    pub struct Model<S: space::SpaceTag, E: encoding::GammaEncodingTag> {
        s: PhantomData<S>,
        e: PhantomData<E>,
    }

    impl RgbModel for Model<space::Srgb, encoding::GammaEncoded> {
        const SPACE: Space = Space::Srgb;
    }

    impl RgbModel for Model<space::Srgb, encoding::LinearLight> {
        const SPACE: Space = Space::SrgbLinear;
    }

    impl RgbModel for Model<space::DisplayP3, encoding::GammaEncoded> {
        const SPACE: Space = Space::DisplayP3;
    }
}

/// A color specified in the sRGB color space.
pub struct Rgb<S: space::SpaceTag, E: encoding::GammaEncodingTag> {
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
    s: PhantomData<S>,
    e: PhantomData<E>,
}

impl<S: space::SpaceTag, E: encoding::GammaEncodingTag> Rgb<S, E>
where
    model::Model<S, E>: RgbModel,
{
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
            _space: <model::Model<S, E> as model::RgbModel>::SPACE,
            s: PhantomData,
            e: PhantomData,
        }
    }
}

/// Model for a color in the sRGB color space with gamma encoding.
pub type Srgb = Rgb<space::Srgb, encoding::GammaEncoded>;

/// Model for a color in the sRGB color space with no gamma encoding.
pub type SrgbLinear = Rgb<space::Srgb, encoding::LinearLight>;

/// Model for a color in the DisplayP3 color space with gamme encoding.
pub type DisplayP3 = Rgb<space::DisplayP3, encoding::GammaEncoded>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_rgb_colors() {
        let srgb = Srgb::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(srgb.red, 0.1);
        assert_eq!(srgb.green, 0.2);
        assert_eq!(srgb.blue, 0.3);
        assert_eq!(srgb.alpha, 0.4);
        assert!(srgb.flags.is_empty());
        assert_eq!(srgb._space, Space::Srgb);

        let srgb_linear = SrgbLinear::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(srgb_linear.red, 0.1);
        assert_eq!(srgb_linear.green, 0.2);
        assert_eq!(srgb_linear.blue, 0.3);
        assert_eq!(srgb_linear.alpha, 0.4);
        assert!(srgb_linear.flags.is_empty());
        assert_eq!(srgb_linear._space, Space::SrgbLinear);

        let display_p3 = DisplayP3::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!(display_p3.red, 0.1);
        assert_eq!(display_p3.green, 0.2);
        assert_eq!(display_p3.blue, 0.3);
        assert_eq!(display_p3.alpha, 0.4);
        assert!(display_p3.flags.is_empty());
        assert_eq!(display_p3._space, Space::DisplayP3);
    }
}
