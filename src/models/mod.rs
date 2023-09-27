use crate::color::{Color, Component};

pub mod hsl;
pub mod hwb;
pub mod lab;
pub mod rgb;
pub mod xyz;

/// A trait implemented for color models that can be converted to and from a
/// generic [`Color`].
pub trait Model {
    /// Convert a model to a generic [`Color`].
    fn to_color(&self, alpha: Component) -> Color;

    /// Convert a generic [`Color`] to a model.
    fn to_model(color: &Color) -> Self;
}
