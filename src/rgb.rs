//! Model a color in the sRGB color space.

use crate::color::{ComponentDetails, HasSpace, SpacePlaceholder};
use crate::{Color, Component, Components, Flags, Space};
use std::marker::PhantomData;

mod space {
    /// This trait is used to identify tags that specify a color space/notation.
    pub trait Space {}

    /// Tag for the sRGB color space.
    pub struct Srgb;
    impl Space for Srgb {}

    /// Tag for the DisplayP3 color space.
    pub struct DisplayP3;
    impl Space for DisplayP3 {}
}

mod encoding {
    /// This trait is used to identity tags that specify gamma encoding.
    pub trait Encoding {}

    pub struct GammaEncoded;
    impl Encoding for GammaEncoded {}

    pub struct LinearLight;
    impl Encoding for LinearLight {}
}

pub trait HasGammaEncoding<S: space::Space> {
    fn to_gamma_encoded(&self) -> Rgb<S, encoding::GammaEncoded>;
}

pub trait HasLinearLight<S: space::Space> {
    fn to_linear_light(&self) -> Rgb<S, encoding::LinearLight>;
}

/// A color specified in the sRGB color space.
pub struct Rgb<S: space::Space, E: encoding::Encoding> {
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

    _space: SpacePlaceholder,
    _s: PhantomData<S>,
    _e: PhantomData<E>,
}

impl<S: space::Space, E: encoding::Encoding> Rgb<S, E> {
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
            _space: 0,
            _s: PhantomData,
            _e: PhantomData,
        }
    }
}

impl<S: space::Space> Rgb<S, encoding::GammaEncoded>
where
    Rgb<S, encoding::GammaEncoded>: HasLinearLight<S>,
{
    pub fn to_linear_light(&self) -> Rgb<S, encoding::LinearLight> {
        <Self as HasLinearLight<S>>::to_linear_light(&self)
    }
}

impl<S: space::Space> Rgb<S, encoding::LinearLight>
where
    Rgb<S, encoding::LinearLight>: HasGammaEncoding<S>,
{
    pub fn to_gamma_encoded(&self) -> Rgb<S, encoding::GammaEncoded> {
        <Self as HasGammaEncoding<S>>::to_gamma_encoded(&self)
    }
}

/// Model for a color in the sRGB color space with gamma encoding.
pub type Srgb = Rgb<space::Srgb, encoding::GammaEncoded>;

impl HasSpace for Srgb {
    const SPACE: Space = Space::Srgb;
}

impl HasLinearLight<space::Srgb> for Srgb {
    /// Convert a gamma encoded sRGB color to a sRGB color without gamma
    /// encoding (linear light).
    fn to_linear_light(&self) -> SrgbLinear {
        let Components(red, green, blue) =
            Components(self.red, self.green, self.blue).map(|value| {
                let abs = value.abs();

                if abs < 0.04045 {
                    value / 12.92
                } else {
                    value.signum() * ((abs + 0.055) / 1.055).powf(2.4)
                }
            });
        SrgbLinear::new(red, green, blue, self.alpha)
    }
}

/// Model for a color in the sRGB color space with no gamma encoding.
pub type SrgbLinear = Rgb<space::Srgb, encoding::LinearLight>;

impl HasGammaEncoding<space::Srgb> for SrgbLinear {
    /// Convert a sRGB color without gamma encoding (linear light) to a sRGB
    /// color with gamma encoding.
    fn to_gamma_encoded(&self) -> Srgb {
        let Components(red, green, blue) =
            Components(self.red, self.green, self.blue).map(|value| {
                let abs = value.abs();

                if abs > 0.0031308 {
                    value.signum() * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
                } else {
                    12.92 * value
                }
            });
        Srgb::new(red, green, blue, self.alpha)
    }
}

impl HasSpace for SrgbLinear {
    const SPACE: Space = Space::SrgbLinear;
}

/// Model for a color in the DisplayP3 color space with gamme encoding.
pub type DisplayP3 = Rgb<space::DisplayP3, encoding::GammaEncoded>;

impl HasSpace for DisplayP3 {
    const SPACE: Space = Space::DisplayP3;
}

impl<S: space::Space, E: encoding::Encoding> From<Rgb<S, E>> for Color
where
    Rgb<S, E>: HasSpace,
{
    fn from(value: Rgb<S, E>) -> Self {
        Self {
            components: Components(value.red, value.green, value.blue),
            alpha: value.alpha,
            flags: value.flags,
            space: <Rgb<S, E> as HasSpace>::SPACE,
        }
    }
}
