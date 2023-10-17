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
        ProPhotoRgb, ProPhotoRgbLinear, Rec2020, Rec2020Linear, Srgb, SrgbLinear, ToXyz, XyzD50,
        XyzD65, D50, D65,
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
            _ => {}
        }

        // The rest converts to XyzD50.
        let xyz: XyzD50 = match self.space {
            S::Srgb => self
                .as_model::<Srgb>()
                .to_linear_light()
                .to_xyz()
                .transfer(),
            S::SrgbLinear => self.as_model::<SrgbLinear>().to_xyz().transfer(),
            S::Hsl => self
                .as_model::<Hsl>()
                .to_srgb()
                .to_linear_light()
                .to_xyz()
                .transfer(),
            S::Hwb => self
                .as_model::<Hwb>()
                .to_srgb()
                .to_linear_light()
                .to_xyz()
                .transfer(),
            S::Lab => self.as_model::<Lab>().to_xyz(),
            S::Lch => self.as_model::<Lch>().to_rectangular().to_xyz(),
            S::Oklab => self.as_model::<Oklab>().to_xyz().transfer(),
            S::Oklch => self
                .as_model::<Oklch>()
                .to_rectangular()
                .to_xyz()
                .transfer(),
            S::XyzD50 => self.as_model::<XyzD50>().clone(),
            S::XyzD65 => self.as_model::<XyzD65>().transfer(),
            S::DisplayP3 => self
                .as_model::<DisplayP3>()
                .to_linear_light()
                .to_xyz()
                .transfer(),
            S::A98Rgb => self
                .as_model::<A98Rgb>()
                .to_linear_light()
                .to_xyz()
                .transfer(),
            S::ProPhotoRgb => self.as_model::<ProPhotoRgb>().to_linear_light().to_xyz(),
            S::Rec2020 => self
                .as_model::<Rec2020>()
                .to_linear_light()
                .to_xyz()
                .transfer(),
        };

        match space {
            S::Srgb => SrgbLinear::from(xyz.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::SrgbLinear => SrgbLinear::from(xyz.transfer()).to_color(self.alpha()),
            S::Hsl => SrgbLinear::from(xyz.transfer())
                .to_gamma_encoded()
                .to_hsl()
                .to_color(self.alpha()),
            S::Hwb => SrgbLinear::from(xyz.transfer())
                .to_gamma_encoded()
                .to_hwb()
                .to_color(self.alpha()),
            S::Lab => Lab::from(xyz).to_color(self.alpha()),
            S::Lch => Lab::from(xyz).to_polar().to_color(self.alpha()),
            S::Oklab => Oklab::from(xyz.transfer()).to_color(self.alpha()),
            S::Oklch => Oklab::from(xyz.transfer())
                .to_polar()
                .to_color(self.alpha()),
            S::DisplayP3 => DisplayP3Linear::from(xyz.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::A98Rgb => A98RgbLinear::from(xyz.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::ProPhotoRgb => ProPhotoRgbLinear::from(xyz)
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::Rec2020 => Rec2020Linear::from(xyz.transfer())
                .to_gamma_encoded()
                .to_color(self.alpha()),
            S::XyzD50 => xyz.to_color(self.alpha()),
            S::XyzD65 => xyz.transfer::<D65>().to_color(self.alpha()),
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
    use crate::color::{Component, Components};

    /// Calculate the hue from RGB components and return it along with the min
    /// and max RGB values.
    fn rgb_to_hue_min_max(from: &Components) -> (Component, Component, Component) {
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
        let (hue, min, max) = rgb_to_hue_min_max(from);

        let lightness = (min + max) / 2.0;
        let delta = max - min;

        let saturation = if delta != 0.0 {
            if lightness == 0.0 || lightness == 1.0 {
                0.0
            } else {
                (max - lightness) / lightness.min(1.0 - lightness)
            }
        } else {
            0.0
        };

        Components(hue, saturation, lightness)
    }

    /// Convert from HSL notation to RGB notation.
    /// <https://drafts.csswg.org/css-color-4/#hsl-to-rgb>
    pub fn hsl_to_rgb(from: &Components) -> Components {
        let saturation = if from.1.is_nan() { 0.0 } else { from.1 };
        let lightness = if from.2.is_nan() { 0.0 } else { from.2 };

        if saturation <= 0.0 {
            return Components(lightness, lightness, lightness);
        }

        let hue = if from.0.is_nan() {
            0.0
        } else {
            from.0.rem_euclid(360.0)
        };

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
        let (hue, min, max) = rgb_to_hue_min_max(from);

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
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::Srgb, 0.82352941, 0.41176471, 0.11764706, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::Hsl, 25.00000000, 0.75000000, 0.47058824, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::Hwb, 25.00000000, 0.11764706, 0.17647059, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::Lab, 56.62930022, 39.23708020, 57.55376917, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::Lch, 56.62930022, 69.65619002, 55.71592715, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            // (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            // (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            // (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            // (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            // (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::Oklab, 0.63439842, 0.09907391, 0.11919316, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            // (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            // (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            // (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            // (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::Oklch, 0.63439842, 0.15499242, 50.26648308, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::DisplayP3, 0.77056903, 0.43401475, 0.19984926, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            // (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::A98Rgb, 0.73040524, 0.41068841, 0.16200485, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::Rec2020, 0.66926598, 0.40190046, 0.14271567, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::XyzD50, 0.33730087, 0.24544919, 0.03195887, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::Srgb, 0.82352941, 0.41176471, 0.11764706),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::Hsl, 25.00000000, 0.75000000, 0.47058824),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::Hwb, 25.00000000, 0.11764706, 0.17647059),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::Lab, 56.62930022, 39.23708020, 57.55376917),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::Lch, 56.62930022, 69.65619002, 55.71592715),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::Oklab, 0.63439842, 0.09907391, 0.11919316),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::Oklch, 0.63439842, 0.15499242, 50.26648308),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::SrgbLinear, 0.64447968, 0.14126329, 0.01298303),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::DisplayP3, 0.77056903, 0.43401475, 0.19984926),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::A98Rgb, 0.73040524, 0.41068841, 0.16200485),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::ProPhotoRgb, 0.59231119, 0.39414858, 0.16428630),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::Rec2020, 0.66926598, 0.40190046, 0.14271567),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::XyzD50, 0.33730087, 0.24544919, 0.03195887),
            (S::XyzD65, 0.31863422, 0.23900588, 0.04163696, S::XyzD65, 0.31863422, 0.23900588, 0.04163696),
        ];

        for &(source_space, source_0, source_1, source_2, dest_space, dest_0, dest_1, dest_2) in
            TESTS
        {
            let source = Color::new(source_space, source_0, source_1, source_2, 1.0);
            let dest = source.to_space(dest_space);
            println!("{:?} -> {:?}", source_space, dest_space);
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
}
