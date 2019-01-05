use specs::{System, VecStorage, Component, ReadStorage, WriteStorage, ReadExpect, Join};
use std::collections::VecDeque;

use super::primitives::*;
use super::car::*;
use super::node::*;
use super::time::*;
use super::town::*;
use super::camera::*;
use super::global_resources::*;
use super::color_utils::*;
use cgmath::InnerSpace;
use cgmath::MetricSpace;
use cgmath::EuclideanSpace;
use piston_window::*;
use rand::Rng;
use rand;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CarHighLevelControllerState {
    pub destination_point: Vec2f32,
    pub path: VecDeque<Vec2f32>,
    pub target_yaw: f32
}

impl CarHighLevelControllerState {
    pub fn new() -> CarHighLevelControllerState {
        CarHighLevelControllerState {
            destination_point: Vec2f32::new(0f32, 0f32),
            path: VecDeque::new(),
            target_yaw: 0f32
        }
    }
}

pub struct CarHighLevelControllerSys {
}

const TARGET_LIMIT :f32 = 1.0e2f32;

impl <'a> System<'a> for CarHighLevelControllerSys {
    type SystemData = (
        ReadExpect<'a, UpdateDeltaTime>, 
        ReadExpect<'a, TownGridMap>, 
        ReadStorage<'a, Node>,
        ReadStorage<'a, Car>,
        WriteStorage<'a, CarHighLevelControllerState>
    );

    fn run(&mut self, (update_delta_time, town_gridmap, nodes, cars, mut controller_states): Self::SystemData) {
        let dt = update_delta_time.dt;

        for (node, car, controller_state) in (&nodes, &cars, &mut controller_states).join() {
            let mut destination_point = &mut controller_state.destination_point;
            let mut controller_state_path = &mut controller_state.path;
            let car_center = vec2f64_2_vec2f32(node.pose.center.to_vec());

            let distance2_target = destination_point.distance2(car_center);

            let mut rng = rand::thread_rng();

            if(distance2_target < 10f32 || *destination_point == Vec2f32::new(0f32, 0f32) ) {

                let random_destination_point = Vec2f32::new(
                    rng.gen_range(-TARGET_LIMIT, TARGET_LIMIT), 
                    rng.gen_range(-TARGET_LIMIT, TARGET_LIMIT));
                match find_free_space_close_to(&town_gridmap, random_destination_point)  {
                    Some(point) => {
                        *destination_point = point;
                        let shortest_path_opt = find_shortest_path(&town_gridmap, car_center, *destination_point);
                        match shortest_path_opt {
                            Some(shortest_path) => {
                                println!("found shortest path");
                                *controller_state_path = shortest_path;
                            },
                            None => {

                            }
                        }
                    },
                    None => *destination_point = Vec2f32::new(0.0f32, 0.0f32)
                }


                
            }


            let direction_yaw = if(controller_state_path.len() > 0) {
                0f32
            } else {
                let mut next_step_point = controller_state_path.front().unwrap();
                let angle = Vec2f32::unit_x().angle(next_step_point - car_center).0;

                let distance2_next_point = next_step_point.distance2(car_center);
                if(distance2_next_point < 1.0f32) {
                    controller_state_path.pop_front();
                }

                angle
            };


            controller_state.target_yaw = direction_yaw;
            // controller_state.destination_point = destination_point;
        }
    }

}

pub struct RendererCarHighLevelControllerSys<'a> {
    pub fps_window: &'a mut PistonWindow,
    pub render_event: &'a Event,
    pub render_args:  RenderArgs, 
}


const PATH_POINT_SIZE : f64  = 0.2f64;

impl <'a, 'b> System<'a> for RendererCarHighLevelControllerSys<'b> {
    type SystemData = (
        ReadStorage<'a, Car>,
        WriteStorage<'a, CarHighLevelControllerState>,
        ReadExpect<'a, Camera>
    );

    fn run(&mut self, (cars, mut controller_states, camera): Self::SystemData) {

        self.fps_window.draw_2d(self.render_event, |context, graphics| {
            let mut context = context;
            let new_trans = camera.apply(context.transform);
            context.transform = new_trans;


            for (car, controller_state) in (&cars, &mut controller_states).join() {
                let controller_dest_point = controller_state.destination_point;
                let color  = to_rgba(&car.color, 1.0f32);
                let center = context.transform.trans(
                     controller_dest_point.x as f64, 
                    -controller_dest_point.y as f64);
                ellipse( color, 
                        [-1.0, 
                            -1.0, 
                            1.0, 
                            1.0],
                    center,
                    graphics);

                for point in &controller_state.path {
                    rectangle( color, 
                            [   -PATH_POINT_SIZE, 
                                -PATH_POINT_SIZE, 
                                 PATH_POINT_SIZE, 
                                 PATH_POINT_SIZE],
                        context.transform.trans(point.x as f64, -point.y as f64),
                        graphics);

                }
            }

        });

    }

}
