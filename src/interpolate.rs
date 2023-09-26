use crate::{Color, Component, Flags, Space};

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
            hue_interpolation_method: Default::default(),
        }
    }
}

fn lerp(a: Component, b: Component, t: Component) -> Component {
    a + t * (b - a)
}

/// The method used for interpolating hue components.
/// <https://drafts.csswg.org/css-color-4/#hue-interpolation>
#[derive(Clone, Copy, Default)]
pub enum HueInterpolationMethod {
    /// Hue angles are interpolated to take the shorter of the two arcs between
    /// the starting and ending hues.
    /// <https://drafts.csswg.org/css-color-4/#hue-shorter>
    #[default]
    Shorter,
    /// Hue angles are interpolated to take the longer of the two arcs between
    /// the starting and ending hues.
    /// <https://drafts.csswg.org/css-color-4/#hue-longer>
    Longer,
    /// Hue angles are interpolated so that, as they progress from the first
    /// color to the second, the angle is always increasing.
    /// <https://drafts.csswg.org/css-color-4/#hue-increasing>
    Increasing,
    /// Hue angles are interpolated so that, as they progress from the first
    /// color to the second, the angle is always decreasing.
    /// <https://drafts.csswg.org/css-color-4/#hue-decreasing>
    Decreasing,
}

impl HueInterpolationMethod {
    fn adjust_hue(&self, a: &mut Component, b: &mut Component) {
        debug_assert!(!a.is_nan());
        debug_assert!(!b.is_nan());

        *a = a.rem_euclid(360.0);
        *b = b.rem_euclid(360.0);

        match self {
            HueInterpolationMethod::Shorter => {
                let delta = *b - *a;

                if delta > 180.0 {
                    *a += 360.0;
                } else if delta < -180.0 {
                    *b += 360.0;
                }
            }
            HueInterpolationMethod::Longer => {
                let delta = *b - *a;
                if 0.0 < delta && delta < 180.0 {
                    *a += 360.0;
                } else if -180.0 < delta && delta <= 0.0 {
                    *b += 360.0;
                }
            }
            HueInterpolationMethod::Increasing => {
                if *b < *a {
                    *b += 360.0;
                }
            }
            HueInterpolationMethod::Decreasing => {
                if *a < *b {
                    *a += 360.0;
                }
            }
        }
    }

    fn lerp(&self, a: Component, b: Component, t: Component) -> Component {
        let (mut a, mut b) = (a, b);
        self.adjust_hue(&mut a, &mut b);
        lerp(a, b, t).rem_euclid(360.0)
    }
}

#[derive(Clone)]
pub struct Interpolation {
    left: Color,
    right: Color,
    space: Space,
    hue_interpolation_method: HueInterpolationMethod,
}

impl Interpolation {
    pub fn with_hue_interpolation(self, hue_interpolation_method: HueInterpolationMethod) -> Self {
        Self {
            hue_interpolation_method,
            ..self
        }
    }

    pub fn at(&self, t: Component) -> Color {
        macro_rules! component {
            ($color:expr,$i:tt,$flag:ident) => {{
                if $color.flags.contains(Flags::$flag) {
                    None
                } else {
                    Some($color.components.$i)
                }
            }};
        }

        let left_flags = analogous_missing_components(self.left.space, self.space, self.left.flags);
        let mut left = self.left.to_space(self.space);
        left.flags = left_flags;

        let left = [
            component!(left, 0, C0_IS_NONE),
            component!(left, 1, C1_IS_NONE),
            component!(left, 2, C2_IS_NONE),
        ];

        let right_flags =
            analogous_missing_components(self.right.space, self.space, self.right.flags);
        let mut right = self.right.to_space(self.space);
        right.flags = right_flags;

        let right = [
            component!(right, 0, C0_IS_NONE),
            component!(right, 1, C1_IS_NONE),
            component!(right, 2, C2_IS_NONE),
        ];

        let mut result: [Option<Component>; 4] = [
            None,
            None,
            None,
            Some(lerp(self.left.alpha, self.right.alpha, t)),
        ];

        // Interpolate each component.
        for i in 0..=2 {
            result[i] = match (left[i], right[i]) {
                (None, None) => None,
                (None, Some(right)) => Some(right),
                (Some(left), None) => Some(left),
                (Some(left), Some(right)) => Some(match self.space.hue_index() {
                    Some(index) if index == i => self.hue_interpolation_method.lerp(left, right, t),
                    _ => lerp(left, right, t),
                }),
            };
        }

        Color::new(self.space, result[0], result[1], result[2], result[3])
    }
}

impl Space {
    /// Returns true if the color space uses red, green and blue components.
    fn is_rgb_like(&self) -> bool {
        match self {
            Space::Srgb
            | Space::SrgbLinear
            | Space::DisplayP3
            | Space::A98Rgb
            | Space::ProPhotoRgb
            | Space::Rec2020 => true,
            Space::Hsl
            | Space::Hwb
            | Space::Lab
            | Space::Lch
            | Space::Oklab
            | Space::Oklch
            | Space::XyzD50
            | Space::XyzD65 => false,
        }
    }

