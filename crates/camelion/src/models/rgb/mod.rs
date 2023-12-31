//! Model a color in the sRGB color space.

mod gamma;
mod gamut;

pub use gamma::HasGammaEncoding;

use crate::{
    color::{Components, CssColorSpaceId, Space},
    color_space::{self, ColorSpace},
    math::{transform, transform_3x3, Transform},
    models::xyz::{ToXyz, Xyz, XyzD50, XyzD65, D50, D65},
    Component,
};

/// Tags for RGB models that are either gamma encoded or linear light.
pub mod encoding {
    /// This trait is used to identity tags that specify gamma encoding.
    pub trait GammaEncoding {}

    /// Tag for a gamma encoded RGB color.
    #[derive(Clone, Debug)]
    pub struct GammaEncoded;

    impl GammaEncoding for GammaEncoded {}

    /// Tag for a linear light RGB color.
    #[derive(Clone, Debug)]
    pub struct LinearLight;

    impl GammaEncoding for LinearLight {}
}

camelion_macros::gen_model! {
    /// A color specified in the sRGB color space.
    pub struct Rgb<S: ColorSpace, E: encoding::GammaEncoding> {
        /// The red component of the color.
        pub red: Component,
        /// The green component of the color.
        pub green: Component,
        /// The blue component of the color.
        pub blue: Component,
    }
}

impl<S: ColorSpace + HasGammaEncoding> Rgb<S, encoding::GammaEncoded> {
    /// Convert this model from gamma encoded to linear light.
    pub fn to_linear_light(&self) -> Rgb<S, encoding::LinearLight> {
        let Components(red, green, blue) = S::to_linear_light(&self.to_components());
        Rgb::new(red, green, blue)
    }
}

impl<S: ColorSpace + HasGammaEncoding> Rgb<S, encoding::LinearLight> {
    /// Convert this model from linear light to gamma encoded.
    pub fn to_gamma_encoded(&self) -> Rgb<S, encoding::GammaEncoded> {
        let Components(red, green, blue) = S::to_gamma_encoded(&self.to_components());
        Rgb::new(red, green, blue)
    }
}

/// Model for a color in the sRGB color space with gamma encoding.
pub type Srgb = Rgb<color_space::Srgb, encoding::GammaEncoded>;

impl CssColorSpaceId for Srgb {
    const ID: Space = Space::Srgb;
}

impl From<Xyz<D65>> for Rgb<color_space::Srgb, encoding::LinearLight> {
    fn from(value: Xyz<D65>) -> Self {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const FROM_XYZ: Transform = transform_3x3(
             3.2409699419045213, -0.9692436362808798,  0.05563007969699361,
            -1.5373831775700935,  1.8759675015077206, -0.20397695888897657,
            -0.4986107602930033,  0.04155505740717561, 1.0569715142428786,
        );

        transform(&FROM_XYZ, Components(value.x, value.y, value.z)).into()
    }
}

impl ToXyz for Rgb<color_space::Srgb, encoding::LinearLight> {
    type WhitePoint = D65;

    fn to_xyz(&self) -> Xyz<Self::WhitePoint> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const TO_XYZ: Transform = transform_3x3(
            0.4123907992659595,  0.21263900587151036, 0.01933081871559185,
            0.35758433938387796, 0.7151686787677559,  0.11919477979462599,
            0.1804807884018343,  0.07219231536073371, 0.9505321522496606,
        );

        transform(&TO_XYZ, Components(self.red, self.green, self.blue)).into()
    }
}

/// Model for a color in the sRGB color space with no gamma encoding.
pub type SrgbLinear = Rgb<color_space::Srgb, encoding::LinearLight>;

impl CssColorSpaceId for SrgbLinear {
    const ID: Space = Space::SrgbLinear;
}

/// Model for a color in the DisplayP3 color space with gamma encoding.
pub type DisplayP3 = Rgb<color_space::DisplayP3, encoding::GammaEncoded>;

/// Model for a color in the DisplayP3 color space without gamma encoding.
pub type DisplayP3Linear = Rgb<color_space::DisplayP3, encoding::LinearLight>;

impl CssColorSpaceId for DisplayP3 {
    const ID: Space = Space::DisplayP3;
}

impl ToXyz for DisplayP3Linear {
    type WhitePoint = D65;

    fn to_xyz(&self) -> Xyz<Self::WhitePoint> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const TO_XYZ: Transform = transform_3x3(
            0.48657094864821626, 0.22897456406974884, 0.0,
            0.26566769316909294, 0.6917385218365062,  0.045113381858902575,
            0.1982172852343625,  0.079286914093745,   1.0439443689009757,
        );

        transform(&TO_XYZ, Components(self.red, self.green, self.blue)).into()
    }
}

impl From<Xyz<D65>> for Rgb<color_space::DisplayP3, encoding::LinearLight> {
    fn from(value: Xyz<D65>) -> Self {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const FROM_XYZ: Transform = transform_3x3(
             2.4934969119414245,  -0.829488969561575,    0.035845830243784335,
            -0.9313836179191236,   1.7626640603183468,  -0.07617238926804171,
            -0.40271078445071684,  0.02362468584194359,  0.9568845240076873,
        );

        transform(&FROM_XYZ, Components(value.x, value.y, value.z)).into()
    }
}

