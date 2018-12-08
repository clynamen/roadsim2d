use super::primitives::*;
use super::physics::*;
use super::node::Node;
use super::car::Car;
use super::global_resources::*;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use nphysics2d::world::World as PWorld;
use nphysics2d::math::Velocity;
use nphysics2d::math::Force;
use nalgebra::Vector2;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CarController {
    // pub target_velocity: f32
}


pub struct CarControllerSys<'a> {
   pub physics_world: &'a mut PWorld<f64>
}

impl <'a, 'b> System<'a> for CarControllerSys<'b> {
    type SystemData = (
        ReadExpect<'a, UpdateDeltaTime>, 
        ReadStorage<'a, CarController>,
        WriteStorage<'a, PhysicsComponent>,
        WriteStorage<'a, Car>,
    );

    fn run(&mut self, (update_delta_time, car_controllers, mut physics_components, mut cars): Self::SystemData) {
        let dt = update_delta_time.dt;

        for (car_controller, physics_component, car) in (&car_controllers, &mut physics_components, &mut cars).join() {
            let mut rigid_body = self.physics_world.rigid_body_mut(physics_component.body_handle).expect("car rigid body not found");
            let current_speed = rigid_body.velocity();

            let current_speed_mag = current_speed.linear.norm();


            let yaw_increment = current_speed_mag as f32 / (car.bb_size.height as f32 / 2.0f32)  *  car.wheel_yaw;
            rigid_body.set_angular_velocity(yaw_increment as f64);

            let mut car_velocity = Vector2::new(current_speed_mag, 0.0);
            rigid_body.position().rotation.rotate(&mut car_velocity);
            rigid_body.set_linear_velocity(car_velocity);
        }
    }

}
