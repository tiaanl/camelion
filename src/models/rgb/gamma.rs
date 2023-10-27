//! Gamma encoding and decoding.

use crate::{color_space, Component, Components};

/// The conversion to and from gamma encoded components.
pub trait HasGammaEncoding {
    /// Convert the components from linear light to gamma encoded.
    fn to_gamma_encoded(from: &Components) -> Components;

    /// Convert the components from gamma encoded to linear light.
    fn to_linear_light(from: &Components) -> Components;
}

impl HasGammaEncoding for color_space::Srgb {
    fn to_gamma_encoded(from: &Components) -> Components {
        from.map(|value| {
            let abs = value.abs();

            if abs > 0.0031308 {
                value.signum() * (1.055 * abs.powf(1.0 / 2.4) - 0.055)
            } else {
                12.92 * value
            }
        })
    }

    fn to_linear_light(from: &Components) -> Components {
        from.map(|value| {
            let abs = value.abs();

            if abs < 0.04045 {
                value / 12.92
            } else {
                value.signum() * ((abs + 0.055) / 1.055).powf(2.4)
            }
        })
    }
}

impl HasGammaEncoding for color_space::DisplayP3 {
    fn to_gamma_encoded(from: &Components) -> Components {
        color_space::Srgb::to_gamma_encoded(from)
    }

    fn to_linear_light(from: &Components) -> Components {
        color_space::Srgb::to_linear_light(from)
    }
}

impl HasGammaEncoding for color_space::A98Rgb {
    fn to_gamma_encoded(from: &Components) -> Components {
        from.map(|v| v.signum() * v.abs().powf(256.0 / 563.0))
    }

    fn to_linear_light(from: &Components) -> Components {
        from.map(|v| v.signum() * v.abs().powf(563.0 / 256.0))
    }
}

impl HasGammaEncoding for color_space::ProPhotoRgb {
    fn to_gamma_encoded(from: &Components) -> Components {
        const E: Component = 1.0 / 512.0;

        from.map(|v| {
            let abs = v.abs();

            if abs >= E {
                v.signum() * abs.powf(1.0 / 1.8)
            } else {
                16.0 * v
            }
        })
    }

    fn to_linear_light(from: &Components) -> Components {
        const E: Component = 16.0 / 512.0;

        from.map(|v| {
            let abs = v.abs();

            if abs <= E {
                v / 16.0
            } else {
                v.signum() * abs.powf(1.8)
            }
        })
    }
}

impl color_space::Rec2020 {
    #[allow(clippy::excessive_precision)]
    const ALPHA: Component = 1.09929682680944;
    #[allow(clippy::excessive_precision)]
    const BETA: Component = 0.018053968510807;
}

impl HasGammaEncoding for color_space::Rec2020 {
    fn to_gamma_encoded(from: &Components) -> Components {
        from.map(|v| {
            let abs = v.abs();

            if abs > Self::BETA {
                v.signum() * (Self::ALPHA * abs.powf(0.45) - (Self::ALPHA - 1.0))
            } else {
                4.5 * v
            }
        })
    }

    fn to_linear_light(from: &Components) -> Components {
        from.map(|v| {
            let abs = v.abs();

            if abs < Self::BETA * 4.5 {
                v / 4.5
            } else {
                v.signum() * ((abs + Self::ALPHA - 1.0) / Self::ALPHA).powf(1.0 / 0.45)
            }
        })
    }
}
