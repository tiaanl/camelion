//! Model a color in the CIE-XYZ color space.

use crate::color::{ComponentDetails, HasSpace, SpacePlaceholder};
use crate::{Color, Component, Components, Flags, Space};
use std::marker::PhantomData;

pub trait WhitePoint {
    const WHITE_POINT: Components;
}

/// CIE-XYZ color with a D50 white point reference.
#[derive(Clone, Debug)]
pub struct D50;

impl WhitePoint for D50 {
    #[allow(clippy::excessive_precision)]
    const WHITE_POINT: Components = Components(0.9642956764295677, 1.0, 0.8251046025104602);
}

/// CIE-XYZ color with a D65 white point reference.
#[derive(Clone, Debug)]
pub struct D65;

impl WhitePoint for D65 {
    #[allow(clippy::excessive_precision)]
    const WHITE_POINT: Components = Components(0.9504559270516716, 1.0, 1.0890577507598784);
}

/// Specify that a color model supports conversion to CIE-XYZ.
pub trait ToXyz<W: WhitePoint> {
    /// Convert this color to CIE-XYZ.
    fn to_xyz(&self) -> Xyz<W>;
}

#[derive(Clone, Debug)]
pub struct Xyz<W: WhitePoint> {
    pub x: Component,
    pub y: Component,
    pub z: Component,
    pub alpha: Component,
    pub flags: Flags,

    _space: SpacePlaceholder,
    _w: PhantomData<W>,
}

impl<W: WhitePoint> Xyz<W> {
    pub fn new(
        x: impl Into<ComponentDetails>,
        y: impl Into<ComponentDetails>,
        z: impl Into<ComponentDetails>,
        alpha: impl Into<ComponentDetails>,
    ) -> Self {
        let mut flags = Flags::empty();

        let x = x.into().value_and_flag(&mut flags, Flags::C0_IS_NONE);
        let y = y.into().value_and_flag(&mut flags, Flags::C1_IS_NONE);
        let z = z.into().value_and_flag(&mut flags, Flags::C2_IS_NONE);
        let alpha = alpha
            .into()
            .value_and_flag(&mut flags, Flags::ALPHA_IS_NONE);

        Self {
            x,
            y,
            z,
            alpha,
            flags,
            _space: 0,
            _w: PhantomData,
        }
    }
}

/// Model for a color in the CIE-XYZ color space with a D50 white point.
pub type XyzD50 = Xyz<D50>;

impl HasSpace for XyzD50 {
    const SPACE: Space = Space::XyzD50;
}

/// Model for a color in the CIE-XYZ color space with a D65 white point.
pub type XyzD65 = Xyz<D65>;

impl HasSpace for XyzD65 {
    const SPACE: Space = Space::XyzD65;
}

impl<W: WhitePoint> From<Xyz<W>> for Color
where
    Xyz<W>: HasSpace,
{
    fn from(value: Xyz<W>) -> Self {
        Self {
            components: Components(value.x, value.y, value.z),
            alpha: value.alpha,
            flags: value.flags,
            space: <Xyz<W> as HasSpace>::SPACE,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Color, Space, XyzD50};

    #[test]
    fn as_model() {
        let color = Color::new(Space::XyzD50, 0.1, 0.2, 0.3, 0.4);

        let model = color.as_model::<XyzD50>().clone();
        assert_eq!(model.x, color.components.0);
        assert_eq!(model.y, color.components.1);
        assert_eq!(model.z, color.components.2);
        assert_eq!(model.alpha, color.alpha);
        assert_eq!(model.flags, color.flags);

        let back = Color::from(model);
        assert_eq!(back.components.0, color.components.0);
        assert_eq!(back.components.1, color.components.1);
        assert_eq!(back.components.2, color.components.2);
        assert_eq!(back.alpha, color.alpha);
        assert_eq!(back.flags, color.flags);
        assert_eq!(back.space, Space::XyzD50);
    }
}
