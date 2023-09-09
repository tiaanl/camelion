use bitflags::bitflags;

pub type Component = f32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Components(pub Component, pub Component, pub Component);

bitflags! {
    /// Flags set on each [`Color`] when some of it's components are missing.
    #[derive(Debug, PartialEq)]
    pub struct Flags : u8 {
        /// Set when the first component of a [`Color`] is missing.
        const C0_IS_NONE = 1 << 0;
        /// Set when the second component of a [`Color`] is missing.
        const C1_IS_NONE = 1 << 1;
        /// Set when the third component of a [`Color`] is missing.
        const C2_IS_NONE = 1 << 2;
        /// Set when the alpha component of a [`Color`] is missing.
        const ALPHA_IS_NONE = 1 << 3;
    }
}

/// Represents a color space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Space {
    /// The sRGB color space.
    Srgb,
}

/// Struct that can hold a color of any color space.
pub struct Color {
    /// The three components that make up any color.
    pub components: Components,
    /// The alpha component of the color.
    pub alpha: Component,
    /// Holds any flags that might be enabled for this color.
    pub flags: Flags,
    /// The color space in which the components are set.
    pub space: Space,
}

impl Color {
    /// Create a new [`Color`]. Each color or alpha component can take values
    /// that can be converted into a [`ComponentDetails`]. This automates the
    /// process of settings values to missing. For example:
    /// ```rust
    /// use camelion::{Color, Space};
    /// let c = Color::new(Space::Srgb, None, None, None, 1.0);
    /// ```
    /// will set all the color components to missing.
    pub fn new(
        space: Space,
        c0: impl Into<ComponentDetails>,
        c1: impl Into<ComponentDetails>,
        c2: impl Into<ComponentDetails>,
        alpha: impl Into<ComponentDetails>,
    ) -> Self {
        let mut flags = Flags::empty();

        let c0 = c0.into().value_and_flag(&mut flags, Flags::C0_IS_NONE);
        let c1 = c1.into().value_and_flag(&mut flags, Flags::C1_IS_NONE);
        let c2 = c2.into().value_and_flag(&mut flags, Flags::C2_IS_NONE);
        let alpha = alpha
            .into()
            .value_and_flag(&mut flags, Flags::ALPHA_IS_NONE);

        Self {
            components: Components(c0, c1, c2),
            alpha,
            flags,
            space,
        }
    }
}

pub struct ComponentDetails {
    value: Component,
    is_none: bool,
}

impl ComponentDetails {
    /// Extract the value and set the given flag if the component is none.
    pub fn value_and_flag(&self, flags: &mut Flags, flag: Flags) -> Component {
        if self.is_none {
            *flags |= flag;
        }
        self.value
    }
}

impl From<Component> for ComponentDetails {
    fn from(value: Component) -> Self {
        Self {
            value,
            is_none: value.is_nan(),
        }
    }
}

impl From<Option<Component>> for ComponentDetails {
    fn from(value: Option<Component>) -> Self {
        if let Some(value) = value {
            Self::from(value)
        } else {
            Self {
                value: Component::NAN,
                is_none: true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_color_with_correct_components() {
        let c = Color::new(Space::Srgb, 0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.components, Components(0.1, 0.2, 0.3));
        assert_eq!(c.alpha, 0.4);
        assert_eq!(c.flags, Flags::empty());
        assert_eq!(c.space, Space::Srgb);

        let c = Color::new(Space::Srgb, 0.1, 0.2, None, 0.4);
        assert!(c.components.2.is_nan());
        assert_eq!(c.alpha, 0.4);
        assert_eq!(c.flags, Flags::C2_IS_NONE);
        assert_eq!(c.space, Space::Srgb);

        let c = Color::new(Space::Srgb, 0.1, 0.2, 0.3, None);
        assert_eq!(c.components, Components(0.1, 0.2, 0.3));
        assert!(c.alpha.is_nan());
        assert_eq!(c.flags, Flags::ALPHA_IS_NONE);
        assert_eq!(c.space, Space::Srgb);
    }

    #[test]
    fn test_component_details() {
        let cd = ComponentDetails::from(10.0);
        assert_eq!(cd.value, 10.0);
        assert_eq!(cd.is_none, false);

        let cd = ComponentDetails::from(Component::NAN);
        assert!(cd.value.is_nan());
        assert_eq!(cd.is_none, true);

        let cd = ComponentDetails::from(Some(20.0));
        assert_eq!(cd.value, 20.0);
        assert_eq!(cd.is_none, false);

        let cd = ComponentDetails::from(None);
        assert!(cd.value.is_nan());
        assert_eq!(cd.is_none, true);

        let cd = ComponentDetails::from(Some(Component::NAN));
        assert!(cd.value.is_nan());
        assert_eq!(cd.is_none, true);
    }
}
