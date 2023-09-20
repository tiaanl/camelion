use num_traits::Float;

use crate::{Color, Component, Space};

fn lerp<T: Float>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

impl Color {
    /// Linearly interpolate from this color to another in the color space
    /// specified using `t` as the progress between them.
    pub fn interpolate(&self, other: &Self, t: Component, space: Space) -> Color {
        let left = self.to_space(space);
        let right = other.to_space(space);

        let color = Color::new(
            space,
            lerp(left.components.0, right.components.0, t),
            lerp(left.components.1, right.components.1, t),
            lerp(left.components.2, right.components.2, t),
            lerp(left.alpha, right.alpha, t),
        );

        color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let left = Color::new(Space::Srgb, 0.1, 0.2, 0.3, 1.0);
        let right = Color::new(Space::Srgb, 0.5, 0.6, 0.7, 1.0);
        let mixed = left.interpolate(&right, 0.5, Space::Srgb);
        assert_eq!(mixed.components.0, 0.3);
        assert_eq!(mixed.components.1, 0.4);
        assert_eq!(mixed.components.2, 0.5);
        assert_eq!(mixed.alpha, 1.0);
        assert_eq!(mixed.space, Space::Srgb);
    }
}
