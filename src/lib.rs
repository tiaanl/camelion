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

// All the models can be accessed through the module.
pub mod models;

// Most common color types.
pub use color::{Color, Component, ComponentDetails, Components, Flags, Space};

// Color interpolation types.
pub use interpolate::{HueInterpolationMethod, Interpolation};
