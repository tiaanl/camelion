//! Model a color with the HWB notation in the sRGB color space.

use crate::{color::HasSpace, Color, Component, Components, Space};

camelion_macros::gen_model! {
    /// A color specified with the HWB notation in the sRGB color space.
    pub struct Hwb {
        /// The hue component of the color.
        pub hue: Component,
        /// The whiteness component of the color.
        pub whiteness: Component,
        /// The blackness component of the color.
        pub blackness: Component,
    }
}

impl HasSpace for Hwb {
    const SPACE: Space = Space::Hwb;
}

impl From<Hwb> for Color {
    fn from(value: Hwb) -> Self {
        Color {
            components: Components(value.hue, value.whiteness, value.blackness),
            alpha: value.alpha,
            flags: value.flags,
            space: Hwb::SPACE,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Color, Hwb, Space};

    #[test]
    fn as_model() {
        let color = Color::new(Space::Hwb, 0.1, 0.2, 0.3, 0.4);
        let model = color.as_model::<Hwb>();
        assert_eq!(model.hue, color.components.0);
        assert_eq!(model.whiteness, color.components.1);
        assert_eq!(model.blackness, color.components.2);
        assert_eq!(model.alpha, color.alpha);
        assert_eq!(model.flags, color.flags);
    }
}
