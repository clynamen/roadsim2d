#![feature(duration_as_u128)]
extern crate piston_window;
extern crate piston;
extern crate cgmath;
extern crate rand;
extern crate euclid;
extern crate conrod;
use std::collections::HashSet;
use piston::input;

#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

rosmsg_include!();

mod camera;
mod simulation;
mod primitives;

use std::time;
use piston_window::*;
use cgmath::*;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;
use piston::event_loop::*;
//use rosrust::PublisherStream;
use rosrust::api::raii::Publisher;
use euclid::*;
use conrod::color::*;
use self::camera::*;
use self::primitives::*;
use self::simulation::Simulation;


struct WorldUnit {}

pub fn random_color() -> Color {
    rgb(::rand::random(), ::rand::random(), ::rand::random())
}

fn toRgba(c : &Color, a: f32) -> [f32; 4] {
    [c.red(), c.green(), c.blue(), a]
} 

use piston_window::context::Context;
use piston_window::G2d;
use piston::input::{Input, Event};
use piston::input::Input::*;

fn draw_car(context: Context, graphics: &mut G2d, 
    center: Point2f64, rot: f64, car_size: Size2f64, color: Color)  {
        let car_center = center + Vec2f64{x: car_size.height/2.0, y: car_size.width/2.0};
        let center = context.transform.trans(car_center.x, car_center.y);
        // let square = rectangle::square(0.0, 0.0, 100.0);
        rectangle( toRgba(&color, 1.0f32), // red
                    [-car_size.height/2.0, 
                    -car_size.width/2.0, 
                    car_size.height, 
                    car_size.width],
                    center.rot_rad(rot),
                    graphics);
}

#[derive(Clone, Debug)]
struct Pose2DF64 {
    center: Point2f64,
    yaw: f64
}


#[derive(Clone, Debug)]
struct Car {
    id : u64,
    pose : Pose2DF64,     
    longitudinal_speed : f32, 
    yaw_rate: f32,
    bb_size : Size2f64,
    color: Color
}


impl Car {
    fn update(self: &mut Car, dt: f32, change_yawrate : bool) {
        let rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(self.pose.yaw));
        let ds  = Vector2{x: self.longitudinal_speed as f64, y: 0.0};
        let rotated_ds = rot.rotate_vector(ds);
        self.pose.center += rotated_ds;

        let direction_to_center = Vector2{x:400.0, y:400.0} - self.pose.center.to_vec();
        let direction_rand = rand::thread_rng().gen_range(0.0, 1e-8)*direction_to_center.magnitude2()as f32;
        if(change_yawrate) {
            self.yaw_rate = -direction_to_center.angle(rotated_ds).0.signum() as f32* direction_rand;
        }
        self.pose.yaw += (self.yaw_rate * dt) as f64;
    }
}


struct IbeoPublisher {
    ibeo_vehicle_pub: Publisher<msg::ibeo_msgs::ObjectListEcu>,

}

impl IbeoPublisher {
    fn try_new() -> Option<IbeoPublisher> {
        let ros_not_available_error_msg = "roscore not started or it is not possible to connect to it";
        let ros_init_result = rosrust::try_init("roadsim2d");
        if ros_init_result.is_err() {
            None            
        } else {
            let ibeo_vehicle_pub = rosrust::publish("/roadsim2d/vehicle_ibeo").expect(ros_not_available_error_msg);
            let ibeo_publisher = IbeoPublisher {
                ibeo_vehicle_pub: ibeo_vehicle_pub
            };
            Some(ibeo_publisher)
        }
    }
}

trait VehicleStatesListener { 
    fn on_vehicle_states<'a>(&'a mut self, vehicles : Box<dyn Iterator<Item = &'a Car> + 'a>);
}

impl VehicleStatesListener for IbeoPublisher {

    fn on_vehicle_states<'a>(&'a mut self, vehicles : Box<dyn Iterator<Item = &'a Car> + 'a>) {
        let mut msg = msg::ibeo_msgs::ObjectListEcu::default();
        msg.header.frame_id = String::from("base_link");
        for vehicle in vehicles {
            let mut object_msg = msg::ibeo_msgs::ObjectListEcuObj::default();
            object_msg.id = 0;
            object_msg.bounding_box.pose.x = vehicle.pose.center.x;
            object_msg.bounding_box.pose.y = vehicle.pose.center.y;
            msg.objects.push(object_msg);
        }
        self.ibeo_vehicle_pub.send(msg).unwrap();
    }

}

struct IdProvider {
    last_id : u64,
}

impl IdProvider {
    fn new() -> IdProvider {
        IdProvider { 
            last_id: 0u64
        }
    }
    fn next(&mut self) -> u64 {
        let next_id = self.last_id;
        self.last_id += 1;
        next_id
    }

}

fn random_car(id_provider: &mut IdProvider) -> Car {

    let bb_width = rand::thread_rng().gen_range(100.0, 300.0);
        
    return Car{
        id: id_provider.next(),
        pose: Pose2DF64 {center: Point2f64{
        x: rand::thread_rng().gen_range(-400.0, 400.0), 
        y: rand::thread_rng().gen_range(-400.0, 400.0)}, 
        yaw: 1.0}, 
        longitudinal_speed: 10.0, 
        yaw_rate: 1.0,
        bb_size : Size2f64::new(bb_width/2.0, bb_width),
        color: random_color()
        }
}

struct Grid {
    enabled: bool
}

