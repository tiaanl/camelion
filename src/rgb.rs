//! Model a color in the sRGB color space.

use crate::color::{ComponentDetails, HasSpace, SpacePlaceholder};
use crate::xyz::{ConvertToXyz, Xyz};
use crate::{Color, Component, Components, Flags, Space, Transform, Vector, D65};
use std::marker::PhantomData;

mod encoding {
    use crate::Components;

    /// This trait is used to identity tags that specify gamma encoding.
    pub trait Encoding {}

    pub struct GammaEncoded;
    impl Encoding for GammaEncoded {}

    pub struct LinearLight;
    impl Encoding for LinearLight {}

    pub trait GammaConversion {
        fn to_gamma_encoded(from: &Components) -> Components;
        fn to_linear_light(from: &Components) -> Components;
    }
}

mod space {
    use crate::Components;

    use super::encoding::GammaConversion;

    /// This trait is used to identify tags that specify a color space/notation.
    pub trait Space {}

    /// Tag for the sRGB color space.
    pub struct Srgb;

    impl Space for Srgb {}

    impl GammaConversion for Srgb {
        fn to_gamma_encoded(from: &Components) -> Components {
            from.map(|value| {
                let abs = value.abs();

                if abs > 0.0031308 {
                    value.signum() * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
                } else {
                    12.92 * value
                }
            })
        }

        fn to_linear_light(from: &Components) -> Components {
            from.map(|value| {
                let abs = value.abs();

                if abs < 0.04045 {
                    value / 12.92
                } else {
                    value.signum() * ((abs + 0.055) / 1.055).powf(2.4)
                }
            })
        }
    }

    /// Tag for the DisplayP3 color space.
    pub struct DisplayP3;

    impl Space for DisplayP3 {}
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

impl<S: space::Space + encoding::GammaConversion> Rgb<S, encoding::GammaEncoded> {
    pub fn to_linear_light(&self) -> Rgb<S, encoding::LinearLight> {
        let Components(red, green, blue) =
            S::to_linear_light(&Components(self.red, self.green, self.blue));
        Rgb::new(red, green, blue, self.alpha)
    }
}

impl<S: space::Space + encoding::GammaConversion> Rgb<S, encoding::LinearLight> {
    pub fn to_gamma_encoded(&self) -> Rgb<S, encoding::GammaEncoded> {
        let Components(red, green, blue) =
            S::to_gamma_encoded(&Components(self.red, self.green, self.blue));
        Rgb::new(red, green, blue, self.alpha)
    }
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

/// Model for a color in the sRGB color space with gamma encoding.
pub type Srgb = Rgb<space::Srgb, encoding::GammaEncoded>;

impl HasSpace for Srgb {
    const SPACE: Space = Space::Srgb;
}

impl ConvertToXyz<D65> for Rgb<space::Srgb, encoding::LinearLight> {
    fn to_xyz(&self) -> Xyz<D65> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const TO_XYZ: Transform = Transform::new(
            0.4123907992659595,  0.21263900587151036, 0.01933081871559185, 0.0,
            0.35758433938387796, 0.7151686787677559,  0.11919477979462599, 0.0,
            0.1804807884018343,  0.07219231536073371, 0.9505321522496606,  0.0,
            0.0,                 0.0,                 0.0,                 1.0,
        );

        let Vector { x, y, z, .. } =
            TO_XYZ.transform_vector3d(Vector::new(self.red, self.green, self.blue));

        Xyz::new(x, y, z, self.alpha)
    }
}

/// Model for a color in the sRGB color space with no gamma encoding.
pub type SrgbLinear = Rgb<space::Srgb, encoding::LinearLight>;

impl HasSpace for SrgbLinear {
    const SPACE: Space = Space::SrgbLinear;
}

/// Model for a color in the DisplayP3 color space with gamme encoding.
pub type DisplayP3 = Rgb<space::DisplayP3, encoding::GammaEncoded>;

impl HasSpace for DisplayP3 {
    const SPACE: Space = Space::DisplayP3;
}
