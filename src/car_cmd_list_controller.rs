use specs::{World, Builder, System, VecStorage, Component, ReadStorage, WriteStorage, ReadExpect, Join};

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
use super::car_controller::*;
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
    cmd_states : VecDeque<CarActionState>
}

impl CarCmdListState {
    pub fn new() -> CarCmdListState {
        CarCmdListState {
            cmd_states: VecDeque::new()
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
            if cmd_list_state.cmd_states.len() > 0 {
                let next_cmd = cmd_list_state.cmd_states.front().unwrap();
                if sim_time > next_cmd.stamp {
                    controller_state.target_yaw = next_cmd.yaw;
                    controller_state.target_long_speed = next_cmd.lon_vel;
                    cmd_list_state.cmd_states.pop_front();
                }
            }
        }
    }

}

pub struct CarCmdListController {

}

impl CarCmdListController {


pub fn create_car(world: &mut World, mut physics_world: &mut PWorld<f64>, 
    mut id_provider: Rc<RefCell<IdProvider>>, first_pose: Pose2DF64, mut cmd_states : VecDeque<CarActionState>) {

    let new_car = Car {
            id: id_provider.borrow_mut().next(),
            wheel_yaw: 0.0,
            wheel_base: 2.5,
            bb_size: Size2f64::new(1.5, 3.0),
            color: rgb(1.0, 0.0, 1.0),
    };

    let mut hl_control_state = CarHighLevelControllerState {
        target_yaw: 0f32,
        target_long_speed: 0f32
    };

    if cmd_states.len() > 0 {
        let first_state = cmd_states.pop_front().unwrap();
        hl_control_state.target_yaw = first_state.yaw;
        hl_control_state.target_long_speed = first_state.lon_vel;
    }

    world.create_entity()
        .with(make_physics_for_car(&mut physics_world, &new_car, &first_pose))
        .with(Node{pose: first_pose})
        .with(new_car)
        .with(CarController{})
        .with(CarCmdListState{cmd_states: cmd_states})
        .with(hl_control_state)
        .build();

}

}
