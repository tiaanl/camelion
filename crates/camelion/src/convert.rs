//! Each color space/form is modeled with its own type. Conversions are only
//! implemented on relevant models, making conversion paths accurate and
//! performant.
//!
//! Conversions only operate on the 3 color components (no alpha, missing
//! components).
//!
//! NOTE: When a conversion yields a NaN value, the component is powerless and
//!       should be treated as missing.
//! NOTE: The reverse is not the same. Passing a value of NaN to a model will
//!       convert the value to 0.0.
//!
//! ```rust
//! use camelion::models::{Lab, Srgb, ToXyz};
//! let blue_on_lch = Lab::from(    // create color in lab.
//!     Srgb::new(0.0, 0.0, 1.0)
//!         .to_linear_light()      // convert to srgb-linear.
//!         .to_xyz()               // convert to xyz-d65.
//!         .transfer(),            // convert to xyz-d50.
//! )
//! .to_polar();                    // convert to lch.
//! ```

use crate::{
    color::{Color, Components, Space},
    models::{
        A98Rgb, A98RgbLinear, DisplayP3, DisplayP3Linear, Hsl, Hwb, Lab, Lch, Model, Oklab, Oklch,
        ProPhotoRgb, ProPhotoRgbLinear, Rec2020, Rec2020Linear, Srgb, SrgbLinear, XyzD50, XyzD65,
        D50, D65,
    },
};

