//! Gamut mapping functions.

use crate::{Color, Component, Space};

#[allow(clippy::manual_range_contains)]
fn in_zero_to_one(value: Component) -> bool {
    value >= 0.0 && value <= 1.0
}

/// Calculate deltaE OK (simple root sum of squares).
/// <https://drafts.csswg.org/css-color-4/#color-difference-OK>
fn delta_eok(reference: &Color, sample: &Color) -> Component {
    // Delta is calculated in the oklab color space.
    let reference = reference.to_space(Space::Oklab);
    let sample = sample.to_space(Space::Oklab);

    let d = sample.components - reference.components;
    (d.0 * d.0 + d.1 * d.1 + d.2 * d.2).sqrt()
}

impl Color {
    /// If this color is not within gamut limits of it's color space, then a
    /// gamut mapping is applied to map the components into range.
    /// <https://drafts.csswg.org/css-color-4/#gamut-mapping>
    pub fn map_into_gamut_range(&self) -> Self {
        // 1. if destination has no gamut limits (XYZ-D65, XYZ-D50, Lab, LCH,
        //    Oklab, Oklch) return origin.
        if matches!(
            self.space,
            Space::Lab | Space::Lch | Space::Oklab | Space::Oklch | Space::XyzD50 | Space::XyzD65
        ) {
            return self.clone();
        }

        // 2. let origin_Oklch be origin converted from origin color space to
        //    the Oklch color space.
        let origin_oklch = self.to_space(Space::Oklch);

        // 3. if the Lightness of origin_Oklch is greater than or equal to
        //    100%, return { 1 1 1 origin.alpha } in destination.
        if origin_oklch.components.0 >= 1.0 {
            return Color::new(self.space, 1.0, 1.0, 1.0, self.alpha);
        }
        // 4. if the Lightness of origin_Oklch is less than than or equal to
        //    0%, return { 0 0 0 origin.alpha } in destination.
        else if origin_oklch.components.0 <= 0.0 {
            return Color::new(self.space, 0.0, 0.0, 0.0, self.alpha);
        }

        // 5. let inGamut(color) be a function which returns true if, when
        //    passed a color, that color is inside the gamut of destination.
        //    For HSL and HWB, it returns true if the color is inside the gamut
        //    of sRGB.
        //    See [`Color::in_gamut`].

        // 6. if inGamut(origin_Oklch) is true, convert origin_Oklch to
        //    destination and return it as the gamut mapped color.
        if origin_oklch.to_space(self.space).in_gamut() {
            return self.clone();
        }

        // 7. otherwise, let delta(one, two) be a function which returns the
        //    deltaEOK of color one compared to color two.
        // See [`delta_eok`] function.

        // 8. let JND be 0.02
        const JND: Component = 0.02;

        // 9. let epsilon be 0.0001
        const EPSILON: Component = 1.0e-4;

        // 10. let clip(color) be a function which converts color to
        //     destination, converts all negative components to zero, converts
        //     all components greater that one to one, and returns the result.
        // See [`Color::clip`].

        // 11. set min to zero
        let mut min = 0.0;

        // 12. set max to the Oklch chroma of origin_Oklch.
        let mut max = origin_oklch.components.1;

        // 13. let min_inGamut be a boolean that represents when min is still
        //     in gamut, and set it to true
        let mut min_in_gamut = true;

        let mut current = origin_oklch.clone();
        let clipped = current.to_space(self.space).clip();
        if delta_eok(&current, &clipped) < JND {
            return clipped;
        }

        // 14. while (max - min is greater than epsilon) repeat the following
        //     steps.
        while max - min > EPSILON {
            // 14.1. set chroma to (min + max) / 2
            let chroma = (min + max) / 2.0;

            // 14.2. set current to origin_Oklch and then set the chroma
            //       component to chroma
            current.components.1 = chroma;

            let current_in_space = current.to_space(self.space);

            // 14.3. if min_inGamut is true and also if inGamut(current) is
            //       true, set min to chroma and continue to repeat these steps.
            if min_in_gamut && current_in_space.in_gamut() {
                min = chroma;
                continue;
            }

            // 14.4. otherwise, if inGamut(current) is false carry out these
            //       steps:

            // 14.4.1. set clipped to clip(current)
            let clipped = current_in_space.clip();

            // 14.4.2. set E to delta(clipped, current)
            let e = delta_eok(&clipped, &current);

            // 14.4.3. if E < JND
            if e < JND {
                // 14.4.3.1. if (JND - E < epsilon) return clipped as the gamut
                //           mapped color
                if JND - e < EPSILON {
                    return clipped;
                }

                // 14.4.3.2. otherwise

                // 14.4.3.2.1. set min_inGamut to false
                min_in_gamut = false;

                // 14.4.3.2.2. set min to chroma
                min = chroma;
            } else {
                // 14.4.4. otherwise, set max to chroma and continue to repeat
                //         these steps
                max = chroma;
            }
        }

        // 15. return current as the gamut mapped color current
        current.to_space(self.space)
    }

    /// Return a color with each of the components clipped (clamped to [0..1]).
    /// NOTE: This is a lossy operation.
    pub fn clip(&self) -> Color {
        Color::new(
            self.space,
            self.components.0.clamp(0.0, 1.0),
            self.components.1.clamp(0.0, 1.0),
            self.components.2.clamp(0.0, 1.0),
            self.alpha,
        )
    }

    /// Returns true if the color is within its gamut limits.
    ///
    /// Mainly for RGB based colors, checking components to be inside [0..1].
    /// `Hsl` and `Hwb` are converted to [`Space::Srgb`] before being checked.
    pub fn in_gamut(&self) -> bool {
        match self.space {
            Space::Srgb
            | Space::SrgbLinear
            | Space::DisplayP3
            | Space::A98Rgb
            | Space::ProPhotoRgb
            | Space::Rec2020 => {
                in_zero_to_one(self.components.0)
                    && in_zero_to_one(self.components.1)
                    && in_zero_to_one(self.components.2)
            }
            Space::Hsl | Space::Hwb => self.to_space(Space::Srgb).in_gamut(),
            Space::Lab
            | Space::Lch
            | Space::Oklab
            | Space::Oklch
            | Space::XyzD50
            | Space::XyzD65 => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_component_eq {
        ($actual:expr,$expected:expr) => {{
            assert!(
                ($actual - $expected).abs() <= 1.0e-6,
                "{} != {}",
                $actual,
                $expected
            )
        }};
    }

    #[test]
    fn basic() {
        // color(display-p3 1 1 0)
        let yellow = Color::new(Space::DisplayP3, 1.0, 1.0, 0.0, 1.0);
        let mapped = yellow.to_space(Space::Srgb).map_into_gamut_range();
        assert_eq!(mapped.space, Space::Srgb);
        assert_component_eq!(mapped.components.0, 0.9962327282577411);
        assert_component_eq!(mapped.components.1, 0.9990142856519192);
        assert_component_eq!(mapped.components.2, 0.0);
    }
}
