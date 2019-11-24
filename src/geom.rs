use core::ops;
use js_sys::Math::{sin, cos};

#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

impl ops::Sub<Point> for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point { x: self.x - other.x, y: self.y - other.y }
    }
}

impl ops::Mul<f32> for Point {
    type Output = Point;

    fn mul(self, v: f32) -> Point {
        Point { x: self.x * v, y: self.y * v }
    }
}

impl Copy for Point {}
impl Clone for Point {
    fn clone(&self) -> Self {
        Point { x: self.x, y: self.y }
    }
}

impl Point {
    pub fn rotate(&self, angle: f32) -> Point {
        Point {
            x: self.x * cos(angle as f64) as f32 - self.y * sin(angle as f64) as f32,
            y: self.x * sin(angle as f64) as f32 + self.y * cos(angle as f64) as f32
        }
    }
}