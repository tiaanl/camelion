use camelion::{Color, Space};
use image::{Rgba, RgbaImage};
use rusttype::{point, Font, Scale};

const WIDTH: u32 = 1000;
const HEIGHT_PER_SPACE: u32 = 100;

fn main() {
    let left = Color::new(Space::Srgb, 1.0, 0.0, 0.0, 1.0);
    let right = Color::new(Space::Srgb, 0.0, 0.0, 1.0, 1.0);

    let interps = [
        Space::Srgb,
        Space::Hsl,
        Space::Hwb,
        Space::Lab,
        Space::Lch,
        Space::Oklab,
        Space::Oklch,
        Space::SrgbLinear,
        Space::DisplayP3,
        Space::A98Rgb,
        Space::ProPhotoRgb,
        Space::Rec2020,
        Space::XyzD50,
        Space::XyzD65,
    ]
    .map(|space| left.interpolate(&right, space));

    let height = interps.len() as u32 * HEIGHT_PER_SPACE;

    let mut img = RgbaImage::new(WIDTH, height);
    img.fill(255);

    img.enumerate_rows_mut().for_each(|(_, pixels)| {
        for (x, y, pixel) in pixels {
            let t = x as f32 / WIDTH as f32;

            let interp_index = y / (height / interps.len() as u32);

            let c = interps[interp_index as usize]
                .at(t)
                .to_space(Space::Srgb)
                .map_into_gamut_limits();

            assert!(
                c.in_gamut(),
                "Out of gamut limits: {:?} {}",
                interps[interp_index as usize].space,
                c.components
            );

            *pixel = Rgba([
                (c.components.0.clamp(0.0, 1.0) * 255.0).round() as u8,
                (c.components.1.clamp(0.0, 1.0) * 255.0).round() as u8,
                (c.components.2.clamp(0.0, 1.0) * 255.0).round() as u8,
                255,
            ]);
        }
    });

    let font = Vec::from(include_bytes!("../DejaVuSans.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    let scale = Scale::uniform(HEIGHT_PER_SPACE as f32 / 2.0);
    interps.iter().enumerate().for_each(|(i, interp)| {
        let text = format!("{:?}", interp.space);
        let (t_width, t_height) = measure_line(&font, text.as_str(), scale);
        imageproc::drawing::draw_text_mut(
            &mut img,
            Rgba([0, 0, 0, 127]),
            ((WIDTH as f32 / 2.0) - t_width / 2.0).round() as i32,
            (HEIGHT_PER_SPACE as f32 * i as f32 + (HEIGHT_PER_SPACE as f32 / 2.0 - t_height / 2.0))
                .round() as i32,
            scale,
            &font,
            text.as_str(),
        );
    });

    img.save("out.png")
        .expect("could not write image to out.png");
}

fn measure_line(font: &Font, text: &str, scale: Scale) -> (f32, f32) {
    let width = font
        .layout(text, scale, point(0.0, 0.0))
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .last()
        .unwrap_or(0.0);

    let v_metrics = font.v_metrics(scale);
    let height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;

    (width, height)
}
