use specs::{World, Builder, System, VecStorage, Component, ReadStorage, WriteStorage, ReadExpect, Join};
use std::collections::VecDeque;

use std::rc::Rc;
use std::cell::RefCell;
use super::primitives::*;
use super::car::*;
use conrod::color::*;
use super::node::*;
use super::primitives::*;
use super::time::*;
use super::town::*;
use super::car_hl_controller::*;
use super::physics::*;
use super::sim_id::*;
use super::camera::*;
use super::global_resources::*;
use super::color_utils::*;
use cgmath::InnerSpace;
use cgmath::MetricSpace;
use cgmath::EuclideanSpace;
use super::scenario::*;
use piston_window::*;
use rand::Rng;
use rand;
use std::collections::VecDeque;
use nphysics2d::world::World as PWorld;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CarCmdListState {
    cmd_states = VecDeque<CarActionState>;
}

impl CarCmdListState {
    pub fn new() -> CarCmdListState {
        CarCmdListState {
            cmd_states = VecDeque::new()
        }
    }
}

pub struct CarCmdListSys {
}

impl <'a> System<'a> for CarCmdListSys {
    type SystemData = (
        ReadExpect<'a, UpdateDeltaTime>, 
        ReadStorage<'a, Car>,
        WriteStorage<'a, CarHighLevelControllerState>,
        WriteStorage<'a, CarCmdListState>
    );

    fn run(&mut self, (update_delta_time, cars, mut controller_states, mut cmd_list_states): Self::SystemData) {
        let dt = update_delta_time.dt;
        let sim_time = update_delta_time.sim_time;

        for (car, controller_state, cmd_list_state) in (&cars, &mut controller_states, &mut cmd_list_states).join() {
        }
    }

}

pub struct CarCmdListController {

}

impl CarCmdListController {


pub fn create_car(world: &mut World, mut physics_world: &mut PWorld<f64>, mut id_provider: Rc<RefCell<IdProvider>>) {
    let first_pose = Pose2DF64 {
        center: Point2f64::new(2.0, 2.0),
        yaw: 0.0
    };
    let new_car = Car {
            id: id_provider.borrow_mut().next(),
            wheel_yaw: 0.0,
            wheel_base: 2.5,
            bb_size: Size2f64::new(1.5, 3.0),
            color: rgb(1.0, 0.0, 1.0),
    };

    world.create_entity()
        .with(make_physics_for_car(&mut physics_world, &new_car, &first_pose))
        .with(Node{pose: first_pose})
        .with(new_car)
        .with(CarCmdListState::new())
        .build();

}

}
