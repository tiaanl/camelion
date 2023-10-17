/// Check for equality between two components allowing for 16-bit rounding
/// errors.
#[macro_export]
macro_rules! assert_component_eq {
    ($actual:expr,$expected:expr) => {{
        approx::assert_abs_diff_eq!($actual, $expected, epsilon = 1.0 / i16::MAX as Component);
    }};
}