impl Color {
    /// Convert this color from its current color space/notation to the
    /// specified color space/notation.
    pub fn to_space(&self, space: Space) -> Self {
        use Space as S;

        if self.space == space {
            return self.clone();
        }

        // Handle direct conversions.
        match (self.space, space) {
            (S::Srgb, S::SrgbLinear) => {
                return self
                    .as_model::<Srgb>()
                    .to_linear_light()
                    .to_color(self.alpha())
            }
            (S::SrgbLinear, S::Srgb) => {
                return self
                    .as_model::<SrgbLinear>()
                    .to_gamma_encoded()
                    .to_color(self.alpha())
            }
            (S::Srgb, S::Hsl) => return self.as_model::<Srgb>().to_hsl().to_color(self.alpha()),
            (S::Hsl, S::Srgb) => return self.as_model::<Hsl>().to_srgb().to_color(self.alpha()),
            (S::Srgb, S::Hwb) => return self.as_model::<Srgb>().to_hwb().to_color(self.alpha()),
            (S::Hwb, S::Srgb) => return self.as_model::<Hwb>().to_srgb().to_color(self.alpha()),
            (S::XyzD50, S::XyzD65) => {
                return self
                    .as_model::<XyzD50>()
                    .transfer::<D65>()
                    .to_color(self.alpha());
            }
            (S::XyzD65, S::XyzD50) => {
                return self
                    .as_model::<XyzD65>()
                    .transfer::<D50>()
                    .to_color(self.alpha())
            }
            (S::Hsl, S::Hwb) => {
                return self
                    .as_model::<Hsl>()
                    .to_srgb()
                    .to_hwb()
                    .to_color(self.alpha())
            }
            (S::Hwb, S::Hsl) => {
                return self
                    .as_model::<Hwb>()
                    .to_srgb()
                    .to_hsl()
                    .to_color(self.alpha())
            }
            (S::Lab, S::Lch) | (S::Oklab, S::Oklch) => {
                return self.as_model::<Lab>().to_polar().to_color(self.alpha())
            }
            (S::Lch, S::Lab) | (S::Oklch, S::Oklab) => {
                return self
                    .as_model::<Lch>()
                    .to_rectangular()
                    .to_color(self.alpha())
            }
            _ => {}
        }

        macro_rules! to_base {
            ($m:ident) => {{
                self.as_model::<$m>().to_base()
            }};
        }

        // The rest converts to XyzD50.
        use crate::models::ToBase;
        let base = match self.space {
            S::Srgb => to_base!(Srgb),
            S::SrgbLinear => to_base!(SrgbLinear),
            S::Hsl => to_base!(Hsl),
            S::Hwb => to_base!(Hwb),
            S::Lab => to_base!(Lab),
            S::Lch => to_base!(Lch),
            S::Oklab => to_base!(Oklab),
            S::Oklch => to_base!(Oklch),
            S::XyzD50 => to_base!(XyzD50),
            S::XyzD65 => to_base!(XyzD65),
            S::DisplayP3 => to_base!(DisplayP3),
            S::A98Rgb => to_base!(A98Rgb),
            S::ProPhotoRgb => to_base!(ProPhotoRgb),
            S::Rec2020 => to_base!(Rec2020),
        };

        match space {
            S::Srgb => SrgbLinear::from(base.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::SrgbLinear => SrgbLinear::from(base.transfer()).to_color(self.alpha()),
            S::Hsl => SrgbLinear::from(base.transfer())
                .to_gamma_encoded()
                .to_hsl()
                .to_color(self.alpha()),
            S::Hwb => SrgbLinear::from(base.transfer())
                .to_gamma_encoded()
                .to_hwb()
                .to_color(self.alpha()),
            S::Lab => Lab::from(base.transfer()).to_color(self.alpha()),
            S::Lch => Lab::from(base.transfer()).to_polar().to_color(self.alpha()),
            S::Oklab => Oklab::from(base.transfer()).to_color(self.alpha()),
            S::Oklch => Oklab::from(base.transfer())
                .to_polar()
                .to_color(self.alpha()),
            S::DisplayP3 => DisplayP3Linear::from(base.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::A98Rgb => A98RgbLinear::from(base.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::ProPhotoRgb => ProPhotoRgbLinear::from(base.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::Rec2020 => Rec2020Linear::from(base.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::XyzD50 => base.transfer::<D50>().to_color(self.alpha()),
            S::XyzD65 => base.transfer::<D65>().to_color(self.alpha()),
        }
    }
}

impl Srgb {
    /// Convert a color specified in the sRGB color space to the HSL notation.
    pub fn to_hsl(&self) -> Hsl {
        util::rgb_to_hsl(&Components(self.red, self.green, self.blue)).into()
    }

    /// Convert a color specified in the sRGB color space to the HWB notation.
    pub fn to_hwb(&self) -> Hwb {
        util::rgb_to_hwb(&Components(self.red, self.green, self.blue)).into()
    }
}

impl Hsl {
    /// Convert this color from the HSL notation to the sRGB color space.
    pub fn to_srgb(&self) -> Srgb {
        util::hsl_to_rgb(&Components(self.hue, self.saturation, self.lightness)).into()
    }
}

impl Hwb {
    /// Convert this color from the HWB notation to the sRGB color space.
    pub fn to_srgb(&self) -> Srgb {
        util::hwb_to_rgb(&Components(self.hue, self.whiteness, self.blackness)).into()
    }
}

mod util {
    use crate::{
        color::{Component, Components},
        math::{almost_zero, normalize, normalize_hue},
    };

    /// Calculate the hue from RGB components and return it along with the min
    /// and max RGB values.
    fn rgb_to_hue_with_min_max(from: &Components) -> (Component, Component, Component) {
        let Components(red, green, blue) = *from;

        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);

        let delta = max - min;

        let hue = if delta != 0.0 {
            60.0 * if max == red {
                (green - blue) / delta + if green < blue { 6.0 } else { 0.0 }
            } else if max == green {
                (blue - red) / delta + 2.0
            } else {
                (red - green) / delta + 4.0
            }
        } else {
            Component::NAN
        };

        (hue, min, max)
    }

    /// Convert from RGB notation to HSL notation.
    /// <https://drafts.csswg.org/css-color-4/#rgb-to-hsl>
    pub fn rgb_to_hsl(from: &Components) -> Components {
        let (hue, min, max) = rgb_to_hue_with_min_max(from);

        let lightness = (min + max) / 2.0;
        let delta = max - min;

        let saturation =
            if almost_zero(delta) || almost_zero(lightness) || almost_zero(1.0 - lightness) {
                0.0
            } else {
                (max - lightness) / lightness.min(1.0 - lightness)
            };

        Components(hue, saturation, lightness)
    }

    /// Convert from HSL notation to RGB notation.
    /// <https://drafts.csswg.org/css-color-4/#hsl-to-rgb>
    pub fn hsl_to_rgb(from: &Components) -> Components {
        let Components(hue, saturation, lightness) = from.map(normalize);

        if saturation <= 0.0 {
            return Components(lightness, lightness, lightness);
        }

        let hue = normalize_hue(hue);

        macro_rules! f {
            ($n:expr) => {{
                let k = ($n + hue / 30.0) % 12.0;
                let a = saturation * lightness.min(1.0 - lightness);
                lightness - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
            }};
        }

        Components(f!(0.0), f!(8.0), f!(4.0))
    }

    /// Convert from RGB notation to HWB notation.
    /// <https://drafts.csswg.org/css-color-4/#rgb-to-hwb>
    pub fn rgb_to_hwb(from: &Components) -> Components {
        let (hue, min, max) = rgb_to_hue_with_min_max(from);

        let whiteness = min;
        let blackness = 1.0 - max;

        Components(hue, whiteness, blackness)
    }

    /// Convert from HWB notation to RGB notation.
    /// <https://drafts.csswg.org/css-color-4/#hwb-to-rgb>
    pub fn hwb_to_rgb(from: &Components) -> Components {
        let hue = from.0;
        let whiteness = from.1;
        let blackness = from.2;

        if whiteness + blackness >= 1.0 {
            let gray = whiteness / (whiteness + blackness);
            return Components(gray, gray, gray);
        }

        let rgb = hsl_to_rgb(&Components(hue, 1.0, 0.5));
        rgb.map(|v| v * (1.0 - whiteness - blackness) + whiteness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_component_eq;
    use crate::color::{Color, Component, Space};

    #[test]
    fn test_conversions() {
        use Space as S;

        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        #[allow(clippy::type_complexity)]
        const TESTS: &[(Space, Component, Component, Component, Space, Component, Component, Component)] = &[
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::Srgb, 0.823529, 0.411765, 0.117647, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::Hsl, 25.000000, 0.750000, 0.470588, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::Hwb, 25.000000, 0.117647, 0.176471, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::Hsl, 25.000023, 0.750000, 0.470588),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::Hwb, 25.000023, 0.117647, 0.176471),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::Oklch, 0.634398, 0.154992, 50.266510),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::Rec2020, 0.669266, 0.401901, 0.142716),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::Lab, 56.629300, 39.237080, 57.553769, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::Hsl, 25.000023, 0.750000, 0.470588),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::Hwb, 25.000023, 0.117647, 0.176471),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::Oklch, 0.634398, 0.154992, 50.266510),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::Rec2020, 0.669266, 0.401901, 0.142716),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::Lch, 56.629300, 69.656190, 55.715927, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::Oklab, 0.634398, 0.099074, 0.119193, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::Oklch, 0.634398, 0.154992, 50.266483, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::SrgbLinear, 0.644480, 0.141263, 0.012983, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::DisplayP3, 0.770569, 0.434015, 0.199849, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::A98Rgb, 0.730405, 0.410688, 0.162005, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::Hsl, 25.000023, 0.750000, 0.470588),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::Hwb, 25.000023, 0.117647, 0.176471),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::Lab, 56.629303, 39.237063, 57.553794),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::Lch, 56.629303, 69.656201, 55.715950),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::Oklch, 0.634398, 0.154992, 50.266510),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::Rec2020, 0.669266, 0.401901, 0.142716),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::ProPhotoRgb, 0.592311, 0.394149, 0.164286, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::Rec2020, 0.669266, 0.401900, 0.142716, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::Hsl, 25.000023, 0.750000, 0.470588),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::Hwb, 25.000023, 0.117647, 0.176471),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::Lab, 56.629303, 39.237063, 57.553794),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::Lch, 56.629303, 69.656201, 55.715950),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::Oklch, 0.634398, 0.154992, 50.266510),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::Rec2020, 0.669266, 0.401901, 0.142716),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::XyzD50, 0.337301, 0.245449, 0.031959, S::XyzD65, 0.318634, 0.239006, 0.041637),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::Srgb, 0.823529, 0.411765, 0.117647),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::Hsl, 25.000000, 0.750000, 0.470588),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::Hwb, 25.000000, 0.117647, 0.176471),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::Lab, 56.629300, 39.237080, 57.553769),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::Lch, 56.629300, 69.656190, 55.715927),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::Oklab, 0.634398, 0.099074, 0.119193),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::Oklch, 0.634398, 0.154992, 50.266483),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::SrgbLinear, 0.644480, 0.141263, 0.012983),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::DisplayP3, 0.770569, 0.434015, 0.199849),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::A98Rgb, 0.730405, 0.410688, 0.162005),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::ProPhotoRgb, 0.592311, 0.394149, 0.164286),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::Rec2020, 0.669266, 0.401900, 0.142716),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::XyzD50, 0.337301, 0.245449, 0.031959),
            (S::XyzD65, 0.318634, 0.239006, 0.041637, S::XyzD65, 0.318634, 0.239006, 0.041637),
        ];

        for &(source_space, source_0, source_1, source_2, dest_space, dest_0, dest_1, dest_2) in
            TESTS
        {
            println!("{:?} -> {:?}", source_space, dest_space);
            let source = Color::new(source_space, source_0, source_1, source_2, 1.0);
            let dest = source.to_space(dest_space);
            assert_component_eq!(dest.components.0, dest_0);
            assert_component_eq!(dest.components.1, dest_1);
            assert_component_eq!(dest.components.2, dest_2);
        }
    }

    #[test]
    fn hue_is_powerless_if_there_is_no_chroma() {
        assert!(Srgb::new(1.0, 1.0, 1.0).to_hsl().hue.is_nan());
        assert!(Srgb::new(0.0, 0.0, 0.0).to_hsl().hue.is_nan());
        assert!(Srgb::new(0.5, 0.5, 0.5).to_hsl().hue.is_nan());
    }

    #[test]
    fn hwb_to_rgb() {
        // hwb(40deg 30% 40%)
        let hwb = Color::new(Space::Hwb, 40.0, 0.3, 0.4, 1.0);
        // rgb(153, 128, 77)
        let srgb = hwb.to_space(Space::Srgb);

        assert_component_eq!(srgb.components.0, 0.6);
        assert_component_eq!(srgb.components.1, 0.5);
        assert_component_eq!(srgb.components.2, 0.3);

        // assert_component_eq!((srgb.components.0 * 255.0).round() as u8, 153);
        // assert_component_eq!((srgb.components.1 * 255.0).round() as u8, 128);
        // assert_component_eq!((srgb.components.2 * 255.0).round() as u8, 77);
    }

    #[test]
    fn converting_a_color_should_maintain_source_alpha() {
        let hsl = Color::new(Space::Hsl, 120.0, 0.4, 0.4, None);
        let srgb = hsl.to_space(Space::Srgb);
        assert!(srgb.alpha().is_none());
    }

    #[test]
    fn alpha_is_clamped_after_conversion() {
        // color-mix(in srgb, color(srgb 2 3 4 / 5), color(srgb 4 6 8 / 10))
        let left = Color::new(Space::Srgb, 2.0, 3.0, 4.0, 5.0);
        let right = Color::new(Space::Srgb, 4.0, 6.0, 8.0, 10.0);
        let interp = left.interpolate(&right, Space::Srgb);
        let result = interp.at(0.5);
        // color(srgb 3 4.5 6)
        assert_eq!(result.alpha(), Some(1.0));
    }

    #[test]
    fn rgb_to_hsl() {
        // color(srgb 0.46 0.52 0.28 / 0.5)
        let srgb = Color::new(Space::Srgb, 0.46, 0.52, 0.28, 0.5);
        let hsl = srgb.to_space(Space::Hsl);
        assert_component_eq!(hsl.components.0, 75.0);
        assert_component_eq!(hsl.components.1, 0.3);
        assert_component_eq!(hsl.components.2, 0.4);
    }
}
