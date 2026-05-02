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
