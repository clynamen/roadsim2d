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
            car.longitudinal_speed = self.target_protagonist_twist.x as f32;
            car.yaw_rate = self.target_protagonist_twist.z_rot as f32;
        }
    }
}
