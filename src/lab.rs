use std::marker::PhantomData;

use crate::{
    color::{ComponentDetails, SpacePlaceholder},
    Component, Flags,
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

/// The model for a color specified in the CIE-Lab color space with the cylindrical polar form.
pub type Lch = CylindricalPolar<space::Lab>;

/// The model for a color specified in the oklab color space with the rectangular orthogonal form.
pub type Oklab = RectangularOrthogonal<space::Oklab>;

/// The model for a color specified in the oklab color space with the cylindrical polar form.
pub type Oklch = CylindricalPolar<space::Oklab>;
