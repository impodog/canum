use bevy::prelude::*;
use canum_res::config::CONFIG;

/// Finds the closest intersection between four rect sides and the ray.
pub fn rect_intersect_ray(rect: Rect, point: Vec2, direction: Dir2) -> Option<Vec2> {
    // Tests if the line is parallel to x axis.
    const EPS: f32 = 1e-9;

    let mut t_enter = f32::NEG_INFINITY;
    let mut t_exit = f32::INFINITY;

    // X-axis slab
    if direction.x.abs() < EPS {
        if point.x < rect.min.x || point.x > rect.max.x {
            return None;
        }
    } else {
        let tx1 = (rect.min.x - point.x) / direction.x;
        let tx2 = (rect.max.x - point.x) / direction.x;
        t_enter = t_enter.max(tx1.min(tx2));
        t_exit = t_exit.min(tx1.max(tx2));
    }

    // Y-axis slab
    if direction.y.abs() < EPS {
        if point.y < rect.min.y || point.y > rect.max.y {
            return None;
        }
    } else {
        let ty1 = (rect.min.y - point.y) / direction.y;
        let ty2 = (rect.max.y - point.y) / direction.y;
        t_enter = t_enter.max(ty1.min(ty2));
        t_exit = t_exit.min(ty1.max(ty2));
    }

    // Miss conditions: slabs don't overlap, or rectangle is entirely behind the ray
    if t_enter > t_exit || t_exit < 0.0 {
        return None;
    }

    // If origin is inside the rect, t_enter will be negative.
    // We want the closest forward intersection, which is t_exit.
    let t_hit = if t_enter < 0.0 { t_exit } else { t_enter };

    // Reject hits at or extremely close to the origin itself
    if t_hit <= EPS {
        return None;
    }

    Some(Vec2 {
        x: point.x + t_hit * direction.x,
        y: point.y + t_hit * direction.y,
    })
}

/// Finds the intersection between the ray and screen border. If there isn't one, the original point is returned.
pub fn screen_border_intersect_ray(point: Vec2, direction: Dir2) -> Vec2 {
    rect_intersect_ray(CONFIG.display.screen_rect, point, direction).unwrap_or(point)
}

/// By using probability, generate a sequence of integers to approximate the actual float number.
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ApproxFloat {
    pub integer: i32,
    pub float: f64,
}

impl ApproxFloat {
    pub fn sample(&self) -> i32 {
        if self.float > 0.0 && self.float <= 1.0 && rand::random_bool(self.float) {
            self.integer + 1
        } else {
            self.integer
        }
    }
    pub fn add(&self, rhs: f64) -> Self {
        let rem = self.float + rhs;
        let floor = rem.floor();
        Self {
            integer: self.integer + floor as i32,
            float: rem - floor,
        }
    }
    pub fn mul(&self, rhs: f64) -> Self {
        let new_value = (self.integer as f64 + self.float) * rhs;
        Self::from(new_value)
    }
}
impl From<f64> for ApproxFloat {
    fn from(value: f64) -> Self {
        let floor = value.floor();
        Self {
            integer: floor as i32,
            float: value - floor,
        }
    }
}
impl From<i32> for ApproxFloat {
    fn from(value: i32) -> Self {
        Self {
            integer: value,
            float: 0.0,
        }
    }
}
