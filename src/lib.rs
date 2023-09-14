//! camelion provides color priminites and operations needed by the CSS color
//! specification.

#![deny(missing_docs)]

mod color;
mod convert;
mod hsl;
mod hwb;
mod lab;
mod rgb;
mod xyz;

pub use color::{Color, Component, Components, Flags, Space};
pub use hsl::Hsl;
pub use hwb::Hwb;
pub use lab::{Lab, Lch, Oklab, Oklch};
pub use rgb::{DisplayP3, Srgb, SrgbLinear};
pub use xyz::{XyzD50, XyzD65};
