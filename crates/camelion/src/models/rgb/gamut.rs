//! Gamut mapping

use super::encoding::{GammaEncoded, LinearLight};
use super::HasGammaEncoding;
use super::{encoding::GammaEncoding, Rgb};
use crate::color_space::{self, ColorSpace};
use crate::models::{Base, Oklab, Polar, Rectangular, ToBase, ToXyz, TransferWhitePoint, D65};
use crate::Component;

type WhitePointFor<S> = <Rgb<S, LinearLight> as ToXyz>::WhitePoint;

impl<S, E> Rgb<S, E>
where
    S: ColorSpace + HasGammaEncoding,
    E: GammaEncoding,
    Self: Clone + From<Oklab> + ToBase,
    Rgb<S, LinearLight>: ToXyz,
    WhitePointFor<S>: TransferWhitePoint<D65>,
    D65: TransferWhitePoint<WhitePointFor<S>>,
    Oklab: From<Self>,
{
    /// Map the color into the gamut limits of the color space.
    pub fn map_into_gamut_limit(&self) -> Self {
        // 1. if destination has no gamut limits (XYZ-D65, XYZ-D50, Lab, LCH,
        //    Oklab, Oklch) return origin.
        // No need to check, we are a RGB based color with gamut limits.

        // Local optimization: If the color is already in gamut, then we can
        // skip the binary search and return the color.
        if self.in_gamut() {
            return self.clone();
        }

        // 2. let origin_Oklch be origin converted from origin color space to
        //    the Oklch color space.
        let origin_oklch = Oklab::from(self.clone()).to_polar();

        // 3. if the Lightness of origin_Oklch is greater than or equal to
        //    100%, return { 1 1 1 origin.alpha } in destination.
        if origin_oklch.lightness >= 1.0 {
            return Self::new(1.0, 1.0, 1.0);
        }

        // 4. if the Lightness of origin_Oklch is less than than or equal to
        //    0%, return { 0 0 0 origin.alpha } in destination.
        if origin_oklch.lightness <= 0.0 {
            return Self::new(0.0, 0.0, 0.0);
        }

        // 5. let inGamut(color) be a function which returns true if, when
        //    passed a color, that color is inside the gamut of destination.
        //    For HSL and HWB, it returns true if the color is inside the gamut
        //    of sRGB.
        //    See [`Self::in_gamut`].

        // 6. if inGamut(origin_Oklch) is true, convert origin_Oklch to
        //    destination and return it as the gamut mapped color.
        // We already made a check at the top.

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
        let mut max = origin_oklch.chroma;

        // 13. let min_inGamut be a boolean that represents when min is still
        //     in gamut, and set it to true
        let mut min_in_gamut = true;

        let mut current = origin_oklch.clone();
        let mut current_in_space = self.clone();

        // If we are already clipped, then we can return the clipped color and
        // avoid the binary search completely.
        let clipped = current_in_space.clip();
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
            current.chroma = chroma;

            current_in_space = Self::from(current.to_rectangular());

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
            let e = delta_eok(&current, &clipped);

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
        current_in_space
    }

    /// Check whether this color is within gamut limits.
    #[inline]
    fn in_gamut(&self) -> bool {
        self.red >= 0.0
            && self.red <= 1.0
            && self.green >= 0.0
            && self.green <= 1.0
            && self.blue >= 0.0
            && self.blue <= 1.0
    }

    /// Clip the components of the color.
    fn clip(&self) -> Self {
        Self::new(
            self.red.clamp(0.0, 1.0),
            self.green.clamp(0.0, 1.0),
            self.blue.clamp(0.0, 1.0),
        )
    }
}

/// Convert RGB colors in color space `S` into Rectangular form in color space `T`.
impl<S: ColorSpace, T: ColorSpace> From<Rgb<S, LinearLight>> for Rectangular<T>
where
    Rgb<S, LinearLight>: ToBase,
    Self: From<Base>,
{
    fn from(value: Rgb<S, LinearLight>) -> Self {
        Self::from(value.to_base())
    }
}

impl<S: ColorSpace> From<Rgb<S, GammaEncoded>> for Oklab
where
    S: HasGammaEncoding,
    Rgb<S, LinearLight>: ToBase,
{
    fn from(value: Rgb<S, GammaEncoded>) -> Self {
        value.to_linear_light().to_base().into()
    }
}

impl<S: ColorSpace> From<Oklab> for Rgb<S, LinearLight>
where
    Self: From<Base>,
{
    fn from(value: Oklab) -> Self {
        Self::from(value.to_base())
    }
}

impl<S: ColorSpace> From<Oklab> for Rgb<S, GammaEncoded>
where
    S: HasGammaEncoding,
    Rgb<S, LinearLight>: From<Base>,
{
    fn from(value: Oklab) -> Self {
        Rgb::<S, LinearLight>::from(value.to_base()).to_gamma_encoded()
    }
}

/// Calculate deltaE OK (simple root sum of squares).
/// <https://drafts.csswg.org/css-color-4/#color-difference-OK>
fn delta_eok<S: ColorSpace, E: GammaEncoding>(
    reference: &Polar<color_space::Oklab>,
    sample: &Rgb<S, E>,
) -> Component
where
    Rgb<S, E>: Clone,
    S: HasGammaEncoding,
    Rgb<S, LinearLight>: ToXyz,
    WhitePointFor<S>: TransferWhitePoint<D65>,
    Oklab: From<Rgb<S, E>>,
{
    // Delta is calculated in the oklab color space.
    let reference = reference.to_rectangular();
    let sample = Oklab::from(sample.clone());

    let dl = sample.lightness - reference.lightness;
    let da = sample.a - reference.a;
    let db = sample.b - reference.b;

    (dl * dl + da * da + db * db).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_component_eq;
    use crate::models::encoding::GammaEncoded;

    #[test]
    fn gamut_map_something() {
        // color(display-p3 1 0 0)
        let red = Rgb::<color_space::DisplayP3, GammaEncoded>::new(1.0, 0.0, 0.0);
        // Convert to sRGB.
        let red = Rgb::<color_space::Srgb, LinearLight>::from(red.to_linear_light().to_xyz())
            .to_gamma_encoded();
        // Map into gamut.
        let result = red.map_into_gamut_limit();

        assert_component_eq!(result.red, 1.0);
        assert_component_eq!(result.green, 0.044567645);
        assert_component_eq!(result.blue, 0.045930468);
    }
}
