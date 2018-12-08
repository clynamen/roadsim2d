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
    center: Point2f64, rot: f64, car_size: Size2f64, wheel_rot: f64, color: Color)  {
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
                    center.rot_rad(reverse_y_rot).trans(car_size.height/2.0-wheel_height, -car_size.width/2.0+wheel_width).rot_rad(-wheel_rot),
                    graphics);

        rectangle( black, 
                    [0.0,  -wheel_width/2.0, 
                    wheel_height, 
                    wheel_width],
                    center.rot_rad(reverse_y_rot).trans(car_size.height/2.0-wheel_height, car_size.width/2.0-wheel_width).rot_rad(-wheel_rot),
                    graphics);

}

#[derive(Clone, Debug)]
pub struct Car {
    pub id : u64,
    pub pose : Pose2DF64,     
    pub wheel_yaw: f32,
    pub target_longitudinal_speed : f32, 
    pub yaw_rate: f32,
    pub bb_size : Size2f64,
    pub color: Color
}

impl Component for Car {
    type Storage = VecStorage<Self>;
}


impl Car {
    pub fn update(self: &mut Car, dt: f64) {
        // let rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(self.pose.yaw));
        // let ds  = Vector2{x: (self.longitudinal_speed as f64) * dt, y: 0.0};
        // let rotated_ds = rot.rotate_vector(ds);
        // self.pose.center += rotated_ds;

        // let yaw_increment = self.longitudinal_speed / (self.bb_size.height as f32 / 2.0f32)  *  self.wheel_yaw;

        // // let direction_to_center = Vector2{x:400.0, y:400.0} - self.pose.center.to_vec();
        // self.pose.yaw += (yaw_increment as f64 * dt) as f64;
        // self.pose.yaw %= 2.0*std::f64::consts::PI;
    }
}


pub fn random_car(id_provider: &mut IdProvider) -> Car {

    let bb_width = rand::thread_rng().gen_range(3.0, 6.0);
        
    return Car{
        id: id_provider.next(),
        pose: Pose2DF64 {
            center: Point2f64{
            x: rand::thread_rng().gen_range(-400.0, 400.0), 
            y: rand::thread_rng().gen_range(-400.0, 400.0)}, 
            yaw: 1.0
        }, 
        wheel_yaw: rand::thread_rng().gen_range(-0.3, 0.3),
        target_longitudinal_speed: rand::thread_rng().gen_range(5.0, 10.0), 
        yaw_rate: 0.0,
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
                    car.bb_size, car.wheel_yaw as f64, car.color);
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
            // node.pose.center.x = pos.x;
            // node.pose.center.y = pos.y;
            // node.pose.yaw = rot.unwrap().re;
            // car.update(update_delta_time.dt);

            let rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(rot.unwrap().re));
            // let ds  = Vector2{x: (self.longitudinal_speed as f64) * dt, y: 0.0};
            // let ds  = Vector2{x: (vel.translation.x as f64) * dt, y: 0.0};
            // let rotated_ds = rot.rotate_vector(ds);

            // self.pose.center += rotated_ds;

            let yaw_increment = vel.linear.x as f32 / (car.bb_size.height as f32 / 2.0f32)  *  car.wheel_yaw;


            rigid_body.set_velocity(Velocity::linear(car.target_longitudinal_speed as f64, 0.0));
            rigid_body.set_velocity(Velocity::angular(yaw_increment as f64));


            // let direction_to_center = Vector2{x:400.0, y:400.0} - self.pose.center.to_vec();
            // self.pose.yaw += (yaw_increment as f64 * dt) as f64;
            // self.pose.yaw %= 2.0*std::f64::consts::PI;

        }
    }

}