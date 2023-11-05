//! Tags for color spaces.

/// Represents a color space.
pub trait ColorSpace: Clone {}

/// The sRGB color space.
#[derive(Clone, Debug)]
pub struct Srgb;

impl ColorSpace for Srgb {}

/// The Diplay-P3 color space.
#[derive(Clone, Debug)]
pub struct DisplayP3;

impl ColorSpace for DisplayP3 {}

/// The ProPhoto-RGB color space.
#[derive(Clone, Debug)]
pub struct ProPhotoRgb;

impl ColorSpace for ProPhotoRgb {}

/// The Adobe-RGB color space.
#[derive(Clone, Debug)]
pub struct A98Rgb;

impl ColorSpace for A98Rgb {}

/// The Rec.2020 color space.
#[derive(Clone, Debug)]
pub struct Rec2020;

impl ColorSpace for Rec2020 {}

/// The CIE-Lab color space.
#[derive(Clone, Debug)]
pub struct Lab;

impl ColorSpace for Lab {}

/// The Oklab color space.
#[derive(Clone, Debug)]
pub struct Oklab;

impl ColorSpace for Oklab {}

/// The CIE-XYZ color space.
#[derive(Clone, Debug)]
pub struct Xyz;

impl ColorSpace for Xyz {}
