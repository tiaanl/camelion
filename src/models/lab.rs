//! Models for rectangular and polar coordinate systems used to model CIE-Lab,
//! CIE-Lch, Oklab and Oklch.

use crate::{
    color::{Component, Components, HasSpace, Space},
    math::{almost_zero, normalize_hue, transform, transform_3x3, Transform},
    models::xyz::{ToXyz, WhitePoint, Xyz, XyzD50, XyzD65, D50, D65},
};

mod space {
    pub trait Space {}

    #[derive(Clone, Debug)]
    pub struct Lab;
    impl Space for Lab {}

    #[derive(Clone, Debug)]
    pub struct Oklab;
    impl Space for Oklab {}
}

camelion_macros::gen_model! {
    /// The model for a color specified in the rectangular orthogonal form.
    pub struct Rectangular<S: space::Space> {
        /// The lightness component.
        pub lightness: Component,
        /// The a component.
        pub a: Component,
        /// The b component.
        pub b: Component,
    }
}

impl<S: space::Space> Rectangular<S> {
    /// Convert this orthogonal rectangular model into its cylindrical polar
    /// form.
    pub fn to_polar(&self) -> Polar<S> {
        let chroma = (self.a * self.a + self.b * self.b).sqrt();
        let hue = if almost_zero(chroma) {
            Component::NAN
        } else {
            normalize_hue(self.b.atan2(self.a).to_degrees())
        };

        Polar::new(self.lightness, chroma, hue)
    }
}

camelion_macros::gen_model! {
    /// The model for a color specified in the cylindrical polar form.
    pub struct Polar<S: space::Space> {
        /// The lightness component.
        pub lightness: Component,
        /// The chroma component.
        pub chroma: Component,
        /// The hue component.
        pub hue: Component,
    }
}

impl<S: space::Space> Polar<S> {
    /// Convert this cylindrical polar model into its orthogonal rectangular
    /// form.
    pub fn to_rectangular(&self) -> Rectangular<S> {
        let hue = self.hue.to_radians();
        let a = self.chroma * hue.cos();
        let b = self.chroma * hue.sin();

        Rectangular::new(self.lightness, a, b)
    }
}

/// The model for a color specified in the CIE-Lab color space with the rectangular orthogonal form.
pub type Lab = Rectangular<space::Lab>;

impl HasSpace for Lab {
    const SPACE: Space = Space::Lab;
}

impl ToXyz<D50> for Lab {
    fn to_xyz(&self) -> Xyz<D50> {
        const KAPPA: Component = 24389.0 / 27.0;
        const EPSILON: Component = 216.0 / 24389.0;

        // To avoid accessing the values through self all the time.
        let (lightness, a, b) = (self.lightness, self.a, self.b);

        let f1 = (lightness + 16.0) / 116.0;
        let f0 = f1 + a / 500.0;
        let f2 = f1 - b / 200.0;

        let f0_cubed = f0 * f0 * f0;
        let x = if f0_cubed > EPSILON {
            f0_cubed
        } else {
            (116.0 * f0 - 16.0) / KAPPA
        };

        let y = if lightness > KAPPA * EPSILON {
            let v = (lightness + 16.0) / 116.0;
            v * v * v
        } else {
            lightness / KAPPA
        };

        let f2_cubed = f2 * f2 * f2;
        let z = if f2_cubed > EPSILON {
            f2_cubed
        } else {
            (116.0 * f2 - 16.0) / KAPPA
        };

        Xyz::new(
            x * D50::WHITE_POINT.0,
            y * D50::WHITE_POINT.1,
            z * D50::WHITE_POINT.2,
        )
    }
}

impl From<XyzD50> for Lab {
    fn from(value: XyzD50) -> Self {
        const KAPPA: Component = 24389.0 / 27.0;
        const EPSILON: Component = 216.0 / 24389.0;

        let adapted = Components(
            value.x / D50::WHITE_POINT.0,
            value.y / D50::WHITE_POINT.1,
            value.z / D50::WHITE_POINT.2,
        );

        // 4. Convert D50-adapted XYZ to Lab.
        let Components(f0, f1, f2) = adapted.map(|v| {
            if v > EPSILON {
                v.cbrt()
            } else {
                (KAPPA * v + 16.0) / 116.0
            }
        });

        let lightness = 116.0 * f1 - 16.0;
        let a = 500.0 * (f0 - f1);
        let b = 200.0 * (f1 - f2);

        Lab::new(lightness, a, b)
    }
}

/// The model for a color specified in the CIE-Lab color space with the cylindrical polar form.
pub type Lch = Polar<space::Lab>;

impl HasSpace for Lch {
    const SPACE: Space = Space::Lch;
}

/// The model for a color specified in the oklab color space with the rectangular orthogonal form.
pub type Oklab = Rectangular<space::Oklab>;

impl HasSpace for Oklab {
    const SPACE: Space = Space::Oklab;
}

impl From<XyzD65> for Oklab {
    fn from(value: XyzD65) -> Self {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const XYZ_TO_LMS: Transform = transform_3x3(
             0.8190224432164319,  0.0329836671980271,  0.048177199566046255,
             0.3619062562801221,  0.9292868468965546,  0.26423952494422764,
            -0.12887378261216414, 0.03614466816999844, 0.6335478258136937,
        );

        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const LMS_TO_OKLAB: Transform = transform_3x3(
             0.2104542553,  1.9779984951,  0.0259040371,
             0.7936177850, -2.4285922050,  0.7827717662,
            -0.0040720468,  0.4505937099, -0.8086757660,
        );

        let lms = transform(&XYZ_TO_LMS, Components(value.x, value.y, value.z));
        let lms = lms.map(|v| v.cbrt());
        transform(&LMS_TO_OKLAB, lms).into()
    }
}

impl ToXyz<D65> for Oklab {
    fn to_xyz(&self) -> Xyz<D65> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const OKLAB_TO_LMS: Transform = transform_3x3(
            0.99999999845051981432,  1.0000000088817607767,    1.0000000546724109177,
            0.39633779217376785678, -0.1055613423236563494,   -0.089484182094965759684,
            0.21580375806075880339, -0.063854174771705903402, -1.2914855378640917399,
        );

        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const LMS_TO_XYZ: Transform = transform_3x3(
             1.2268798733741557,  -0.04057576262431372, -0.07637294974672142,
            -0.5578149965554813,   1.1122868293970594,  -0.4214933239627914,
             0.28139105017721583, -0.07171106666151701,  1.5869240244272418,
        );

        let lms = transform(&OKLAB_TO_LMS, Components(self.lightness, self.a, self.b));
        let lms = lms.map(|v| v * v * v);
        transform(&LMS_TO_XYZ, lms).into()
    }
}

/// The model for a color specified in the oklab color space with the cylindrical polar form.
pub type Oklch = Polar<space::Oklab>;

impl HasSpace for Oklch {
    const SPACE: Space = Space::Oklch;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converting_to_polar_should_set_hue_to_missing_for_small_a_b_values() {
        let lab = Lab::new(50.0, 0.0, 0.0);
        let polar = lab.to_polar();

        assert!(polar.hue.is_nan());
    }
}
