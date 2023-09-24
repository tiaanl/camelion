use camelion::{Lab, SrgbLinear, ToXyz};

pub fn main() {
    #[allow(clippy::excessive_precision)]
    let lab = Lab::new(
        56.629300221279735,
        39.237080198427755,
        57.553769167682276,
        1.0,
    );

    // 0.3373008675302083, 0.2454491947638009, 0.0319588705314679
    let xyz_d50 = lab.to_xyz();

    // 0.31863421971306805, 0.23900587532696937, 0.041636956453517074
    let xyz_d65 = xyz_d50.to_xyz_d65();

    // 0.6444796819705821, 0.14126329114027164, 0.012983032342173012
    let srgb_linear = SrgbLinear::from(xyz_d65);

    // 0.8235294117647058, 0.4117647058823529, 0.11764705882352941
    let srgb = srgb_linear.to_gamma_encoded();

    dbg!(srgb);
}
