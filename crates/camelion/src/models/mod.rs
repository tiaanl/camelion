//! Models are structs that represent a color in a specified color space or
//! form. They represent a type safe way to convert between different color
//! spaces and forms.

use crate::{color::Color, Component};

mod base;
mod hsl;
mod hwb;
mod lab;
mod rgb;
mod xyz;

pub use hsl::*;
pub use hwb::*;
pub use lab::*;
pub use rgb::*;
pub use xyz::*;

pub use base::{Base, BaseWhitePoint, ToBase};

/// A trait implemented for color models that can be converted to and from a
/// generic [`Color`].
pub trait Model {
    /// Convert a model to a generic [`Color`].
    fn to_color(&self, alpha: Option<Component>) -> Color;
}
