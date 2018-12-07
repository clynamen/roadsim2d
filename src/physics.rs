extern crate nphysics2d;
extern crate nalgebra as na;

use nphysics2d::object::BodyHandle;

use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use nphysics2d::object::RigidBody;
use nphysics2d::world::World as PWorld;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use na::{Isometry2, Point2, Vector2};
use nphysics2d::volumetric::Volumetric;

use super::car::*;
use super::node::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct PhysicsComponent {
    pub body_handle: BodyHandle
}

pub fn make_physics_for_car(world: &mut PWorld<f64>, car: &Car) -> PhysicsComponent {
    let cuboid = ncollide2d::shape::Cuboid::new(Vector2::new(car.bb_size.height, car.bb_size.width));
    let geom = ShapeHandle::new(cuboid);
    let pos = Isometry2::new(Vector2::new(0.0, 0.0), 0.0);
    let inertia = geom.inertia(1.0);
    let center_of_mass = geom.center_of_mass();

    let handle = world.add_rigid_body(pos, inertia, center_of_mass);
    PhysicsComponent{body_handle: handle}
}

pub struct PhysicsUpdateNodeSys;

impl <'a> System<'a> for PhysicsUpdateNodeSys {
    type SystemData = (
        ReadStorage<'a, PhysicsComponent>,
        WriteStorage<'a, Node>,
    );

    fn run(&mut self, (physics_component, mut node): Self::SystemData) {
        // let pos = nalgebra::translation(physics_component.body.position());
        // let rot = nalgebra::rotation(physics_component.body.position());
    }
}
