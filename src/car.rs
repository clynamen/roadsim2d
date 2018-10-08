use super::primitives::*;

use cgmath::*;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;
use piston::event_loop::*;
//use rosrust::PublisherStream;
use rosrust::api::raii::Publisher;
use euclid::*;
use conrod::color::*;
use piston_window::*;
use piston::event_loop::*;

use super::color_utils::toRgba;
use super::primitives::*;


pub fn draw_car(context: Context, graphics: &mut G2d, 
    center: Point2f64, rot: f64, car_size: Size2f64, color: Color)  {
        let car_center = center + Vec2f64{x: car_size.height/2.0, y: car_size.width/2.0};
        let center = context.transform.trans(car_center.x, car_center.y);
        // let square = rectangle::square(0.0, 0.0, 100.0);
        rectangle( toRgba(&color, 1.0f32), // red
                    [-car_size.height/2.0, 
                    -car_size.width/2.0, 
                    car_size.height, 
                    car_size.width],
                    center.rot_rad(rot),
                    graphics);
}

#[derive(Clone, Debug)]
pub struct Car {
    pub id : u64,
    pub pose : Pose2DF64,     
    pub longitudinal_speed : f32, 
    pub yaw_rate: f32,
    pub bb_size : Size2f64,
    pub color: Color
}


impl Car {
    pub fn update(self: &mut Car, dt: f32, change_yawrate : bool) {
        let rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(self.pose.yaw));
        let ds  = Vector2{x: self.longitudinal_speed as f64, y: 0.0};
        let rotated_ds = rot.rotate_vector(ds);
        self.pose.center += rotated_ds;

        let direction_to_center = Vector2{x:400.0, y:400.0} - self.pose.center.to_vec();
        let direction_rand = rand::thread_rng().gen_range(0.0, 1e-8)*direction_to_center.magnitude2()as f32;
        if(change_yawrate) {
            self.yaw_rate = -direction_to_center.angle(rotated_ds).0.signum() as f32* direction_rand;
        }
        self.pose.yaw += (self.yaw_rate * dt) as f64;
    }
}
