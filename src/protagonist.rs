use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use std::vec::Vec;
use std::collections::hash_set::HashSet;
use std::collections::vec_deque::VecDeque;
use super::car::*;
extern crate piston_window;
extern crate specs_derive;

#[derive(Default)]
pub struct Twist2D {
    pub x: f64,
    pub y: f64,
    pub z_rot: f64
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ProtagonistTag;


pub struct ControlProtagonistSys<'a> {
   pub target_protagonist_twist: &'a Twist2D
}

impl <'a, 'b> System<'a> for ControlProtagonistSys<'b> {
    type SystemData = (
        WriteStorage<'a, Car>,
        ReadStorage<'a, ProtagonistTag>,
    );

    fn run(&mut self, (mut cars, protagonists): Self::SystemData) {
        for (car, _protagonist) in (&mut cars, &protagonists).join() {
            let speed = self.target_protagonist_twist.x as f32;
            let yaw_rate = self.target_protagonist_twist.z_rot as f32;
            car.longitudinal_speed = speed;

            let yaw_increment = (yaw_rate - car.yaw_rate);  
            let max_yaw_increment = 0.2f32;
            let yaw_increment_clamped = f32::max(-max_yaw_increment, f32::min(max_yaw_increment, yaw_increment));

            if(car.longitudinal_speed > 0.0f32) {
                let new_wheel_yaw = yaw_increment_clamped / car.longitudinal_speed * (car.bb_size.height as f32 / 2.0f32);
                let max_wheel_yaw = 0.6;
                let new_clamped_wheel_yaw = f32::max(-max_wheel_yaw, f32::min(max_wheel_yaw, new_wheel_yaw));
                car.wheel_yaw = new_clamped_wheel_yaw;
            }
        }
    }
}
