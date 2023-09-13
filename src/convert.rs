//! Implementations on all the models that has conversions to other models.

use crate::{Color, Component, Components, Hsl, Hwb, Space, Srgb, SrgbLinear, XyzD50, XyzD65};

type Transform = euclid::default::Transform3D<Component>;
type Vector = euclid::default::Vector3D<Component>;

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
                let srgb: &Srgb = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(srgb.to_linear_light()) };
            }
            (S::SrgbLinear, S::Srgb) => {
                let srgb_linear: &SrgbLinear = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(srgb_linear.to_gamma_encoded()) };
            }
            (S::Srgb, S::Hsl) => {
                let srgb: &Srgb = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(srgb.to_hsl()) };
            }
            (S::Hsl, S::Srgb) => {
                let hsl: &Hsl = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(hsl.to_srgb()) };
            }
            (S::Srgb, S::Hwb) => {
                let srgb: &Srgb = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(srgb.to_hwb()) };
            }
            (S::Hwb, S::Srgb) => {
                let hwb: &Hwb = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(hwb.to_srgb()) };
            }
            (S::XyzD50, S::XyzD65) => {
                let xyz_d50: &XyzD50 = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(xyz_d50.to_xyz_d65()) };
            }
            (S::XyzD65, S::XyzD50) => {
                let xyz_d65: &XyzD65 = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(xyz_d65.to_xyz_d50()) };
            }
            _ => {}
        }

        // Handle a special case where the connecting space is sRGB.
        match (self.space, space) {
            (S::Hsl, S::Hwb) => {
                let hsl: &Hsl = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(hsl.to_srgb().to_hwb()) };
            }
            (S::Hwb, S::Hsl) => {
                let hwb: &Hwb = unsafe { std::mem::transmute(self) };
                return unsafe { std::mem::transmute(hwb.to_srgb().to_hsl()) };
            }
            _ => {}
        }

        todo!()
    }
}

