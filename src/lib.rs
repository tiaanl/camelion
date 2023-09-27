//! camelion provides color priminites and operations needed by the CSS color
//! specification.
//!
//! Each color space has a struct representing its model:
//!
//! - [`Srgb`] for colors in the sRGB (gamma encoded) color space.
//! - [`SrgbLinear`] for colors in the sRGB (linear light) color space.
//! - [`Hsl`] for sRGB colors specified in the HSL (hue, saturation, lightness) form.
//! - [`Hwb`] for sRGB colors specified in the HWB (hue, whiteness, blackness) form.
//! - [`Lab`] for colors specified in the CIE-Lab color space, using the rectangular orthogonal form.
//! - [`Lch`] for colors specified in the CIE-Lab color space, using the cylindrical polar form.
//! - [`Oklab`] for colors specified in the oklab color space, using the rectangular orthogonal form.
//! - [`Oklch`] for colors specified in the oklab color space, using the cylindrical polar form.
//! - [`XyzD50`] for colors specified in the CIE-XYZ color space, with a D50 white reference.
//! - [`XyzD65`] for colors specified in the CIE-XYZ color space, with a D65 white reference.
//! - [`DisplayP3`] for colors in the Display-P3 color space, specified with red, green and blue components.
//! - [`A98Rgb`] for colors in the A98 color space, specified with red, green and blue components.
//! - [`ProPhotoRgb`] for colors in the ProPhoto RGB color space, specified with red, green and blue components.
//! - [`Rec2020`] for colors in the rec2020 color space, specified with red, green and blue components.

#![deny(missing_docs)]

mod color;
mod convert;
mod interpolate;
mod math;
mod models;

// Most common color types.
pub use color::{Color, Component, ComponentDetails, Components, Flags, Space};

// Each of the valid CSS color spaces and forms.
pub use models::hsl::Hsl;
pub use models::hwb::Hwb;
pub use models::lab::{Lab, Lch, Oklab, Oklch};
pub use models::rgb::{A98Rgb, DisplayP3, ProPhotoRgb, Rec2020, Srgb, SrgbLinear};
pub use models::xyz::{XyzD50, XyzD65};

// Trait for converting to CIE-XYZ.
pub use models::xyz::ToXyz;

// /// Most common types used while working with a camelion [`Color`].
// pub mod prelude {
//     pub use super::{Color, Component, Space};
// }
