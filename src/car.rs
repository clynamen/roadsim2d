use cgmath::*;
use rand::Rng;
use conrod::color::*;
use piston_window::*;

use super::color_utils::*;
use super::primitives::*;
use super::sim_id::*;
use super::global_resources::*;
use super::physics::*;
use super::camera::Camera;
use super::node::Node;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use nphysics2d::world::World as PWorld;
use nphysics2d::math::Velocity;


pub fn draw_car(context: Context, graphics: &mut G2d, 
    center: Point2f64, rot: f64, car_size: Size2f64, wheel_rot: f64, color: Color, wheel_base: f64)  {
        // note: some vector must be reversed due to the fact that piston_2d Y points toward bottom of screen
        let reverse_y_center = Point2f64{x: center.x, y: -center.y};
        let reverse_y_rot = -rot;
        let car_center = reverse_y_center ;//- Vec2f64{x: car_size.height/2.0, y: car_size.width/2.0};
        let center = context.transform.trans(car_center.x, car_center.y);
        rectangle( to_rgba(&color, 1.0f32), 
                    [-car_size.height/2.0, 
                    -car_size.width/2.0, 
                    car_size.height, 
                    car_size.width],
                    center.rot_rad(reverse_y_rot),
                    graphics);

        let wheel_width : f64 = 0.3;
        let wheel_height : f64 = 0.6;
        let black = [0.0f32, 0.0f32, 0.0f32, 1.0f32];

        rectangle(black, 
                    [0.0, -wheel_width/2.0, 
                    wheel_height, 
                    wheel_width],
                    center.rot_rad(reverse_y_rot).trans(-wheel_base/2.0+wheel_height/2.0, -car_size.width/2.0+wheel_width),
                    graphics);

        rectangle(black, 
                    [0.0, -wheel_width/2.0, 
                    wheel_height, 
                    wheel_width],
                    center.rot_rad(reverse_y_rot).trans(-wheel_base/2.0+wheel_height/2.0, car_size.width/2.0-wheel_width),
                    graphics);

        rectangle(black, 
                    [0.0, -wheel_width/2.0, 
                    wheel_height, 
                    wheel_width],
                    center.rot_rad(reverse_y_rot).trans(wheel_base/2.0-wheel_height, -car_size.width/2.0+wheel_width).rot_rad(-wheel_rot),
                    graphics);

        rectangle( black, 
                    [0.0,  -wheel_width/2.0, 
                    wheel_height, 
                    wheel_width],
                    center.rot_rad(reverse_y_rot).trans(wheel_base/2.0-wheel_height, car_size.width/2.0-wheel_width).rot_rad(-wheel_rot),
                    graphics);

}

#[derive(Clone, Debug)]
pub struct Car {
    pub id : u64,
    pub wheel_yaw: f32,
    pub wheel_base: f32,
    pub bb_size : Size2f64,
    pub color: Color
}

impl Component for Car {
    type Storage = VecStorage<Self>;
}


impl Car {
}


pub fn random_car(id_provider: &mut IdProvider) -> Car {

    let bb_width : f64 = rand::thread_rng().gen_range(3.0, 6.0);
        
    return Car{
        id: id_provider.next(),
        wheel_yaw: rand::thread_rng().gen_range(-0.05, 0.05),
        wheel_base: bb_width as f32 /4.0f32*3.0f32,
        bb_size : Size2f64::new(bb_width/2.0, bb_width),
        color: random_color()
    }
}

pub struct RenderCarSys<'a> {
    pub fps_window: &'a mut PistonWindow,
    pub render_event: &'a Event,
    pub render_args:  RenderArgs, 
}


impl<'a, 'b> System<'a> for RenderCarSys<'b> {
    type SystemData = (ReadStorage<'a, Node>, ReadStorage<'a, Car>, ReadExpect<'a, Camera>);

    fn run(&mut self, (nodes, cars, camera): Self::SystemData) {
        use specs::Join;

        self.fps_window.draw_2d(self.render_event, |context, graphics| {
            let mut context = context;
            let new_trans = camera.apply(context.transform);
            context.transform = new_trans;

            for (node, car) in (&nodes, &cars).join() {
                // println!("node {:?} {:?}", node.pose.center, node.pose.yaw);
                draw_car(context, graphics,
                    node.pose.center, node.pose.yaw, 
                    car.bb_size, car.wheel_yaw as f64, car.color, car.wheel_base as f64);
            }

        });

    }
}

pub struct UpdateCarsSys<'a> {
    pub physics_world: &'a mut PWorld<f64>
}

impl<'a, 'b> System<'a> for UpdateCarsSys<'b> {
    type SystemData = (ReadExpect<'a, UpdateDeltaTime>, WriteStorage<'a, PhysicsComponent>, ReadStorage<'a, Car>);

    fn run(&mut self, (update_delta_time, mut physics_components, mut cars): Self::SystemData) {
        for (physics_component, car) in (&mut physics_components, & cars).join() {

            let mut rigid_body = self.physics_world.rigid_body_mut(physics_component.body_handle).expect("car rigid body not found");
            let pos = rigid_body.position().translation.vector;
            let rot = rigid_body.position().rotation;
            let vel = rigid_body.velocity();

            let rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(rot.unwrap().re));
            let yaw_increment = vel.linear.x as f32 / (car.bb_size.height as f32 / 2.0f32)  *  car.wheel_yaw;


            rigid_body.set_angular_velocity(yaw_increment as f64);
        }
    }

}