/// Model for a color in the a98 RGB color space with gamma encoding.
pub type A98Rgb = Rgb<color_space::A98Rgb, encoding::GammaEncoded>;

/// Model for a color in the a98 RGB color space without gamma encoding.
pub type A98RgbLinear = Rgb<color_space::A98Rgb, encoding::LinearLight>;

impl CssColorSpaceId for A98Rgb {
    const ID: Space = Space::A98Rgb;
}

impl ToXyz for A98RgbLinear {
    type WhitePoint = D65;

    fn to_xyz(&self) -> Xyz<Self::WhitePoint> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const TO_XYZ: Transform = transform_3x3(
            0.5766690429101308,  0.29734497525053616, 0.027031361386412378,
            0.18555823790654627, 0.627363566255466,   0.07068885253582714,
            0.18822864623499472, 0.07529145849399789, 0.9913375368376389,
        );

        transform(&TO_XYZ, Components(self.red, self.green, self.blue)).into()
    }
}

impl From<XyzD65> for A98RgbLinear {
    fn from(value: XyzD65) -> Self {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const FROM_XYZ: Transform = transform_3x3(
             2.041587903810746,  -0.9692436362808798,   0.013444280632031024,
            -0.5650069742788596,  1.8759675015077206,  -0.11836239223101824,
            -0.3447313507783295,  0.04155505740717561,  1.0151749943912054,
        );

        transform(&FROM_XYZ, Components(value.x, value.y, value.z)).into()
    }
}

/// Model for a color in the ProPhoto RGB color space with gamma encoding.
pub type ProPhotoRgb = Rgb<color_space::ProPhotoRgb, encoding::GammaEncoded>;

/// Model for a color in the ProPhoto RGB color space without gamma encoding.
pub type ProPhotoRgbLinear = Rgb<color_space::ProPhotoRgb, encoding::LinearLight>;

impl CssColorSpaceId for ProPhotoRgb {
    const ID: Space = Space::ProPhotoRgb;
}

impl ToXyz for ProPhotoRgbLinear {
    type WhitePoint = D50;

    fn to_xyz(&self) -> Xyz<Self::WhitePoint> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const TO_XYZ: Transform = transform_3x3(
            0.7977604896723027,  0.2880711282292934,     0.0,
            0.13518583717574031, 0.7118432178101014,     0.0,
            0.0313493495815248,  0.00008565396060525902, 0.8251046025104601,
        );

        transform(&TO_XYZ, Components(self.red, self.green, self.blue)).into()
    }
}

impl From<XyzD50> for ProPhotoRgbLinear {
    fn from(value: XyzD50) -> Self {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const FROM_XYZ: Transform = transform_3x3(
             1.3457989731028281,  -0.5446224939028347,  0.0,
            -0.25558010007997534,  1.5082327413132781,  0.0,
            -0.05110628506753401,  0.02053603239147973, 1.2119675456389454,
        );

        transform(&FROM_XYZ, Components(value.x, value.y, value.z)).into()
    }
}

/// Model for a color in the ProPhoto RGB color space with gamma encoding.
pub type Rec2020 = Rgb<color_space::Rec2020, encoding::GammaEncoded>;

/// Model for a color in the ProPhoto RGB color space without gamma encoding.
pub type Rec2020Linear = Rgb<color_space::Rec2020, encoding::LinearLight>;

impl CssColorSpaceId for Rec2020 {
    const ID: Space = Space::Rec2020;
}

impl ToXyz for Rec2020Linear {
    type WhitePoint = D65;

    fn to_xyz(&self) -> Xyz<Self::WhitePoint> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const TO_XYZ: Transform = transform_3x3(
            0.6369580483012913,  0.26270021201126703,  0.0,
            0.14461690358620838, 0.677998071518871,    0.028072693049087508,
            0.16888097516417205, 0.059301716469861945, 1.0609850577107909,
        );

        transform(&TO_XYZ, Components(self.red, self.green, self.blue)).into()
    }
}

impl From<XyzD65> for Rec2020Linear {
    fn from(value: XyzD65) -> Self {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const FROM_XYZ: Transform = transform_3x3(
             1.7166511879712676, -0.666684351832489,    0.017639857445310915,
            -0.3556707837763924,  1.616481236634939,   -0.042770613257808655,
            -0.2533662813736598,  0.01576854581391113,  0.942103121235474,
        );

        transform(&FROM_XYZ, Components(value.x, value.y, value.z)).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Model;
    use crate::Flags;

    #[test]
    fn nan_is_missing_component() {
        let c = Srgb::new(Component::NAN, 1.0, 1.0).to_color(Some(1.0));
        assert_eq!(c.components.0, 0.0);
        assert_eq!(c.components.1, 1.0);
        assert_eq!(c.components.2, 1.0);
        assert_eq!(c.flags, Flags::C0_IS_NONE);
    }
}
