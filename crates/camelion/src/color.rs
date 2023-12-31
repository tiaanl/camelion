//! A [`Color`] represents a color that was specified in any of the supported
//! CSS color spaces.

use bitflags::bitflags;

use crate::{models::Model, Component};

/// Represent the three components that describe any color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Components(pub Component, pub Component, pub Component);

impl Components {
    /// Return new components with each component mapped with the given
    /// function.
    pub fn map(&self, f: impl Fn(Component) -> Component) -> Self {
        Self(f(self.0), f(self.1), f(self.2))
    }
}

impl std::fmt::Display for Components {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.6} {:.6} {:.6})", self.0, self.1, self.2)
    }
}

impl std::ops::Sub for Components {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

bitflags! {
    /// Flags to mark any missing components on a [`Color`]
    #[derive(Clone, Copy, Debug, PartialEq)]
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

/// Various color spaces and forms supported by the CSS specification.
///<https://drafts.csswg.org/css-color-4/#color-type>
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Space {
    /// The sRGB color space.
    /// <https://drafts.csswg.org/css-color-4/#numeric-srgb>
    Srgb = 0,
    /// The HSL (hue, saturation, lightness) notation is used as an improved
    /// method of representing colors in the sRGB color space.
    /// <https://drafts.csswg.org/css-color-4/#the-hsl-notation>
    Hsl = 1,
    /// The HWB (hue, whiteness, blackness) notation is used as an improved
    /// method of specifying colors in the sRGB color space.
    /// <https://drafts.csswg.org/css-color-4/#the-hsl-notation>
    Hwb = 2,
    /// Lab
    Lab = 3,
    /// Lch
    Lch = 4,
    /// Oklab
    Oklab = 5,
    /// Oklch
    Oklch = 6,
    /// The sRGB color space with no gamma mapping.
    /// <https://drafts.csswg.org/css-color-4/#predefined-sRGB-linear>
    SrgbLinear = 7,
    /// display-p3
    DisplayP3 = 8,
    /// a98-rgb
    A98Rgb = 9,
    /// prophoto-rgb
    ProPhotoRgb = 10,
    /// rec2020
    Rec2020 = 11,
    /// xyz-d50
    XyzD50 = 12,
    /// xyz-d65
    XyzD65 = 13,
}

pub trait CssColorSpaceId {
    const ID: Space;
}

/// Used to hold any CSS supported color.
#[derive(Clone, Debug)]
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
        // `alpha` values are ALWAYS clamped to [0..1].
        let alpha = alpha
            .into()
            .value_and_flag(&mut flags, Flags::ALPHA_IS_NONE)
            .clamp(0.0, 1.0);

        Self {
            components: Components(c0, c1, c2),
            alpha,
            flags,
            space,
        }
    }

    /// Return the first component of the color.
    pub fn c0(&self) -> Option<Component> {
        if self.flags.contains(Flags::C0_IS_NONE) {
            None
        } else {
            Some(self.components.0)
        }
    }

    /// Return the second component of the color.
    pub fn c1(&self) -> Option<Component> {
        if self.flags.contains(Flags::C1_IS_NONE) {
            None
        } else {
            Some(self.components.1)
        }
    }

    /// Return the third component of the color.
    pub fn c2(&self) -> Option<Component> {
        if self.flags.contains(Flags::C2_IS_NONE) {
            None
        } else {
            Some(self.components.2)
        }
    }

    /// Return the alpha component of the color.
    pub fn alpha(&self) -> Option<Component> {
        if self.flags.contains(Flags::ALPHA_IS_NONE) {
            None
        } else {
            Some(self.alpha)
        }
    }

    /// Return a reference to this color types as the given model.
    pub fn as_model<T: Model + From<Components>>(&self) -> T {
        macro_rules! c {
            ($c:expr) => {{
                match $c {
                    // NAN values are converted to 0 for conversions, etc.
                    Some(v) if v.is_nan() => 0.0,
                    // Missing components are represented by a NAN.
                    None => Component::NAN,
                    Some(v) => v,
                }
            }};
        }

        Components(c!(self.c0()), c!(self.c1()), c!(self.c2())).into()
    }
}

/// A struct that holds details about a component passed to any of the `new`
/// functions for color models. Any components that can be passed implements
/// a `From<?> for ComponentDetails`.
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
            is_none: false,
        }
    }
}

impl From<Option<Component>> for ComponentDetails {
    fn from(value: Option<Component>) -> Self {
        if let Some(value) = value {
            Self::from(value)
        } else {
            Self {
                value: 0.0,
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
        assert_eq!(c.components.2, 0.0);
        assert_eq!(c.alpha, 0.4);
        assert_eq!(c.flags, Flags::C2_IS_NONE);
        assert_eq!(c.space, Space::Srgb);

        let c = Color::new(Space::Srgb, 0.1, 0.2, 0.3, None);
        assert_eq!(c.components, Components(0.1, 0.2, 0.3));
        assert_eq!(c.alpha, 0.0);
        assert_eq!(c.flags, Flags::ALPHA_IS_NONE);
        assert_eq!(c.space, Space::Srgb);
    }

    #[test]
    fn test_component_details() {
        let cd = ComponentDetails::from(10.0);
        assert_eq!(cd.value, 10.0);
        assert!(!cd.is_none);

        let cd = ComponentDetails::from(Component::NAN);
        assert!(cd.value.is_nan());
        assert!(!cd.is_none);

        let cd = ComponentDetails::from(Some(20.0));
        assert_eq!(cd.value, 20.0);
        assert!(!cd.is_none);

        let cd = ComponentDetails::from(None);
        assert_eq!(cd.value, 0.0);
        assert!(cd.is_none);

        let cd = ComponentDetails::from(Some(Component::NAN));
        assert!(cd.value.is_nan());
        assert!(!cd.is_none);
    }

    #[test]
    fn models_use_zero_not_nan() {
        let c = Color::new(
            Space::Oklch,
            Component::NAN,
            Component::NAN,
            Component::NAN,
            1.0,
        );
        let model = c.as_model::<crate::models::Oklch>();
        assert_eq!(model.lightness, 0.0);
        assert_eq!(model.chroma, 0.0);
        assert_eq!(model.hue, 0.0);
    }
}
