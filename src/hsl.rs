//! Model a color with the HSL notation in the sRGB color space.

use crate::{color::HasSpace, Color, Component, Components, Space};

camelion_macros::gen_model! {
    /// A color specified with the HSL notation in the sRGB color space.
    pub struct Hsl {
        /// The hue component of the color.
        pub hue: Component,
        /// The saturation component of the color.
        saturation: Component,
        /// The lightness component of the color.
        lightness: Component,
    }
}

impl HasSpace for Hsl {
    const SPACE: Space = Space::Hsl;
}

impl From<Hsl> for Color {
    fn from(value: Hsl) -> Self {
        Color {
            components: Components(value.hue, value.saturation, value.lightness),
            alpha: value.alpha,
            flags: value.flags,
            space: Hsl::SPACE,
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

    // #[test]
    // fn p() {
    //     let _hsl2 = super::Hsl2 {
    //         hue: 0.0,
    //         saturation: 0.0,
    //         lightness: 0.0,
    //         alpha: 0.0,
    //         flags: Flags::empty(),
    //         _space: Default::default(),
    //     };
    // }
}
