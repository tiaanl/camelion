use camelion::prelude::*;

fn main() {
    let left = Color::new(Space::Srgb, 1.0, 0.0, 0.0, 1.0);
    let right = Color::new(Space::Srgb, 0.0, 1.0, 0.0, 1.0);

    let middle = left.interpolate(&right, Space::Oklab).at(0.5);
    println!("interpolated at 0.5 = {:?}", middle);

    // Convert the result back to sRGB color space.
    let srgb = middle.to_space(Space::Srgb);
    println!("back to sRGB = {:?}", srgb);
}
