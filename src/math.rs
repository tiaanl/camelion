//! Math utility functions.

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