fn draw_circle<G>(color: [f32; 4], radius: f64, transform: [[f64; 3]; 2], 
    g: &mut G) where G : piston_window::Graphics{

        Ellipse::new(color).resolution(10)
            .draw([10.0, 10.0, 10.0, 10.0], &Default::default(), transform, g);
    // ellipse(color, );
}

impl Grid {
    fn update(&mut self, buttons: &HashSet<input::Button>) {
        macro_rules! if_key {
            ($key:path : $buttons:ident $then:block) => {
                if $buttons.contains(&input::Button::Keyboard($key)) {
                    $then
                }
            };
        }
        if_key! [ Key::G : buttons { self.enabled = !self.enabled; }];
    }

    fn draw(&self, context: Context, graphics: &mut G2d) {
        // let center = context.transform.trans(ix as f64 *100.0, iy as f64 *100.0);
        // let square = rectangle::square(0.0, 0.0, 100.0);
        // draw_circle( [0.25, 0.25, 0.25, 0.5], // red
        if (!self.enabled) {
            return;
        }
         let color = [0.2, 0.2, 0.2, 0.8];
        let grid_size = 16;
        let grid_dist = 100.0;
        let center = context.transform.trans( -grid_size as f64 / 2.0 * grid_dist,
                                              -grid_size as f64 / 2.0 * grid_dist);
        for ix in 0..grid_size {
            for iy in 0..grid_size {
                let center = context.transform.trans(ix as f64 *100.0, iy as f64 *100.0);
                draw_circle( color, // red
                            10.0, 
                            center,
                            graphics);
            }
        }
        // rectangle( color, // red
        //             [-100.0, 
        //             -100.0, 
        //             100.0, 
        //             100.0],
        //             center,
        //             graphics);
    }
}

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true).build().expect("Unable to create piston application");

    let mut id_provider = IdProvider::new();

    // let mut cars = vec![
    //     random_car(),
    //     random_car(),
    //     random_car(),
    // ];
    let mut protagonist_car = Car {
        id: id_provider.next(),
        pose: Pose2DF64 {center: Point2f64{
                x: 0.0, 
                y: 0.0,},
            yaw: 0.0,}, 
        longitudinal_speed: 10.0, 
        yaw_rate: 0.0,
        bb_size : Size2f64::new(50.0, 100.0),
        color: rgb(1.0, 0.0, 1.0),
    };
    let mut cars : Vec<Car> = (0..3).map(|x| random_car( &mut id_provider ) ).collect();
    
    let mut vehicle_state_listeners : Vec<Box<VehicleStatesListener>> = Vec::new();

    let ibeo_publisher = IbeoPublisher::try_new(); 
    if ibeo_publisher.is_some() {
        vehicle_state_listeners.push(Box::new(ibeo_publisher.unwrap()));
        println!("Added ROS publisher");
    } else {
        println!("Could not start ROS publisher");
    }

    let previous_frame_end_timestamp = time::Instant::now();
    let previous_msg_stamp = time::Instant::now();

    let mut grid = Grid{ enabled: true};
    let mut camera = Camera::new( Vec2f64{x: 0.0, y: 0.0}, 1.0);

    // for e in window.events().ups(60).max_fps(60) {
    let mut simulation = Simulation::new();

    while let Some(e) = window.next() {
            // piston::event::Event::Input(args) => {
            //     //game.key_press(button);
            // }

            // piston::event::Event::Input(args) => {
            //     //game.key_release(button);
            // }

            if let Some(args) = e.press_args() {
                simulation.key_press(args);

            }

            if let Some(args) = e.release_args() {
                simulation.key_release(args);
            }

            if let Some(args) = e.update_args() {
                grid.update(simulation.get_buttons());
                camera.set_target_trals(protagonist_car.pose.center);
                simulation.update_camera(&mut camera, args.dt, window.size());
            }

            if let Some(args) = e.render_args() {
                let now = time::Instant::now();
                let dt = now-previous_frame_end_timestamp;
                let dt_s = (dt.as_millis() as f32)/1000.0f32;
                window.draw_2d(&e, |context, graphics| {
                    clear([1.0; 4], graphics);
                    let mut context = context;
                    let new_trans = camera.apply(context.transform);
                    context.transform = new_trans;
                    grid.draw(context, graphics);
                    draw_car(context, graphics,
                        protagonist_car.pose.center, protagonist_car.pose.yaw,
                        protagonist_car.bb_size, protagonist_car.color);
                    for car in &cars {
                        draw_car(context, graphics,
                            car.pose.center, car.pose.yaw, 
                            car.bb_size, car.color);
                    }
                });

                &mut protagonist_car.update(dt_s, false);
                for car in &mut cars {
                    car.update(dt_s, true);
                }
                for  listener in &mut vehicle_state_listeners {
                    listener.on_vehicle_states(Box::new(cars.iter()));
                }
                if (now-previous_msg_stamp).as_secs() >= 1 {
                    // let mut msg = msg::ibeo_msgs::ObjectListEcu::default();
                }
            }

            // Event::Update(args) => {
            //     //game.update(&args);
            //     ;
            // }

    }

    // while let Some(event) = window.next() {

        // Send string message to topic via publisher
        // draw_car(&mut window, &event, Point2f64{x:1.0, y:1.0}, 0.0f64);
        // draw_car(&mut window, &event, Point2f64{x:200.0, y:200.0}, 3.14/2.0f64);
}