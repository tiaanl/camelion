//! Gamut mapping functions.

use crate::{Color, Space};

#[allow(clippy::manual_range_contains)]
fn in_zero_to_one(value: f32) -> bool {
    value >= 0.0 && value <= 1.0
}

/// Calculate deltaE OK (simple root sum of squares).
/// <https://drafts.csswg.org/css-color-4/#color-difference-OK>
/**
 * @param {number[]} reference - Array of OKLab values: L as 0..1, a and b as -1..1
 * @param {number[]} sample - Array of OKLab values: L as 0..1, a and b as -1..1
 * @return {number} How different a color sample is from reference
 */
fn delta_eok(reference: Color, sample: Color) -> f32 {
    debug_assert_eq!(reference.space, Space::Oklab);
    debug_assert_eq!(sample.space, Space::Oklab);

    let delta_l = reference.components.0 - sample.components.0;
    let delta_a = reference.components.1 - sample.components.1;
    let delta_b = reference.components.2 - sample.components.2;

    (delta_l * delta_l + delta_a * delta_a + delta_b * delta_b).sqrt()
}

impl Color {
    /// If this color is not within gamut limits of it's color space, then a
    /// gamut mapping is applied to map the components into range.
    /// <https://drafts.csswg.org/css-color-4/#gamut-mapping>
    pub fn map_into_gamut_range(&self) -> Self {
        // 1. if destination has no gamut limits (XYZ-D65, XYZ-D50, Lab, LCH,
        //    Oklab, Oklch) return origin.
        if self.in_gamut() {
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
        if origin_oklch.components.0 <= 0.0 {
            return Color::new(self.space, 0.0, 0.0, 0.0, self.alpha);
        }

        // 5. let inGamut(color) be a function which returns true if, when
        //    passed a color, that color is inside the gamut of destination.
        //    For HSL and HWB, it returns true if the color is inside the gamut
        //    of sRGB.
        //    See [`Color::in_gamut`] below.

        // 6. if inGamut(origin_Oklch) is true, convert origin_Oklch to
        //    destination and return it as the gamut mapped color.
        if origin_oklch.in_gamut() {
            return self.clone();
        }

        // 7. otherwise, let delta(one, two) be a function which returns the
        //    deltaEOK of color one compared to color two.

        // 8. let JND be 0.02
        let jnd = 0.02;

        // 9. let epsilon be 0.0001
        let epsilon = 0.0001;

        // 10. let clip(color) be a function which converts color to
        //     destination, converts all negative components to zero, converts
        //     all components greater that one to one, and returns the result.
        //     See [`Color::clip`] below.

        // 11. set min to zero
        let mut min = 0.0;

        // 12. set max to the Oklch chroma of origin_Oklch.
        let max = origin_oklch.components.1;

        // 13. let min_inGamut be a boolean that represents when min is still
        //     in gamut, and set it to true
        let min_in_gamut = true;

        // 14. while (max - min is greater than epsilon) repeat the following
        //     steps.
        while max - min > epsilon {
            // 1. set chroma to (min + max) / 2
            let chroma = (min + max) / 2.0;

            // 2. set current to origin_Oklch and then set the chroma component
            //    to chroma
            let mut current = origin_oklch.clone();
            current.components.1 = chroma;

            // 3. if min_inGamut is true and also if inGamut(current) is true,
            //    set min to chroma and continue to repeat these steps.
            if min_in_gamut && current.in_gamut() {
                min = chroma;
                continue;
            }

            // 4. otherwise, if inGamut(current) is false carry out these
            //    steps:
        }

        todo!()
    }

    fn clip(&self) -> Color {
        Color::new(
            self.space,
            self.components.0.clamp(0.0, 1.0),
            self.components.1.clamp(0.0, 1.0),
            self.components.2.clamp(0.0, 1.0),
            self.alpha,
        )
    }

    fn in_gamut(&self) -> bool {
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
            | Space::XyzD65 => {
                // TODO: Should this be unreachable? Seems a bit unnescessary.
                unreachable!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Component;

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
    fn basic() {
        // sRGB white'ish with the red component slightly out of gamut range,
        // which is [0..1].
        let c = Color::new(Space::Srgb, 1.1, 1.0, 1.0, 1.0);
        assert_component_eq!(c.components.0, 0.9);
        assert_component_eq!(c.components.1, 0.9);
        assert_component_eq!(c.components.2, 0.9);
    }
}
