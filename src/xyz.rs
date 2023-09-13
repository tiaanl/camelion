//! Model a color in the CIE-XYZ color space.

use crate::color::ComponentDetails;
use crate::{Component, Flags, Space};
use std::marker::PhantomData;

mod white_point {
    use crate::{Components, Space};

    pub trait WhitePoint {
        const WHITE_POINT: Components;
        const SPACE: Space;
    }

    pub struct D50;
    impl WhitePoint for D50 {
        const WHITE_POINT: Components = Components(0.9642956764295677, 1.0, 0.8251046025104602);
        const SPACE: Space = Space::XyzD50;
    }

    pub struct D65;
    impl WhitePoint for D65 {
        const WHITE_POINT: Components = Components(0.9504559270516716, 1.0, 1.0890577507598784);
        const SPACE: Space = Space::XyzD65;
    }
}

pub struct Xyz<W: white_point::WhitePoint> {
    pub x: Component,
    pub y: Component,
    pub z: Component,
    pub alpha: Component,
    pub flags: Flags,
    _space: Space,

    white_point: PhantomData<W>,
}

impl<W: white_point::WhitePoint> Xyz<W> {
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
            _space: W::SPACE,
            white_point: PhantomData,
        }
    }
}

/// Model for a color in the CIE-XYZ color space with a D50 white point.
pub type XyzD50 = Xyz<white_point::D50>;

/// Model for a color in the CIE-XYZ color space with a D65 white point.
pub type XyzD65 = Xyz<white_point::D65>;
