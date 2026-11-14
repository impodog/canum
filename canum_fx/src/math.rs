pub use rand::{random_bool as rand_bool, random_range as rand_range};

/// this will ensure the result falls between mean `mean` +/- 3 * `std_dev`
#[inline(always)]
pub fn rand_normal(mean: f32, std_dev: f32) -> f32 {
    rand_normal_clamp(mean, std_dev, 3.0)
}

/// this will ensure the result falls between mean `mean` +/- `multiply` * `std_dev`
pub fn rand_normal_clamp(mean: f32, std_dev: f32, multiply: f32) -> f32 {
    use rand_distr::Distribution;
    let result = rand_distr::Normal::new(mean, std_dev)
        .unwrap()
        .sample(&mut rand::rng());
    result.clamp(mean - multiply * std_dev, mean + multiply * std_dev)
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

/// Returns the longer coordinate relative to screen size, while the other one is set to `0.0`.
pub fn long_side_relative_to_screen(vec: bevy::prelude::Vec2) -> bevy::prelude::Vec2 {
    use canum_res::config::CONFIG;
    if (vec.x * CONFIG.display.screen_size.y).abs() >= (vec.y * CONFIG.display.screen_size.x).abs()
    {
        bevy::prelude::vec2(vec.x, 0.0)
    } else {
        bevy::prelude::vec2(0.0, vec.y)
    }
}
/// Keep one random coordinate x or y, and set the other to `0.0`.
pub fn random_side(vec: bevy::prelude::Vec2) -> bevy::prelude::Vec2 {
    if rand_bool(0.5) {
        bevy::prelude::vec2(vec.x, 0.0)
    } else {
        bevy::prelude::vec2(0.0, vec.y)
    }
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
macro_rules! quadratic_simple_curve {
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

#[macro_export]
/// Builds a `fn(f32) -> f32` for a derivable piecewise-quadratic curve on \[0, 1\].
///
/// Requires:
///   start_slope, end_slope >= 0
///
/// Guarantees:
///   f(0) = 0,  f(1) = 1
///   f'(0) = start_slope,  f'(1) = end_slope
///   f'(x) >= 0
macro_rules! quadratic_curve {
    ($start_slope:expr, $end_slope:expr) => {{
        fn f(x: f32) -> f32 {
            const START_SLOPE: f32 = $start_slope;
            const END_SLOPE: f32 = $end_slope;
            debug_assert!(
                START_SLOPE >= 0.0 && END_SLOPE >= 0.0,
                "slopes must be non-negative"
            );

            // Exact endpoint values (avoid f32 rounding near 0 and 1).
            if x <= 0.0 {
                return 0.0;
            }
            if x >= 1.0 {
                return 1.0;
            }

            // t = half-width of each quadratic end segment.
            const T: f32 = 1.0 / (2.0 * (START_SLOPE + END_SLOPE + 2.0));
            // Middle linear slope (always strictly positive).
            const M: f32 = (1.0 - 0.5 * T * (START_SLOPE + END_SLOPE)) / (1.0 - T);

            // Left quadratic:  fL(x) = a·x + α·x²            on [0, t]
            // Right quadratic: fR(x) = yR + m·w + β·w², w=x−(1−t) on [1−t, 1]
            const ALPHA: f32 = (M - START_SLOPE) / (2.0 * T);
            const BETA: f32 = (END_SLOPE - M) / (2.0 * T);
            const YL: f32 = 0.5 * T * (START_SLOPE + M);
            const YR: f32 = 1.0 - 0.5 * T * (END_SLOPE + M);

            if x <= T {
                START_SLOPE * x + ALPHA * x * x
            } else if x <= 1.0 - T {
                YL + M * (x - T)
            } else {
                let w = x - (1.0 - T);
                YR + M * w + BETA * w * w
            }
        }
        f as fn(f32) -> f32
    }};
}

/// Builds a `fn(f32) -> f32` for a derivable piecewise-quadratic curve on \[0, 1\].
///
/// Requires:
///   start_slope, end_slope >= 0 and 1.0 is between the two slopes (exclusive)
///
/// Guarantees:
///   f(0) = 0,  f(1) = 1
///   f'(0) = start_slope,  f'(1) = end_slope
///   f'(x) >= 0 and is monotonic
#[macro_export]
macro_rules! cubic_curve {
    ($start_slope:expr, $end_slope:expr) => {{
        const START_SLOPE: f32 = $start_slope;
        const END_SLOPE: f32 = $end_slope;

        // ---- normalise to the convex case 0 <= A < 1 < B -----------------
        const FLIP: bool = START_SLOPE > END_SLOPE;
        const A: f32 = if FLIP { END_SLOPE } else { START_SLOPE };
        const B: f32 = if FLIP { START_SLOPE } else { END_SLOPE };

        const _: () = assert!(
            A >= 0.0 && B >= 0.0 && A < 1.0 && B > 1.0,
            "mono_cubic!: slopes must be >= 0 and satisfy min(A,B) < 1 < max(A,B)"
        );

        const P: f32 = 1.0 - A; // p
        const Q: f32 = B - 1.0; // q
        const D: f32 = P + Q; // B - A  = total mass of f''

        // 0 = single cubic | 1 = late rise (t in (1/2,1)) | 2 = early rise (t in (0,1/2))
        const REGIME: i8 = if Q <= 2.0 * P && P <= 2.0 * Q {
            0
        } else if Q > 2.0 * P {
            1
        } else {
            2
        };

        const T: f32 = match REGIME {
            // knot
            0 => 1.0,
            1 => (2.0 * Q - P) / (2.0 * D),
            _ => 3.0 * Q / (2.0 * D),
        };
        const ELL: f32 = 1.0 - T; // 1 - t

        const K_T: f32 = match REGIME {
            1 => 3.0 * P,
            2 => 3.0 * Q,
            _ => 0.0,
        }; // f''(t)
        const K_1: f32 = if REGIME == 1 {
            (2.0 * Q - P) / ELL
        } else {
            0.0
        }; // f''(1)
        const K_0: f32 = if REGIME == 2 { (2.0 * P - Q) / T } else { 0.0 }; // f''(0)

        // left piece, in x:       L0 + L1·x + L2·x² + L3·x³
        const L0: f32 = 0.0;
        const L1: f32 = A;
        const L2: f32 = match REGIME {
            0 => 2.0 * P - Q,
            1 => 0.0,
            _ => K_0 / 2.0,
        };
        const L3: f32 = match REGIME {
            0 => Q - P,
            1 => P / (2.0 * T),
            _ => (K_T - K_0) / (6.0 * T),
        };

        // right piece, in u = 1-x: R0 + R1·u + R2·u² + R3·u³
        const R0: f32 = 1.0;
        const R1: f32 = -B;
        const R2: f32 = if REGIME == 1 { K_1 / 2.0 } else { 0.0 };
        const R3: f32 = match REGIME {
            1 => (K_T - K_1) / (6.0 * ELL),
            2 => Q / (2.0 * ELL),
            _ => 0.0,
        };

        fn curve(x: f32) -> f32 {
            let x = if FLIP { 1.0 - x } else { x };
            let y = if x <= T {
                ((L3 * x + L2) * x + L1) * x + L0
            } else {
                let u = 1.0 - x;
                ((R3 * u + R2) * u + R1) * u + R0
            };
            if FLIP { 1.0 - y } else { y }
        }

        curve as fn(f32) -> f32
    }};
}
