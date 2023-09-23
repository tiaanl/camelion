//! Math utility functions.

use std::marker::PhantomData;

use euclid::default::{Transform3D, Vector3D};

pub type Transform = Transform3D<Component>;

type Vector = Vector3D<Component>;

use crate::Component;

/// Multiply the given matrix in `transform` with the 3 components.
pub fn transform(
    transform: &Transform,
    x: Component,
    y: Component,
    z: Component,
) -> [Component; 3] {
    let Vector { x, y, z, .. } = transform.transform_vector3d(Vector::new(x, y, z));
    [x, y, z]
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