    /// Returns true if the color space uses X, Y and Z components. Typically
    /// used by the CIE-XYZ color space.
    fn is_xyz_like(&self) -> bool {
        match self {
            Space::XyzD50 | Space::XyzD65 => true,
            Space::Srgb
            | Space::SrgbLinear
            | Space::Hsl
            | Space::Hwb
            | Space::Lab
            | Space::Lch
            | Space::Oklab
            | Space::Oklch
            | Space::DisplayP3
            | Space::A98Rgb
            | Space::ProPhotoRgb
            | Space::Rec2020 => false,
        }
    }

    /// Returns the index of a hue component, otherwise None if the color does
    /// not have a hue component.
    fn hue_index(&self) -> Option<usize> {
        match self {
            Space::Hsl => Some(0),
            Space::Hwb => Some(0),
            Space::Lch => Some(2),
            Space::Oklch => Some(2),
            Space::Srgb
            | Space::SrgbLinear
            | Space::Lab
            | Space::Oklab
            | Space::XyzD50
            | Space::XyzD65
            | Space::DisplayP3
            | Space::A98Rgb
            | Space::ProPhotoRgb
            | Space::Rec2020 => None,
        }
    }
}

fn analogous_missing_components(from: Space, to: Space, flags: Flags) -> Flags {
    if from == to {
        return flags;
    }

    // Reds             r, x
    // Greens           g, y
    // Blues            b, z
    if (from.is_rgb_like() || from.is_xyz_like()) && (from.is_rgb_like() || to.is_xyz_like()) {
        return flags;
    }

    let mut result = Flags::empty();

    // Lightness        L
    if matches!(from, Space::Lab | Space::Lch | Space::Oklab | Space::Oklch) {
        if matches!(to, Space::Lab | Space::Lch | Space::Oklab | Space::Oklch) {
            result.set(Flags::C0_IS_NONE, flags.contains(Flags::C0_IS_NONE));
        } else if matches!(to, Space::Hsl) {
            result.set(Flags::C2_IS_NONE, flags.contains(Flags::C0_IS_NONE));
        }
    } else if matches!(from, Space::Hsl)
        && matches!(to, Space::Lab | Space::Lch | Space::Oklab | Space::Oklch)
    {
        result.set(Flags::C0_IS_NONE, flags.contains(Flags::C2_IS_NONE));
    }

    // Colorfulness     C, S
    if matches!(from, Space::Hsl | Space::Lch | Space::Oklch)
        && matches!(to, Space::Hsl | Space::Lch | Space::Oklch)
    {
        result.set(Flags::C1_IS_NONE, flags.contains(Flags::C1_IS_NONE));
    }

    // Hue              H
    if matches!(from, Space::Hsl | Space::Hwb) {
        if matches!(to, Space::Hsl | Space::Hwb) {
            result.set(Flags::C0_IS_NONE, flags.contains(Flags::C0_IS_NONE));
        } else if matches!(to, Space::Lch | Space::Oklch) {
            result.set(Flags::C2_IS_NONE, flags.contains(Flags::C0_IS_NONE));
        }
    } else if matches!(from, Space::Lch | Space::Oklch) {
        if matches!(to, Space::Hsl | Space::Hwb) {
            result.set(Flags::C0_IS_NONE, flags.contains(Flags::C2_IS_NONE));
        } else if matches!(to, Space::Lch | Space::Oklch) {
            result.set(Flags::C2_IS_NONE, flags.contains(Flags::C2_IS_NONE));
        }
    }

    // Opponent         a, a
    // Opponent         b, b
    if matches!(from, Space::Lab | Space::Oklab) && matches!(to, Space::Lab | Space::Oklab) {
        result.set(Flags::C1_IS_NONE, flags.contains(Flags::C1_IS_NONE));
        result.set(Flags::C2_IS_NONE, flags.contains(Flags::C2_IS_NONE));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_component_eq {
        ($actual:expr,$expected:expr) => {{
            assert!(
                ($actual - $expected).abs() <= Component::EPSILON * 1e3,
                "{} != {}",
                $actual,
                $expected
            )
        }};
    }

    #[test]
    fn test_analogous_missing_components() {
        use Flags as F;
        use Space as S;

        #[rustfmt::skip]
        let tests = [
            // Reds             r, x
            // Greens           g, y
            // Blues            b, z
            (S::Srgb, S::DisplayP3, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Srgb, S::DisplayP3, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Srgb, S::DisplayP3, F::C2_IS_NONE, F::C2_IS_NONE),
            (S::Srgb, S::DisplayP3, F::ALPHA_IS_NONE, F::ALPHA_IS_NONE),
            (S::Srgb, S::DisplayP3, F::C0_IS_NONE | F::C1_IS_NONE | F::C2_IS_NONE | F::ALPHA_IS_NONE,
                                    F::C0_IS_NONE | F::C1_IS_NONE | F::C2_IS_NONE | F::ALPHA_IS_NONE),

            // Lightness        L
            (S::Lab, S::Lab, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Lab, S::Lch, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Lch, S::Lch, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Lch, S::Lab, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Lab, S::Hsl, F::C0_IS_NONE, F::C2_IS_NONE),
            (S::Lch, S::Hsl, F::C0_IS_NONE, F::C2_IS_NONE),
            (S::Hsl, S::Lab, F::C2_IS_NONE, F::C0_IS_NONE),
            (S::Hsl, S::Lch, F::C2_IS_NONE, F::C0_IS_NONE),
            (S::Oklab, S::Oklab, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Oklab, S::Oklch, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Oklch, S::Oklch, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Oklch, S::Oklab, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Oklab, S::Hsl, F::C0_IS_NONE, F::C2_IS_NONE),
            (S::Oklch, S::Hsl, F::C0_IS_NONE, F::C2_IS_NONE),
            (S::Hsl, S::Oklab, F::C2_IS_NONE, F::C0_IS_NONE),
            (S::Hsl, S::Oklch, F::C2_IS_NONE, F::C0_IS_NONE),

            // Colorfulness     C, S
            (S::Hsl, S::Hsl, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Hsl, S::Lch, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Hsl, S::Oklch, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Lch, S::Hsl, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Lch, S::Lch, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Lch, S::Oklch, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Oklch, S::Hsl, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Oklch, S::Lch, F::C1_IS_NONE, F::C1_IS_NONE),
            (S::Oklch, S::Oklch, F::C1_IS_NONE, F::C1_IS_NONE),

            // Hue              H
            (S::Hsl, S::Hsl, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Hsl, S::Hwb, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Hsl, S::Lch, F::C0_IS_NONE, F::C2_IS_NONE),
            (S::Hsl, S::Oklch, F::C0_IS_NONE, F::C2_IS_NONE),
            (S::Hwb, S::Hsl, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Hwb, S::Hwb, F::C0_IS_NONE, F::C0_IS_NONE),
            (S::Hwb, S::Lch, F::C0_IS_NONE, F::C2_IS_NONE),
            (S::Hwb, S::Oklch, F::C0_IS_NONE, F::C2_IS_NONE),
            (S::Lch, S::Hsl, F::C2_IS_NONE, F::C0_IS_NONE),
            (S::Lch, S::Hwb, F::C2_IS_NONE, F::C0_IS_NONE),
            (S::Lch, S::Lch, F::C2_IS_NONE, F::C2_IS_NONE),
            (S::Lch, S::Oklch, F::C2_IS_NONE, F::C2_IS_NONE),
            (S::Oklch, S::Hsl, F::C2_IS_NONE, F::C0_IS_NONE),
            (S::Oklch, S::Hwb, F::C2_IS_NONE, F::C0_IS_NONE),
            (S::Oklch, S::Lch, F::C2_IS_NONE, F::C2_IS_NONE),
            (S::Oklch, S::Oklch, F::C2_IS_NONE, F::C2_IS_NONE),

            // Opponent         a, a
            // Opponent         b, b
            (S::Lab, S::Lab, F::C1_IS_NONE | F::C2_IS_NONE, F::C1_IS_NONE | F::C2_IS_NONE),
            (S::Lab, S::Oklab, F::C1_IS_NONE | F::C2_IS_NONE, F::C1_IS_NONE | F::C2_IS_NONE),
            (S::Oklab, S::Lab, F::C1_IS_NONE | F::C2_IS_NONE, F::C1_IS_NONE | F::C2_IS_NONE),
            (S::Oklab, S::Oklab, F::C1_IS_NONE | F::C2_IS_NONE, F::C1_IS_NONE | F::C2_IS_NONE),
        ];

        for (from, to, flags, expected) in tests {
            let result = analogous_missing_components(from, to, flags);
            assert_eq!(
                result, expected,
                "{:?} to {:?}, {:?} != {:?}",
                from, to, result, expected
            );
        }
    }

    #[test]
    fn linear_components() {
        let left = Color::new(Space::Srgb, 0.1, 0.2, 0.3, 1.0);
        let right = Color::new(Space::Srgb, 0.5, 0.6, 0.7, 1.0);

        let result = left.interpolate(&right, Space::Srgb).at(0.75);
        assert_eq!(result.components.0, 0.4);
        assert_eq!(result.components.1, 0.5);
        assert_eq!(result.components.2, 0.6);
    }

    #[test]
    fn hue_components() {
        use HueInterpolationMethod as H;

        let left = Color::new(Space::Hsl, 50.0, 0.3, 0.7, 1.0);
        let right = Color::new(Space::Hsl, -30.0, 0.7, 0.3, 1.0);
        let interp = left.interpolate(&right, Space::Hsl);

        let shorter = interp.clone().with_hue_interpolation(H::Shorter);
        assert_component_eq!(shorter.at(0.5).components.0, 10.0);

        let longer = interp.clone().with_hue_interpolation(H::Longer);
        assert_component_eq!(longer.at(0.5).components.0, 190.0);

        let increasing = interp.clone().with_hue_interpolation(H::Increasing);
        assert_component_eq!(increasing.at(0.5).components.0, 190.0);

        let decreasing = interp.clone().with_hue_interpolation(H::Decreasing);
        assert_component_eq!(decreasing.at(0.5).components.0, 10.0);
    }
}
