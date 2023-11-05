//! Math utility functions.

use crate::color::{Component, Components};
use euclid::default::{Transform3D, Vector3D};
use std::marker::PhantomData;

/// Normalize a floating point value to 0.0 if it is NaN.
#[inline]
pub fn normalize(v: Component) -> Component {
    if v.is_nan() {
        0.0
    } else {
        v
    }
}

/// Normalize a hue component to within [0..360] and ensure it is not NAN.
#[inline]
pub fn normalize_hue(hue: Component) -> Component {
    normalize(hue).rem_euclid(360.0)
}

/// Returns true if the value is very close to zero.
#[inline]
pub fn almost_zero(v: Component) -> bool {
    // TODO(tlouw): This could be re-evaluated at some point to allow for a
    //              more lenient threshold.
    v.abs() < Component::EPSILON
}

pub type Transform = Transform3D<Component>;

type Vector = Vector3D<Component>;

/// Multiply the given matrix in `transform` with the 3 components.
pub fn transform(transform: &Transform, components: Components) -> Components {
    let Vector { x, y, z, .. } =
        transform.transform_vector3d(Vector::new(components.0, components.1, components.2));
    Components(x, y, z)
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
pub const fn transform_3x3(
    m11: Component,
    m12: Component,
    m13: Component,
    m21: Component,
    m22: Component,
    m23: Component,
    m31: Component,
    m32: Component,
    m33: Component,
) -> Transform {
    Transform {
        m11,
        m12,
        m13,
        m14: 0.0,
        m21,
        m22,
        m23,
        m24: 0.0,
        m31,
        m32,
        m33,
        m34: 0.0,
        m41: 0.0,
        m42: 0.0,
        m43: 0.0,
        m44: 1.0,
        _unit: PhantomData,
    }
}