impl Srgb {
    /// Convert a gamma encoded sRGB color to a sRGB color without gamma
    /// encoding (linear light).
    pub fn to_linear_light(&self) -> SrgbLinear {
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

    /// Convert a color specified in the sRGB color space to the HSL notation.
    pub fn to_hsl(&self) -> Hsl {
        let Components(hue, saturation, lightness) =
            util::rgb_to_hsl(&Components(self.red, self.green, self.blue));
        Hsl::new(hue, saturation, lightness, self.alpha)
    }

    /// Convert a color specified in the sRGB color space to the HWB notation.
    pub fn to_hwb(&self) -> Hwb {
        let Components(hue, whitenss, blackness) =
            util::rgb_to_hwb(&Components(self.red, self.green, self.blue));
        Hwb::new(hue, whitenss, blackness, self.alpha)
    }
}

impl SrgbLinear {
    /// Convert a sRGB color without gamma encoding (linear light) to a sRGB
    /// color with gamma encoding.
    pub fn to_gamma_encoded(&self) -> Srgb {
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

impl Hsl {
    /// Convert this color from the HSL notation to the sRGB color space.
    pub fn to_srgb(&self) -> Srgb {
        let Components(red, green, blue) =
            util::hsl_to_rgb(&Components(self.hue, self.saturation, self.lightness));
        Srgb::new(red, green, blue, self.alpha)
    }
}

impl Hwb {
    /// Convert this color from the HWB notation to the sRGB color space.
    pub fn to_srgb(&self) -> Srgb {
        let Components(red, green, blue) =
            util::hwb_to_rgb(&Components(self.hue, self.whiteness, self.blackness));
        Srgb::new(red, green, blue, self.alpha)
    }
}

impl XyzD50 {
    pub fn to_xyz_d65(&self) -> XyzD65 {
        #[rustfmt::skip]
        const MAT: Transform = Transform::new(
             0.9554734527042182,   -0.028369706963208136,  0.012314001688319899, 0.0,
            -0.023098536874261423,  1.0099954580058226,   -0.020507696433477912, 0.0,
             0.0632593086610217,    0.021041398966943008,  1.3303659366080753,   0.0,
             0.0,                   0.0,                   0.0,                  1.0,
        );

        let result = MAT.transform_vector3d(Vector::new(self.x, self.y, self.z));
        XyzD65::new(result.x, result.y, result.z, self.alpha)
    }
}

impl XyzD65 {
    pub fn to_xyz_d50(&self) -> XyzD50 {
        #[rustfmt::skip]
        const MAT: Transform = Transform::new(
            1.0479298208405488,    0.029627815688159344, -0.009243058152591178, 0.0,
            0.022946793341019088,  0.990434484573249,     0.015055144896577895, 0.0,
            -0.05019222954313557,  -0.01707382502938514,   0.7518742899580008,   0.0,
            0.0,                   0.0,                   0.0,                  1.0,
        );

        let result = MAT.transform_vector3d(Vector::new(self.x, self.y, self.z));
        XyzD50::new(result.x, result.y, result.z, self.alpha)
    }
}

mod util {
    use crate::{Component, Components};

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
        fn hue_to_rgb(t1: Component, t2: Component, hue: Component) -> Component {
            let hue = hue.rem_euclid(360.0);

            if hue * 6.0 < 360.0 {
                t1 + (t2 - t1) * hue / 60.0
            } else if hue * 2.0 < 360.0 {
                t2
            } else if hue * 3.0 < 720.0 {
                t1 + (t2 - t1) * (240.0 - hue) / 60.0
            } else {
                t1
            }
        }

        let Components(hue, saturation, lightness) = *from;

        let t2 = if lightness <= 0.5 {
            lightness * (saturation + 1.0)
        } else {
            lightness + saturation - lightness * saturation
        };
        let t1 = lightness * 2.0 - t2;

        Components(
            hue_to_rgb(t1, t2, hue + 120.0),
            hue_to_rgb(t1, t2, hue),
            hue_to_rgb(t1, t2, hue - 120.0),
        )
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
        let Components(hue, whiteness, blackness) = *from;

        if whiteness + blackness > 1.0 {
            let gray = whiteness / (whiteness + blackness);
            return Components(gray, gray, gray);
        }

        let x = 1.0 - whiteness - blackness;
        hsl_to_rgb(&Components(hue, 1.0, 0.5)).map(|v| v * x + whiteness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Component;

    macro_rules! assert_component_eq {
        ($actual:expr,$expected:expr) => {{
            assert!(
                ($actual - $expected).abs() <= Component::EPSILON,
                "component {} it not equal to {}",
                $actual,
                $expected
            )
        }};
    }

    #[test]
    fn convert_srgb_to_srgb_linear() {
        let srgb_linear = Srgb::new(0.1804, 0.5451, 0.3412, 1.0).to_linear_light();
        assert_component_eq!(srgb_linear.red, 0.027323073);
        assert_component_eq!(srgb_linear.green, 0.25818488);
        assert_component_eq!(srgb_linear.blue, 0.09532106);
    }

    #[test]
    fn convert_srgb_to_hsl() {
        let hsl = Srgb::new(0.1804, 0.5451, 0.3412, 1.0).to_hsl();
        assert_component_eq!(hsl.hue, 146.45462);
        assert_component_eq!(hsl.saturation, 0.50268775);
        assert_component_eq!(hsl.lightness, 0.36275);
    }

    #[test]
    fn convert_srgb_to_hwb() {
        let hwb = Srgb::new(0.1804, 0.5451, 0.3412, 1.0).to_hwb();
        assert_component_eq!(hwb.hue, 146.45462);
        assert_component_eq!(hwb.whiteness, 0.1804);
        assert_component_eq!(hwb.blackness, 0.45490003);
    }

    #[test]
    fn convert_srgb_linear_to_srgb() {
        let srgb = SrgbLinear::new(0.0319, 0.6105, 0.0319, 1.0).to_gamma_encoded();
        assert_component_eq!(srgb.red, 0.19609144);
        assert_component_eq!(srgb.green, 0.8039241);
        assert_component_eq!(srgb.blue, 0.19609144);
    }

    #[test]
    fn convert_hsl_to_srgb() {
        let srgb = Hsl::new(210.0, 0.5, 0.3, 1.0).to_srgb();
        assert_component_eq!(srgb.red, 0.15);
        assert_component_eq!(srgb.green, 0.3);
        assert_component_eq!(srgb.blue, 0.45);
    }

    #[test]
    fn convert_hwb_to_srgb() {
        let srgb = Hwb::new(210.0, 0.15, 0.55, 1.0).to_srgb();
        assert_component_eq!(srgb.red, 0.15);
        assert_component_eq!(srgb.green, 0.3);
        assert_component_eq!(srgb.blue, 0.45);
    }

    #[test]
    fn convert_xyz_d50_to_xyz_d65() {
        let xyz_d65 = XyzD50::new(0.1, 0.2, 0.3, 0.4).to_xyz_d65();
        assert_component_eq!(xyz_d65.x, 0.10990542);
        assert_component_eq!(xyz_d65.y, 0.20547454);
        assert_component_eq!(xyz_d65.z, 0.39623964);
    }

    #[test]
    fn convert_xyz_d65_to_xyz_d50() {
        let xyz_d50 = XyzD65::new(0.1, 0.2, 0.3, 0.4).to_xyz_d50();
        assert_component_eq!(xyz_d50.x, 0.09432466);
        assert_component_eq!(xyz_d50.y, 0.19592753);
        assert_component_eq!(xyz_d50.z, 0.22764902);
    }
}
