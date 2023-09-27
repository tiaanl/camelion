//! Model a color in the CIE-XYZ color space.

use crate::{
    color::{Component, Components, HasSpace, Space},
    math::{transform, transform_3x3, Transform},
};

/// This trait is used for types that represent a CIE-XYZ white point
/// reference.
pub trait WhitePoint {
    /// The reference coordinates for the white point.
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

/// This trait is implemented on [`WhitePoint`]'s to transfer colors between
/// them.
pub trait TransferWhitePoint<T: WhitePoint>
where
    Self: WhitePoint + Sized,
{
    /// Transfer the white point reference to another.
    fn transfer(from: &Xyz<Self>) -> Xyz<T>;
}

impl TransferWhitePoint<D65> for D50 {
    /// Convert this model from CIE-XYZ with a D50 white point to a D65 white
    /// point.
    fn transfer(from: &Xyz<Self>) -> Xyz<D65> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const MAT: Transform = transform_3x3(
             0.9554734527042182,   -0.028369706963208136,  0.012314001688319899,
            -0.023098536874261423,  1.0099954580058226,   -0.020507696433477912,
             0.0632593086610217,    0.021041398966943008,  1.3303659366080753,
        );

        transform(&MAT, Components(from.x, from.y, from.z)).into()
    }
}

impl TransferWhitePoint<D50> for D65 {
    /// Convert this model from CIE-XYZ with a D65 white point to a D50 white
    /// point.
    fn transfer(from: &Xyz<Self>) -> Xyz<D50> {
        #[rustfmt::skip]
        #[allow(clippy::excessive_precision)]
        const MAT: Transform = transform_3x3(
             1.0479298208405488,    0.029627815688159344, -0.009243058152591178,
             0.022946793341019088,  0.990434484573249,     0.015055144896577895,
            -0.05019222954313557,  -0.01707382502938514,   0.7518742899580008,
        );

        transform(&MAT, Components(from.x, from.y, from.z)).into()
    }
}

/// Specify that a color model supports conversion to CIE-XYZ.
pub trait ToXyz<W: WhitePoint> {
    /// Convert this color to CIE-XYZ.
    fn to_xyz(&self) -> Xyz<W>;
}

camelion_macros::gen_model! {
    /// A model for a color in the CIE-XYZ color space with a specified white point reference.
    pub struct Xyz<W: WhitePoint> {
        /// The X component of the color.
        pub x: Component,
        /// The Y component of the color.
        pub y: Component,
        /// The Z component of the color.
        pub z: Component,
    }
}

impl<W: WhitePoint> Xyz<W> {
    /// Transfer the white point reference of this color to another.
    pub fn transfer<T: WhitePoint>(&self) -> Xyz<T>
    where
        W: TransferWhitePoint<T>,
    {
        W::transfer(self)
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
