use super::primitives::*;
use super::physics::*;
use super::car_hl_controller::*;
use super::node::Node;
use super::car::Car;
use super::global_resources::*;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use nphysics2d::world::World as PWorld;
use nphysics2d::math::Velocity;
use nphysics2d::math::Force;
use nalgebra::Vector2;
use num;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CarController {
    // pub target_velocity: f32
}


pub struct CarControllerSys<'a> {
   pub physics_world: &'a mut PWorld<f64>
}

const CAR_ACC : f64 = 10.0f64;

impl <'a, 'b> System<'a> for CarControllerSys<'b> {
    type SystemData = (
        ReadExpect<'a, UpdateDeltaTime>, 
        ReadStorage<'a, CarController>,
        ReadStorage<'a, CarHighLevelControllerState>,
        ReadStorage<'a, Node>,
        WriteStorage<'a, PhysicsComponent>,
        WriteStorage<'a, Car>,
    );

    fn run(&mut self, (update_delta_time, car_controllers, car_high_level_controller_states, 
            nodes, mut physics_components, mut cars): Self::SystemData) {
        let dt = update_delta_time.dt;

        for (car_controller, physics_component, node, car, car_high_level_controller_state) in 
                (&car_controllers, &mut physics_components, &nodes, &mut cars, &car_high_level_controller_states).join() {
            let mut rigid_body = self.physics_world.rigid_body_mut(physics_component.body_handle).expect("car rigid body not found");
            let current_speed = rigid_body.velocity();

            let current_speed_mag = current_speed.linear.norm();

            let target_yaw_diff = car_high_level_controller_state.target_yaw - node.pose.yaw as f32;
            let correct_direction_yaw_diff = if target_yaw_diff < 0.0f32 {
                target_yaw_diff  + std::f32::consts::PI * 2f32
            } else {
                target_yaw_diff 
            };
            let sign_mul = if correct_direction_yaw_diff < std::f32::consts::PI {
                1.0f32
            } else {
                -1.0f32
            };
            car.wheel_yaw = num::clamp(correct_direction_yaw_diff, 0.0f32, 0.5f32) * sign_mul as f32;

            let yaw_increment = current_speed_mag as f32 / (car.bb_size.height as f32 / 2.0f32)  *  car.wheel_yaw;
            rigid_body.set_angular_velocity(yaw_increment as f64);


            let target_long_speed = car_high_level_controller_state.target_long_speed;
            let speed_increment = CAR_ACC * dt * (target_long_speed as f64 - current_speed_mag).signum();

            let mut car_velocity = Vector2::new(current_speed_mag + speed_increment, 0.0);
            rigid_body.position().rotation.rotate(&mut car_velocity);
            rigid_body.set_linear_velocity(car_velocity);
        }
    }

}
