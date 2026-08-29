pub use rand::{random_bool as rand_bool, random_range as rand_range};

/// This will ensure the result falls between mean +/- 3 * std_dev
pub fn rand_normal(mean: f32, std_dev: f32) -> f32 {
    use rand_distr::Distribution;
    let result = rand_distr::Normal::new(mean, std_dev)
        .unwrap()
        .sample(&mut rand::rng());
    result.clamp(mean - 3.0 * std_dev, mean + 3.0 * std_dev)
}

/// Returns a random sign (1.0 or -1.0) with equal probability.
pub fn rand_sign() -> f32 {
    if rand::random_bool(0.5) { 1.0 } else { -1.0 }
}

/// Returns a small enough random transform z offset, when there are multiple entities with the same z layer,
/// this makes sure they have a constant random layering.
pub fn rand_offset() -> f32 {
    rand::random_range(-1e-3..1e-3)
}

/// Take any angle to return between [0, 2π).
#[inline(always)]
pub fn normalize_angle(angle: f32) -> f32 {
    use std::f32::consts::TAU;
    let a = angle % TAU;
    if a < 0.0 { a + TAU } else { a }
}

/// Take any angle to return between (-π, π].
#[inline(always)]
pub fn normalize_angle_signed(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    let a = normalize_angle(angle);
    if a > PI { a - TAU } else { a }
}

#[macro_export]
/// Builds a `fn(f32) -> f32` for a derivable piecewise-quadratic curve on [0, 1].
///
/// Guarantees:
///   f(0) = 0,  f(1) = 1
///   f'(0) = start_slope,  f'(1) = end_slope
///   The two quadratic halves meet at x = 0.5 with matched value and slope.
///
/// This overshoots [0, 1] if any of the slopes is outside [0, 2]
macro_rules! quadratic_curve {
    ($start_slope:expr, $end_slope:expr) => {{
        const S0: f32 = $start_slope as f32;
        const S1: f32 = $end_slope as f32;

        const A1: f32 = 0.5 * (4.0 - 3.0 * S0 - S1);
        const B2: f32 = 0.5 * (S0 + 3.0 * S1 - 4.0);

        fn __curve(x: f32) -> f32 {
            if x <= 0.5 {
                A1 * x * x + S0 * x
            } else {
                let u = x - 1.0;
                1.0 + u * (S1 + B2 * u)
            }
        }

        __curve as fn(f32) -> f32
    }};
}
