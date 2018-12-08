use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use std::vec::Vec;
use std::collections::hash_set::HashSet;
use std::collections::vec_deque::VecDeque;
use super::car::*;
use super::physics::*;
use nphysics2d::world::World as PWorld;
use nphysics2d::math::Velocity;
use nphysics2d::math::Force;
use nalgebra::Vector2;
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
   pub physics_world: &'a mut PWorld<f64>,
   pub target_protagonist_twist: &'a Twist2D
}

impl <'a, 'b> System<'a> for ControlProtagonistSys<'b> {
    type SystemData = (
        WriteStorage<'a, PhysicsComponent>,
        WriteStorage<'a, Car>,
        ReadStorage<'a, ProtagonistTag>,
    );

    fn run(&mut self, (mut physics_components, mut cars, protagonists): Self::SystemData) {
        for (physics_component, car, _protagonist) in (&mut physics_components, &mut cars, &protagonists).join() {
            let speed = self.target_protagonist_twist.x as f32;
            let yaw_rate = self.target_protagonist_twist.z_rot as f32;

            let mut rigid_body = self.physics_world.rigid_body_mut(physics_component.body_handle).expect("protagonist rigid body not found");

            let current_yaw_rate = rigid_body.velocity().angular as f32;

            let yaw_increment = (yaw_rate - current_yaw_rate);  
            let max_yaw_increment = 0.2f32;
            let yaw_increment_clamped = f32::max(-max_yaw_increment, f32::min(max_yaw_increment, yaw_increment));

            let car_frame_velocity = Vector2::<f64>::new(speed as f64, 0.0);
            let mut car_velocity = car_frame_velocity.clone();
            rigid_body.position().rotation.rotate(&mut car_velocity);

            rigid_body.set_linear_velocity(car_velocity);

            // println!("car long speed {}  yaw_rate {}", car_longitudinal_speed, yaw_increment_clamped);
            if(speed > 0.0f32) {
                let new_wheel_yaw = yaw_increment_clamped / speed * (car.bb_size.height as f32 / 2.0f32);
                let max_wheel_yaw = 0.6;
                let new_clamped_wheel_yaw = f32::max(-max_wheel_yaw, f32::min(max_wheel_yaw, new_wheel_yaw));

                car.wheel_yaw = new_clamped_wheel_yaw;
                rigid_body.set_angular_velocity(yaw_increment_clamped as f64);
            }

        }
    }
}
