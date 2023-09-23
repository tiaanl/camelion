use num_traits::Float;

use crate::{Color, Component, Components, Flags, Space};

impl Color {
    /// Premultiply
    pub fn premultiply(&self) -> (Self, Component) {
        let alpha = self.alpha;
        (
            Self {
                components: self.components.map(|c| c * self.alpha),
                alpha: 1.0,
                flags: self.flags,
                space: self.space,
            },
            alpha,
        )
    }

    /// Linearly interpolate from this color to another in the color space
    /// specified using `t` as the progress between them.
    pub fn interpolate(&self, other: &Self, space: Space) -> Interpolation {
        let left = self.to_space(space);
        let right = other.to_space(space);

        Interpolation {
            left,
            right,
            space,
            _typed: (),
        }
    }
}

fn lerp<T: Float>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

pub struct Interpolation<Typed = ()> {
    left: Color,
    right: Color,
    space: Space,

    _typed: Typed,
}

pub struct Premultiplied {
    _left_alpha: Component,
    _right_alpha: Component,
}

impl Interpolation<()> {
    pub fn premultiply(&self) -> Interpolation<Premultiplied> {
        let (left, left_alpha) = self.left.premultiply();
        let (right, right_alpha) = self.right.premultiply();
        Interpolation {
            left,
            right,
            space: self.space,
            _typed: Premultiplied {
                _left_alpha: left_alpha,
                _right_alpha: right_alpha,
            },
        }
    }
}

impl<Typed> Interpolation<Typed> {
    pub fn at(&self, t: Component) -> Color {
        Color {
            components: Components(
                lerp(self.left.components.0, self.right.components.0, t),
                lerp(self.left.components.1, self.right.components.1, t),
                lerp(self.left.components.2, self.right.components.2, t),
            ),
            alpha: lerp(self.left.alpha, self.right.alpha, t),
            flags: Flags::empty(),
            space: self.space,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_component_eq {
        ($actual:expr,$expected:expr) => {{
            assert!(($actual - $expected).abs() <= Component::EPSILON * 1e3)
        }};
    }

    #[test]
    fn basic() {
        let left = Color::new(Space::Srgb, 0.1, 0.2, 0.3, 1.0);
        let right = Color::new(Space::Srgb, 0.5, 0.6, 0.7, 1.0);
        let mixed = left.interpolate(&right, Space::Srgb).at(0.5);
        assert_component_eq!(mixed.components.0, 0.3);
        assert_component_eq!(mixed.components.1, 0.4);
        assert_component_eq!(mixed.components.2, 0.5);
        assert_component_eq!(mixed.alpha, 1.0);
        assert_eq!(mixed.space, Space::Srgb);

        let interp = left.interpolate(&right, Space::Srgb);
        let middle = interp.at(0.5);
        assert_component_eq!(middle.components.0, 0.3);
        assert_component_eq!(middle.components.1, 0.4);
        assert_component_eq!(middle.components.2, 0.5);
        assert_component_eq!(middle.alpha, 1.0);
        assert_eq!(middle.space, Space::Srgb);
    }
}
