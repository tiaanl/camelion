use crate::color::{Color, Component, Flags, Space};

impl Color {
    /// Create an interpolation that will interpolate from `self` to `other` using the specified [`Space`](color space).
    pub fn interpolate(&self, other: &Self, space: Space) -> Interpolation {
        Interpolation::new(self, other, space)
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
struct Premultiplied {
    components: [Option<Component>; 3],
    alpha: Option<Component>,
}

impl Premultiplied {
    /// Un-premultiply the values back into a color using the specified alpha
    /// value.
    /// <https://drafts.csswg.org/css-color-4/#interpolation-alpha>
    fn into_color(self, space: Space, alpha: Option<Component>) -> Color {
        let alpha = match alpha {
            Some(alpha) if alpha != 0.0 => alpha,
            _ => {
                return Color::new(
                    space,
                    self.components[0],
                    self.components[1],
                    self.components[2],
                    alpha,
                );
            }
        };

        let hue_index = space.hue_index();

        macro_rules! c {
            ($i:literal) => {{
                if hue_index == Some($i) {
                    self.components[$i]
                } else {
                    self.components[$i].map(|v| v / alpha)
                }
            }};
        }

        Color::new(space, c!(0), c!(1), c!(2), alpha)
    }
}

impl From<Color> for Premultiplied {
    /// Premultiply the color with it's alpha and return the result according
    /// to:
    /// <https://drafts.csswg.org/css-color-4/#interpolation-alpha>
    fn from(value: Color) -> Self {
        // If the alpha value is none, the premultiplied value is the
        // un-premultiplied value.
        if value.flags.contains(Flags::ALPHA_IS_NONE) {
            return Self {
                components: [value.c0(), value.c1(), value.c2()],
                alpha: None,
            };
        }

        let hue_index = value.space.hue_index();

        macro_rules! c {
            ($c:expr,$i:literal) => {{
                $c.map(|v| {
                    if hue_index != Some($i) {
                        v * value.alpha
                    } else {
                        v
                    }
                })
            }};
        }

        Self {
            components: [c!(value.c0(), 0), c!(value.c1(), 1), c!(value.c2(), 2)],
            alpha: value.alpha(),
        }
    }
}

/// Represents an interpolation between two colors using a specified color space.
#[derive(Clone)]
pub struct Interpolation {
    /// The color that will be interpolated from.
    left: Premultiplied,
    /// The color that will be interpolated to.
    right: Premultiplied,
    /// The color space/form used to interpolate between the two colors.
    pub space: Space,
    /// Which hue interpolation method to use when a hue component is present
    /// in the color space/from used for interpolation.
    pub hue_interpolation_method: HueInterpolationMethod,
}

impl Interpolation {
    /// Create a new interpolation with the given colors and color space.
    pub fn new(left: &Color, right: &Color, space: Space) -> Self {
        let mut left = left.to_space(space);
        let mut right = right.to_space(space);

        match (left.alpha(), right.alpha()) {
            (Some(left), None) => {
                right.flags.remove(Flags::ALPHA_IS_NONE);
                right.alpha = left;
            }
            (None, Some(right)) => {
                left.flags.remove(Flags::ALPHA_IS_NONE);
                left.alpha = right;
            }
            _ => {}
        }

        Self {
            left: left.into(),
            right: right.into(),
            space,
            hue_interpolation_method: Default::default(),
        }
    }

    /// Set the hue interpolation method on the [`Interpolation`].
    pub fn with_hue_interpolation(self, hue_interpolation_method: HueInterpolationMethod) -> Self {
        Self {
            hue_interpolation_method,
            ..self
        }
    }

    /// Calculate an interpolated color using weights for the left and right
    /// sides.
    pub fn with_weights(&self, left: Component, right: Component) -> Color {
        // <https://drafts.csswg.org/css-color-5/#color-mix-percent-norm>
        let (t, alpha_multiplier) = {
            let sum = left + right;
            if sum != 1.0 {
                let scale = 1.0 / sum;

                if sum < 1.0 {
                    (right * scale, sum)
                } else {
                    (right * scale, 1.0)
                }
            } else {
                (right, 1.0)
            }
        };

        let mut result = self.at(t);
        result.alpha *= alpha_multiplier;
        result
    }

    /// Calculate an interpolated color using a mid point specified by `t`.
    pub fn at(&self, t: Component) -> Color {
        // Interpolate the original alpha components.
        // TODO: This is essentially the same code used for each component,
        // can we somehow not duplicate it here.
        let alpha = match (self.left.alpha, self.right.alpha) {
            (None, None) => None,
            (Some(left), Some(right)) => Some(lerp(left, right, t)),
            (None, Some(_)) | (Some(_), None) => {
                // The alpha values were adjusted during premultiplication and
                // should either be both none or both some.
                unreachable!()
            }
        };

        // Interpolate the premultiplied components.
        let mut result = Premultiplied {
            components: [None, None, None],
            alpha: None,
        };
        result.components.iter_mut().enumerate().for_each(|(i, r)| {
            *r = match (self.left.components[i], self.right.components[i]) {
                (None, None) => None,
                (None, Some(right)) => Some(right),
                (Some(left), None) => Some(left),
                (Some(left), Some(right)) => Some(match self.space.hue_index() {
                    Some(index) if index == i => self.hue_interpolation_method.lerp(left, right, t),
                    _ => lerp(left, right, t),
                }),
            };
        });

        result.into_color(self.space, alpha)
    }
}

impl Space {
    /// Returns true if the color space uses red, green and blue components.
    fn _is_rgb_like(&self) -> bool {
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
    fn _is_xyz_like(&self) -> bool {
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

fn _analogous_missing_components(from: Space, to: Space, flags: Flags) -> Flags {
    if from == to {
        return flags;
    }

    // Reds             r, x
    // Greens           g, y
    // Blues            b, z
    if (from._is_rgb_like() || from._is_xyz_like()) && (from._is_rgb_like() || to._is_xyz_like()) {
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
            let result = _analogous_missing_components(from, to, flags);
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
        assert_component_eq!(shorter.at(0.0).components.0, 50.0);
        assert_component_eq!(shorter.at(0.25).components.0, 30.0);
        assert_component_eq!(shorter.at(0.5).components.0, 10.0);
        assert_component_eq!(shorter.at(0.75).components.0, 350.0);
        assert_component_eq!(shorter.at(1.0).components.0, 330.0);

        let longer = interp.clone().with_hue_interpolation(H::Longer);
        assert_component_eq!(longer.at(0.5).components.0, 190.0);

        let increasing = interp.clone().with_hue_interpolation(H::Increasing);
        assert_component_eq!(increasing.at(0.5).components.0, 190.0);

        let decreasing = interp.clone().with_hue_interpolation(H::Decreasing);
        assert_component_eq!(decreasing.at(0.5).components.0, 10.0);
    }

    #[test]
    fn test_premultiplied() {
        // rgb(24% 12% 98% / 0.4) => [9.6% 4.8% 39.2%]
        let left = Premultiplied::from(Color::new(Space::Srgb, 0.24, 0.12, 0.98, 0.4));
        assert_component_eq!(left.components[0].unwrap(), 0.096);
        assert_component_eq!(left.components[1].unwrap(), 0.048);
        assert_component_eq!(left.components[2].unwrap(), 0.392);
        assert_component_eq!(left.alpha.unwrap(), 0.4);

        // rgb(62% 26% 64% / 0.6) => [37.2% 15.6% 38.4%]
        let right = Premultiplied::from(Color::new(Space::Srgb, 0.62, 0.26, 0.64, 0.6));
        assert_component_eq!(right.components[0].unwrap(), 0.372);
        assert_component_eq!(right.components[1].unwrap(), 0.156);
        assert_component_eq!(right.components[2].unwrap(), 0.384);
        assert_component_eq!(right.alpha.unwrap(), 0.6);
    }

    #[test]
    fn interpolate_with_alpha() {
        let left = Color::new(Space::Srgb, 0.24, 0.12, 0.98, 0.4);
        let right = Color::new(Space::Srgb, 0.62, 0.26, 0.64, 0.6);

        let middle = left.interpolate(&right, Space::Srgb).at(0.5);

        assert_component_eq!(middle.components.0, 0.468);
        assert_component_eq!(middle.components.1, 0.204);
        assert_component_eq!(middle.components.2, 0.776);
        assert_component_eq!(middle.alpha, 0.5);
    }

    #[test]
    fn ad_hoc() {
        // // color-mix(in xyz-d65, color(xyz-d65 .1 .2 .3 / none), color(xyz-d65 .5 .6 .7 / none))
        // let left = Color::new(Space::XyzD65, 0.1, 0.2, 0.3, None);
        // let right = Color::new(Space::XyzD65, 0.5, 0.6, 0.7, None);
        // let middle = left.interpolate(&right, Space::XyzD65).at(0.5);
        // // color(xyz-d65 0.3 0.4 0.5 / none)
        // assert_component_eq!(middle.components.0, 0.3);
        // assert_component_eq!(middle.components.1, 0.4);
        // assert_component_eq!(middle.components.2, 0.5);
        // assert_eq!(middle.flags, Flags::ALPHA_IS_NONE);

        // // color-mix(in hsl, hsl(120deg 10% 20%) 12.5%, hsl(30deg 30% 40%) 37.5%)
        // let left = Color::new(Space::XyzD65, 0.1, 0.2, 0.3, None);
        // let right = Color::new(Space::XyzD65, 0.5, 0.6, 0.7, None);
        // let middle = left.interpolate(&right, Space::XyzD65).at(0.5);
        // // color(srgb 0.4375 0.415625 0.2625 / 0.5)
        // assert_component_eq!(middle.components.0, 0.3);
        // assert_component_eq!(middle.components.1, 0.4);
        // assert_component_eq!(middle.components.2, 0.5);
        // assert_eq!(middle.flags, Flags::ALPHA_IS_NONE);

        // // color-mix(in hsl, hsl(120deg 10% 20% / .4), hsl(30deg 30% 40% / .8))
        // let left = Color::new(Space::Hsl, 120.0, 0.1, 0.2, 0.4);
        // let right = Color::new(Space::Hsl, 30.0, 0.3, 0.4, 0.8);
        // let middle = left
        //     .interpolate(&right, Space::Hsl)
        //     .at(0.5)
        //     .to_space(Space::Srgb);
        // // color(srgb 0.372222 0.411111 0.255556 / 0.6)
        // assert_component_eq!(middle.components.0, 0.372222);
        // assert_component_eq!(middle.components.1, 0.411111);
        // assert_component_eq!(middle.components.2, 0.255556);
        // assert_component_eq!(middle.alpha, 0.6);

        // // color-mix(in hsl longer hue, hsl(40deg 50% 50%), hsl(60deg 50% 50%))
        // let left = Color::new(Space::Hsl, 40.0, 0.5, 0.5, 1.0);
        // let right = Color::new(Space::Hsl, 60.0, 0.5, 0.5, 1.0);
        // let middle = left
        //     .interpolate(&right, Space::Hsl)
        //     .with_hue_interpolation(HueInterpolationMethod::Longer)
        //     .at(0.5)
        //     .to_space(Space::Srgb);
        // // color(srgb 0.25 0.333333 0.75)
        // assert_component_eq!(middle.components.0, 0.25);
        // assert_component_eq!(middle.components.1, 0.333333);
        // assert_component_eq!(middle.components.2, 0.75);
        // assert_component_eq!(middle.alpha, 1.0);

        // // color-mix(in srgb, color(srgb .1 .2 .3 / none), color(srgb .5 .6 .7))
        // let left = Color::new(Space::Srgb, 0.1, 0.2, 0.3, None);
        // let right = Color::new(Space::Srgb, 0.5, 0.6, 0.7, 1.0);
        // let interp = Interpolation::new(&left, &right, Space::Srgb);
        // // color(${resultColorSpace} 0.3 0.4 0.5)
        // let result = interp.at(0.5);
        // assert_component_eq!(result.components.0, 0.3);
        // assert_component_eq!(result.components.1, 0.4);
        // assert_component_eq!(result.components.2, 0.5);
        // assert_component_eq!(result.alpha, 1.0);

        // // color-mix(in srgb, color(srgb .1 .2 .3 / none), color(srgb .5 .6 .7 / 0.5))
        // let left = Color::new(Space::Srgb, 0.1, 0.2, 0.3, None);
        // let right = Color::new(Space::Srgb, 0.5, 0.6, 0.7, 0.5);
        // let interp = Interpolation::new(&left, &right, Space::Srgb);
        // // color(srgb 0.3 0.4 0.5 / 0.5)
        // let result = interp.at(0.5);
        // assert_component_eq!(result.components.0, 0.3);
        // assert_component_eq!(result.components.1, 0.4);
        // assert_component_eq!(result.components.2, 0.5);
        // assert_component_eq!(result.alpha, 0.5);

        // color-mix(in hsl, transparent, hsl(30deg 30% 40%))
        let left = Color::new(Space::Srgb, 0.0, 0.0, 0.0, 0.0);
        let right = Color::new(Space::Hsl, 30.0, 0.3, 0.4, 1.0);
        let interp = Interpolation::new(&left, &right, Space::Hsl);
        // color(srgb 0.52 0.4 0.28 / 0.5)
        let result = interp.at(0.5).to_space(Space::Srgb);
        assert_component_eq!(result.components.0, 0.52);
        assert_component_eq!(result.components.1, 0.4);
        assert_component_eq!(result.components.2, 0.28);
        assert_component_eq!(result.alpha, 0.5);

        // // color-mix(in hsl, hsl(none none none), hsl(none none none))
        // let left = Color::new(Space::Hsl, None, None, None, 1.0);
        // let right = Color::new(Space::Hsl, None, None, None, 1.0);
        // let interp = Interpolation::new(&left, &right, Space::Hsl);
        // let result = interp.at(0.5);
        // assert!(result.c0().is_none());
        // assert!(result.c1().is_none());
        // assert!(result.c2().is_none());

        // let srgb = result.to_space(Space::Srgb);
        // assert!(srgb.c0().is_none());
        // assert!(srgb.c1().is_none());
        // assert!(srgb.c2().is_none());
    }
}
