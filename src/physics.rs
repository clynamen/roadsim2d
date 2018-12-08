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
use nphysics2d::object::BodyStatus;

use super::car::*;
use super::node::*;
use super::primitives::*;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct PhysicsComponent {
    pub body_handle: BodyHandle
}

const COLLIDER_MARGIN: f64 = 0.00001;

pub fn make_physics_for_car(world: &mut PWorld<f64>, car: &Car, pose: &Pose2DF64) -> PhysicsComponent {
    let cuboid = ncollide2d::shape::Cuboid::new(Vector2::new(car.bb_size.height/2.0, car.bb_size.width/2.0));
    let geom = ShapeHandle::new(cuboid);
    let pos = Isometry2::new(Vector2::new(pose.center.x, pose.center.y), pose.yaw);
    let inertia = geom.inertia(1.0);
    let center_of_mass = geom.center_of_mass();

    let handle = world.add_rigid_body(pos, inertia, center_of_mass);
    world.add_collider(
        COLLIDER_MARGIN,
        geom,
        handle,
        // Isometry2::new(Vector2::new(car.bb_size.width/2.0, car.bb_size.height/2.0), 0.0),
        Isometry2::new(Vector2::new(0.0, 0.0), 0.0),
        // pos,
        Material::default(),
    );
    let mut rigid_body = world.rigid_body_mut(handle).expect("just added rigid body not found");
    // rigid_body.set_status(BodyStatus::Kinematic);


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
