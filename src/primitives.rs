extern crate cgmath;
extern crate rand;
extern crate euclid;
extern crate conrod;
use serde::{Deserialize, Serialize};

// use piston_window::*;
use cgmath::*;
use euclid::*;

pub type Point2f64 = Point2<f64>;

pub type Vec2i32 = cgmath::Vector2<i32>;
pub type Vec2f32 = cgmath::Vector2<f32>;
pub type Vec2f64 = Vector2<f64>;

pub type Size2f64 = Size2D<f64>;

pub fn vec2f64_2_vec2f32(v: Vec2f64) -> Vec2f32 {
    Vec2f32::new(v.x as f32, v.y as f32)
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "cgmath::Point2::<f64>")]
struct Point2f64Serde {
    x: f64,
    y: f64,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pose2DF64 {
   #[serde(with = "Point2f64Serde")]
   pub center: Point2f64,
   pub yaw: f64
}

impl Default for Pose2DF64 {


    fn default() -> Self {
        Pose2DF64 {
            center : Point2f64{x: 0.0, y: 0.0},
            yaw: 0.0
        }
    }

}


pub fn zero_vec2f64() -> Vec2f64 {
    Vec2f64{x: 0.0, y: 0.0}
}