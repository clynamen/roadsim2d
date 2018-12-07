use cgmath::*;
use rand::Rng;
use conrod::color::*;
use piston_window::*;

use super::color_utils::*;
use super::primitives::*;
use super::sim_id::*;
use super::global_resources::*;
use super::camera::Camera;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};


pub fn draw_car(context: Context, graphics: &mut G2d, 
    center: Point2f64, rot: f64, car_size: Size2f64, wheel_rot: f64, color: Color)  {
        // note: some vector must be reversed due to the fact that piston_2d Y points toward bottom of screen
        let reverse_y_center = Point2f64{x: center.x, y: -center.y};
        let reverse_y_rot = -rot;
        let car_center = reverse_y_center + Vec2f64{x: car_size.height/2.0, y: car_size.width/2.0};
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
                    center.rot_rad(reverse_y_rot).trans(car_size.height/2.0-wheel_height, -car_size.width/2.0+wheel_width).rot_rad(wheel_rot),
                    graphics);

        rectangle( black, 
                    [0.0,  -wheel_width/2.0, 
                    wheel_height, 
                    wheel_width],
                    center.rot_rad(reverse_y_rot).trans(car_size.height/2.0-wheel_height, car_size.width/2.0-wheel_width).rot_rad(wheel_rot),
                    graphics);

}

#[derive(Clone, Debug)]
pub struct Car {
    pub id : u64,
    pub pose : Pose2DF64,     
    pub wheel_yaw: f32,
    pub longitudinal_speed : f32, 
    pub yaw_rate: f32,
    pub bb_size : Size2f64,
    pub color: Color
}

impl Component for Car {
    type Storage = VecStorage<Self>;
}


impl Car {
    pub fn update(self: &mut Car, dt: f64) {
        let rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(self.pose.yaw));
        let ds  = Vector2{x: (self.longitudinal_speed as f64) * dt, y: 0.0};
        let rotated_ds = rot.rotate_vector(ds);
        self.pose.center += rotated_ds;

        // let direction_to_center = Vector2{x:400.0, y:400.0} - self.pose.center.to_vec();
        self.pose.yaw += (self.yaw_rate as f64 * dt) as f64;
        self.pose.yaw %= 2.0*std::f64::consts::PI;
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
        wheel_yaw: rand::thread_rng().gen_range(-1.0, 1.0),
        longitudinal_speed: rand::thread_rng().gen_range(5.0, 10.0), 
        yaw_rate: 0.01,
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
    type SystemData = (ReadStorage<'a, Car>, ReadExpect<'a, Camera>);

    fn run(&mut self, (car, camera): Self::SystemData) {
        use specs::Join;

        self.fps_window.draw_2d(self.render_event, |context, graphics| {
            let mut context = context;
            let new_trans = camera.apply(context.transform);
            context.transform = new_trans;

            for car in car.join() {
                draw_car(context, graphics,
                    car.pose.center, car.pose.yaw, 
                    car.bb_size, car.wheel_yaw as f64, car.color);
            }

        });

    }
}

pub struct UpdateCarsSys;

impl<'a> System<'a> for UpdateCarsSys {
    type SystemData = (ReadExpect<'a, UpdateDeltaTime>, WriteStorage<'a, Car>);

    fn run(&mut self, (update_delta_time, mut cars): Self::SystemData) {
        for car in (&mut cars).join() {
            car.update(update_delta_time.dt);
        }
    }

}