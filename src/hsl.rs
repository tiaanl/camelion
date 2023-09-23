//! Model a color with the HSL notation in the sRGB color space.

use crate::color::{ComponentDetails, SpacePlaceholder};
use crate::{Color, Component, Components, Flags, Space};

/// A color specified with the HSL notation in the sRGB color space.
#[derive(Debug, PartialEq)]
pub struct Hsl {
    /// The hue component of the color.
    pub hue: Component,
    /// The saturation component of the color.
    pub saturation: Component,
    /// The lightness component of the color.
    pub lightness: Component,
    /// The alpha component of the color.
    pub alpha: Component,
    /// Holds any flags that might be enabled for this color.
    pub flags: Flags,
    _space: SpacePlaceholder,
}

impl Hsl {
    /// Create a new color with RGB (red, green, blue) components.
    pub fn new(
        hue: impl Into<ComponentDetails>,
        saturation: impl Into<ComponentDetails>,
        lightness: impl Into<ComponentDetails>,
        alpha: impl Into<ComponentDetails>,
    ) -> Self {
        let mut flags = Flags::empty();

        let hue = hue.into().value_and_flag(&mut flags, Flags::C0_IS_NONE);
        let saturation = saturation
            .into()
            .value_and_flag(&mut flags, Flags::C1_IS_NONE);
        let lightness = lightness
            .into()
            .value_and_flag(&mut flags, Flags::C2_IS_NONE);
        let alpha = alpha
            .into()
            .value_and_flag(&mut flags, Flags::ALPHA_IS_NONE);

        Self {
            hue,
            saturation,
            lightness,
            alpha,
            flags,
            _space: 0,
        }
    }
}

impl From<Hsl> for Color {
    fn from(value: Hsl) -> Self {
        Color {
            components: Components(value.hue, value.saturation, value.lightness),
            alpha: value.alpha,
            flags: value.flags,
            space: Space::Hsl,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Color, Hsl, Space};

    #[test]
    fn as_model() {
        let color = Color::new(Space::Hsl, 0.1, 0.2, 0.3, 0.4);
        let model = color.as_model::<Hsl>();
        assert_eq!(model.hue, color.components.0);
        assert_eq!(model.saturation, color.components.1);
        assert_eq!(model.lightness, color.components.2);
        assert_eq!(model.alpha, color.alpha);
        assert_eq!(model.flags, color.flags);
    }
}
