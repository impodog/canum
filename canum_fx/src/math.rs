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
