use std::marker::PhantomData;

use crate::{
    color::{ComponentDetails, SpacePlaceholder},
    math::{transform, Transform},
    xyz::{WhitePoint, Xyz},
    Color, Component, Components, Flags, Space, ToXyz, XyzD50, XyzD65, D50, D65,
};

mod space {
    pub trait Space {}

    #[derive(Debug)]
    pub struct Lab;
    impl Space for Lab {}

    #[derive(Debug)]
    pub struct Oklab;
    impl Space for Oklab {}
}

/// The model for a color specified in the rectangular orthogonal form.
#[derive(Debug)]
pub struct Rectangular<S: space::Space> {
    pub lightness: Component,
    pub a: Component,
    pub b: Component,
    pub alpha: Component,
    pub flags: Flags,

    _space: SpacePlaceholder,
    _s: PhantomData<S>,
}

impl<S: space::Space> Rectangular<S> {
    /// Create a new color with a rectangular orthogonal form.
    pub fn new(
        lightness: impl Into<ComponentDetails>,
        a: impl Into<ComponentDetails>,
        b: impl Into<ComponentDetails>,
        alpha: impl Into<ComponentDetails>,
    ) -> Self {
        let mut flags = Flags::empty();

        let lightness = lightness
            .into()
            .value_and_flag(&mut flags, Flags::C0_IS_NONE);
        let a = a.into().value_and_flag(&mut flags, Flags::C1_IS_NONE);
        let b = b.into().value_and_flag(&mut flags, Flags::C2_IS_NONE);
        let alpha = alpha
            .into()
            .value_and_flag(&mut flags, Flags::ALPHA_IS_NONE);

        Self {
            lightness,
            a,
            b,
            alpha,
            flags,
            _space: 0,
            _s: PhantomData,
        }
    }

    pub fn to_polar(&self) -> Polar<S> {
        let hue = self.b.atan2(self.a).to_degrees().rem_euclid(360.0);
        let chroma = (self.a * self.a + self.b * self.b).sqrt();

        Polar::new(self.lightness, chroma, hue, self.alpha)
    }
}

/// The model for a color specified in the cylindrical polar form.
pub struct Polar<S: space::Space> {
    pub lightness: Component,
    pub chroma: Component,
    pub hue: Component,
    pub alpha: Component,
    pub flags: Flags,

    _space: SpacePlaceholder,
    _s: PhantomData<S>,
}

impl<S: space::Space> Polar<S> {
    /// Create a new color with the cylindrical polar form.
    pub fn new(
        lightness: impl Into<ComponentDetails>,
        chroma: impl Into<ComponentDetails>,
        hue: impl Into<ComponentDetails>,
        alpha: impl Into<ComponentDetails>,
    ) -> Self {
        let mut flags = Flags::empty();

        let lightness = lightness
            .into()
            .value_and_flag(&mut flags, Flags::C0_IS_NONE);
        let chroma = chroma.into().value_and_flag(&mut flags, Flags::C1_IS_NONE);
        let hue = hue.into().value_and_flag(&mut flags, Flags::C2_IS_NONE);
        let alpha = alpha
            .into()
            .value_and_flag(&mut flags, Flags::ALPHA_IS_NONE);

        Self {
            lightness,
            chroma,
            hue,
            alpha,
            flags,
            _space: 0,
            _s: PhantomData,
        }
    }

    pub fn to_rectangular(&self) -> Rectangular<S> {
        let hue = self.hue.to_radians();
        let a = self.chroma * hue.cos();
        let b = self.chroma * hue.sin();

        Rectangular::new(self.lightness, a, b, self.alpha)
    }
}

/// The model for a color specified in the CIE-Lab color space with the rectangular orthogonal form.
pub type Lab = Rectangular<space::Lab>;

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
            self.alpha,
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

        Lab::new(lightness, a, b, value.alpha)
    }
}

impl From<Lab> for Color {
    fn from(value: Lab) -> Self {
        Color::new(Space::Lab, value.lightness, value.a, value.b, value.alpha)
    }
}

/// The model for a color specified in the CIE-Lab color space with the cylindrical polar form.
pub type Lch = Polar<space::Lab>;

impl From<Lch> for Color {
    fn from(value: Lch) -> Self {
        Color::new(
            Space::Lch,
            value.lightness,
            value.chroma,
            value.hue,
            value.alpha,
        )
    }
}

/// The model for a color specified in the oklab color space with the rectangular orthogonal form.
pub type Oklab = Rectangular<space::Oklab>;

impl From<XyzD65> for Oklab {
    fn from(value: XyzD65) -> Self {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const XYZ_TO_LMS: Transform = Transform::new(
             0.8190224432164319,  0.0329836671980271,  0.048177199566046255, 0.0,
             0.3619062562801221,  0.9292868468965546,  0.26423952494422764,  0.0,
            -0.12887378261216414, 0.03614466816999844, 0.6335478258136937,   0.0,
             0.0,                 0.0,                 0.0,                  1.0,
        );

        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const LMS_TO_OKLAB: Transform = Transform::new(
             0.2104542553,  1.9779984951,  0.0259040371, 0.0,
             0.7936177850, -2.4285922050,  0.7827717662, 0.0,
            -0.0040720468,  0.4505937099, -0.8086757660, 0.0,
             0.0,           0.0,           0.0,          1.0,
        );

        let lms = transform(&XYZ_TO_LMS, value.x, value.y, value.z);
        let [x, y, z] = lms.map(|v| v.cbrt());
        let [lightness, a, b] = transform(&LMS_TO_OKLAB, x, y, z);
        Self::new(lightness, a, b, value.alpha)
    }
}

impl ToXyz<D65> for Oklab {
    fn to_xyz(&self) -> Xyz<D65> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const OKLAB_TO_LMS: Transform = Transform::new(
            0.99999999845051981432,  1.0000000088817607767,    1.0000000546724109177,   0.0,
            0.39633779217376785678, -0.1055613423236563494,   -0.089484182094965759684, 0.0,
            0.21580375806075880339, -0.063854174771705903402, -1.2914855378640917399,   0.0,
            0.0,                     0.0,                      0.0,                     1.0,
        );

        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const LMS_TO_XYZ: Transform = Transform::new(
             1.2268798733741557,  -0.04057576262431372, -0.07637294974672142, 0.0,
            -0.5578149965554813,   1.1122868293970594,  -0.4214933239627914,  0.0,
             0.28139105017721583, -0.07171106666151701,  1.5869240244272418,  0.0,
             0.0,                  0.0,                  0.0,                 1.0,
        );

        let [x, y, z] = transform(&OKLAB_TO_LMS, self.lightness, self.a, self.b);
        let x = x * x * x;
        let y = y * y * y;
        let z = z * z * z;
        let [x, y, z] = transform(&LMS_TO_XYZ, x, y, z);

        Xyz::new(x, y, z, self.alpha)
    }
}

impl From<Oklab> for Color {
    fn from(value: Oklab) -> Self {
        Color::new(Space::Oklab, value.lightness, value.a, value.b, value.alpha)
    }
}

/// The model for a color specified in the oklab color space with the cylindrical polar form.
pub type Oklch = Polar<space::Oklab>;

impl From<Oklch> for Color {
    fn from(value: Oklch) -> Self {
        Color::new(
            Space::Oklab,
            value.lightness,
            value.chroma,
            value.hue,
            value.alpha,
        )
    }
}
