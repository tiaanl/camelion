//! Functions for converting color models to a base color space common to all
//! models.  Used for color conversion.

use super::WithoutGammaEncoding;
use crate::models::ToXyz;
use crate::{color_space::ColorSpace, models};

// We use XYZ with a D65 white point as the base color space for converting
// between color spaces.
// D65 is used by many more color spaces than D50, so it's a better choice for
// not having to do unnecessary matrix multiplications.
pub type BaseWhitePoint = models::D65;
pub type Base = models::Xyz<BaseWhitePoint>;

/// Used to convert any model to a base color space.
pub trait ToBase {
    /// Convert the model to a base color space.
    fn to_base(&self) -> Base;
}

impl<S: ColorSpace, E: super::rgb::encoding::GammaEncoding> ToBase for models::Rgb<S, E>
where
    models::Rgb<S, E>: WithoutGammaEncoding<S>,
    models::Rgb<S, models::rgb::encoding::LinearLight>: models::ToXyz,
    <models::rgb::Rgb<S, models::rgb::encoding::LinearLight> as models::ToXyz>::WhitePoint:
        models::xyz::TransferWhitePoint<BaseWhitePoint>,
{
    fn to_base(&self) -> Base {
        self.without_gamma_encoding().to_xyz().transfer()
    }
}

impl ToBase for models::Hsl {
    fn to_base(&self) -> Base {
        self.to_srgb().to_base()
    }
}

impl ToBase for models::Hwb {
    fn to_base(&self) -> Base {
        self.to_srgb().to_base()
    }
}

impl<S: ColorSpace> ToBase for models::Rectangular<S>
where
    models::Rectangular<S>: ToXyz,
    <models::Rectangular<S> as ToXyz>::WhitePoint: models::xyz::TransferWhitePoint<BaseWhitePoint>,
{
    fn to_base(&self) -> Base {
        self.to_xyz().transfer()
    }
}

impl<S: ColorSpace> ToBase for models::Polar<S>
where
    models::Rectangular<S>: ToBase,
    models::Rectangular<S>: ToXyz,
{
    fn to_base(&self) -> Base {
        self.to_rectangular().to_base()
    }
}

impl<W: models::WhitePoint> ToBase for models::Xyz<W>
where
    W: models::xyz::TransferWhitePoint<BaseWhitePoint>,
{
    fn to_base(&self) -> Base {
        self.transfer()
    }
}

#[cfg(test)]
mod tests {
    use crate::color_space;

    use super::*;

    #[test]
    fn test_rgb_to_base() {
        models::Rgb::<color_space::Srgb, models::encoding::GammaEncoded>::new(0.0, 0.0, 0.0)
            .to_base();
        models::Rgb::<color_space::Srgb, models::encoding::LinearLight>::new(0.0, 0.0, 0.0)
            .to_base();
        models::Rgb::<color_space::ProPhotoRgb, models::encoding::GammaEncoded>::new(0.0, 0.0, 0.0)
            .to_base();
        models::Rgb::<color_space::ProPhotoRgb, models::encoding::LinearLight>::new(0.0, 0.0, 0.0)
            .to_base();
    }

    #[test]
    fn test_hsl_hwb() {
        models::Hsl::new(0.0, 0.0, 0.0).to_base();
        models::Hwb::new(0.0, 0.0, 0.0).to_base();
    }

    #[test]
    fn test_lab() {
        models::Lab::new(0.0, 0.0, 0.0).to_base();
        models::Lch::new(0.0, 0.0, 0.0).to_base();
        models::Oklab::new(0.0, 0.0, 0.0).to_base();
        models::Oklch::new(0.0, 0.0, 0.0).to_base();
    }

    #[test]
    fn test_xyz() {
        models::XyzD50::new(0.0, 0.0, 0.0).to_base();
        models::XyzD65::new(0.0, 0.0, 0.0).to_base();
    }
}
