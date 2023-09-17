use std::marker::PhantomData;

use crate::{
    color::{ComponentDetails, SpacePlaceholder},
    xyz::white_point::WhitePoint,
    Color, Component, Components, Flags, Space, Transform, Vector, XyzD50, XyzD65, D50,
};

mod space {
    pub trait Space {}

    pub struct Lab;
    impl Space for Lab {}

    pub struct Oklab;
    impl Space for Oklab {}
}

/// The model for a color specified in the rectangular orthogonal form.
pub struct RectangularOrthogonal<S: space::Space> {
    pub lightness: Component,
    pub a: Component,
    pub b: Component,
    pub alpha: Component,
    pub flags: Flags,

    _space: SpacePlaceholder,
    _s: PhantomData<S>,
}

impl<S: space::Space> RectangularOrthogonal<S> {
    /// Create a new color in the rectangular orthogonal form.
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

    pub fn to_cylindrical_polar(&self) -> CylindricalPolar<S> {
        let hue = self.b.atan2(self.a).to_degrees().rem_euclid(360.0);
        let chroma = (self.a * self.a + self.b * self.b).sqrt();

        CylindricalPolar::new(self.lightness, chroma, hue, self.alpha)
    }
}

/// The model for a color specified in the cylindrical polar form.
pub struct CylindricalPolar<S: space::Space> {
    pub lightness: Component,
    pub chroma: Component,
    pub hue: Component,
    pub alpha: Component,
    pub flags: Flags,

    _space: SpacePlaceholder,
    _s: PhantomData<S>,
}

impl<S: space::Space> CylindricalPolar<S> {
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

    pub fn to_rectangular_orthogonal(&self) -> RectangularOrthogonal<S> {
        let hue = self.hue.to_radians();
        let a = self.chroma * hue.cos();
        let b = self.chroma * hue.sin();

        RectangularOrthogonal::new(self.lightness, a, b, self.alpha)
    }
}

/// The model for a color specified in the CIE-Lab color space with the rectangular orthogonal form.
pub type Lab = RectangularOrthogonal<space::Lab>;

impl From<XyzD50> for Lab {
    fn from(value: XyzD50) -> Self {
        const KAPPA: f32 = 24389.0 / 27.0;
        const EPSILON: f32 = 216.0 / 24389.0;

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
pub type Lch = CylindricalPolar<space::Lab>;

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
pub type Oklab = RectangularOrthogonal<space::Oklab>;

impl From<XyzD65> for Oklab {
    fn from(value: XyzD65) -> Self {
        #[rustfmt::skip]
        const XYZ_TO_LMS: Transform = Transform::new(
             0.8190224432164319,  0.0329836671980271,  0.048177199566046255, 0.0,
             0.3619062562801221,  0.9292868468965546,  0.26423952494422764,  0.0,
            -0.12887378261216414, 0.03614466816999844, 0.6335478258136937,   0.0,
             0.0,                 0.0,                 0.0,                  1.0,
        );

        #[rustfmt::skip]
        const LMS_TO_OKLAB: Transform = Transform::new(
             0.2104542553,  1.9779984951,  0.0259040371, 0.0,
             0.7936177850, -2.4285922050,  0.7827717662, 0.0,
            -0.0040720468,  0.4505937099, -0.8086757660, 0.0,
             0.0,           0.0,           0.0,          1.0,
        );

        let lms = XYZ_TO_LMS.transform_vector3d(Vector::new(value.x, value.y, value.z));
        let lms = Vector::new(lms.x.cbrt(), lms.y.cbrt(), lms.z.cbrt());
        let Vector { x, y, z, .. } = LMS_TO_OKLAB.transform_vector3d(lms);

        Self::new(x, y, z, value.alpha)
    }
}

impl From<Oklab> for Color {
    fn from(value: Oklab) -> Self {
        Color::new(Space::Oklab, value.lightness, value.a, value.b, value.alpha)
    }
}

/// The model for a color specified in the oklab color space with the cylindrical polar form.
pub type Oklch = CylindricalPolar<space::Oklab>;

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
