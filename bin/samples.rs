//! This program will render some sample images with color of different color
//! spaces converted to sRGB.

use std::path::Path;

use camelion::{Color, Space};
use image::RgbImage;

fn write_image(path: impl AsRef<Path>, color: &Color) {
    let color = color.to_space(Space::Srgb);
    let mut image = RgbImage::new(20, 20);
    for pixel in image.enumerate_pixels_mut() {
        let (_, _, pixel) = pixel;
        pixel[0] = (color.components.0 * 255.0).round() as u8;
        pixel[1] = (color.components.1 * 255.0).round() as u8;
        pixel[2] = (color.components.2 * 255.0).round() as u8;
    }
    image.save(path).unwrap();
}

fn main() {
    println!("Building samples");

    // chocolate
    let color = Color::new(Space::Srgb, 0.8235, 0.4118, 0.11765, 1.0);

    write_image("samples/srgb.png", &color);
    write_image(
        "samples/srgb-linear.png",
        &color.to_space(Space::SrgbLinear),
    );
    write_image("samples/hsl.png", &color.to_space(Space::Hsl));
    write_image("samples/hwb.png", &color.to_space(Space::Hwb));
}
