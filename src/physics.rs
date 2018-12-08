extern crate nphysics2d;
extern crate nalgebra as na;

use nphysics2d::object::BodyHandle;

use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use nphysics2d::object::RigidBody;
use nphysics2d::object::Material;
use nphysics2d::world::World as PWorld;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use na::{Isometry2, Point2, Vector2};
use nphysics2d::volumetric::Volumetric;
use nphysics2d::math::Velocity;

use super::car::*;
use super::node::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct PhysicsComponent {
    pub body_handle: BodyHandle
}

const COLLIDER_MARGIN: f64 = 0.01;

pub fn make_physics_for_car(world: &mut PWorld<f64>, car: &Car) -> PhysicsComponent {
    let cuboid = ncollide2d::shape::Cuboid::new(Vector2::new(car.bb_size.height, car.bb_size.width));
    let geom = ShapeHandle::new(cuboid);
    let pos = Isometry2::new(Vector2::new(0.0, 0.0), 0.0);
    let inertia = geom.inertia(1.0);
    let center_of_mass = geom.center_of_mass();

    let handle = world.add_rigid_body(pos, inertia, center_of_mass);
    world.add_collider(
        COLLIDER_MARGIN,
        geom,
        handle,
        pos,
        Material::default(),
    );
    let mut rigid_body = world.rigid_body_mut(handle).expect("just added rigid body not found");


    rigid_body.set_velocity(Velocity::linear(car.target_longitudinal_speed as f64, 0.0));
// car.pose.yaw

    PhysicsComponent{body_handle: handle}
}

pub struct PhysicsUpdateNodeSys<'a> {
    pub physics_world: &'a PWorld<f64>
}

impl <'a, 'b> System<'a> for PhysicsUpdateNodeSys<'b> {
    type SystemData = (
        ReadStorage<'a, PhysicsComponent>,
        WriteStorage<'a, Node>,
    );

    fn run(&mut self, (physics_components, mut nodes): Self::SystemData) {
        for (physics_component, node) in (&physics_components, &mut nodes).join() {
            let rigid_body = self.physics_world.rigid_body(physics_component.body_handle).expect("Rigid-body not found.");
            let pos = rigid_body.position().translation.vector;
            let rot = rigid_body.position().rotation;
            node.pose.center.x = pos.x;
            node.pose.center.y = pos.y;
            node.pose.yaw = rot.angle();
        }
    }
}
