extern crate cgmath;
extern crate rand;
extern crate euclid;
extern crate conrod;

// use piston_window::*;
use cgmath::*;
use euclid::*;

pub type Point2f64 = Point2<f64>;
pub type Vec2f64 = Vector2<f64>;
pub type Size2f64 = Size2D<f64>;

#[derive(Clone, Debug)]
pub struct Pose2DF64 {
   pub center: Point2f64,
   pub yaw: f64
}


pub fn zero_vec2f64() -> Vec2f64 {
    Vec2f64{x: 0.0, y: 0.0}
}