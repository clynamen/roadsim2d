#![feature(duration_as_u128)]
extern crate piston_window;
extern crate cgmath;
extern crate rand;
extern crate euclid;
extern crate conrod;

#[macro_use]
extern crate rosrust;
#[macro_use]
extern crate rosrust_codegen;

rosmsg_include!();

use std::time;
use piston_window::*;
use cgmath::*;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;
//use rosrust::PublisherStream;
use rosrust::api::raii::Publisher;
use euclid::*;
use conrod::color::*;

struct WorldUnit {}

type Point2f64 = Point2<f64>;
type Vec2f64 = Vector2<f64>;
type Size2f64 = Size2D<f64>;

pub fn random_color() -> Color {
    rgb(::rand::random(), ::rand::random(), ::rand::random())
}

fn toRgba(c : &Color, a: f32) -> [f32; 4] {
    [c.red(), c.green(), c.blue(), a]
} 

fn draw_car<T>(window: &mut  PistonWindow, event: &T, 
    center: Point2f64, rot: f64, car_size: Size2f64, color: Color
    ) where T:piston_window::GenericEvent {
    window.draw_2d(event, |context, graphics| {
        let car_center = center + Vec2f64{x: car_size.height/2.0, y: car_size.width/2.0};
        let center = context.transform.trans(car_center.x, car_center.y);
        let square = rectangle::square(0.0, 0.0, 100.0);
        rectangle( toRgba(&color, 1.0f32), // red
                    [-car_size.height/2.0, 
                    -car_size.width/2.0, 
                    car_size.height, 
                    car_size.width],
                    center.rot_rad(rot),
                    graphics);
    });
}

#[derive(Clone, Debug)]
struct Pose2DF64 {
    center: Point2f64,
    yaw: f64
}


#[derive(Clone, Debug)]
struct Car {
    pose : Pose2DF64,     
    longitudinal_speed : f32, 
    yaw_rate: f32,
    bb_size : Size2f64,
    color: Color
}



impl Car {
    fn update(self: &mut Car, dt: f32) {
        let rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(self.pose.yaw));
        let ds  = Vector2{x: self.longitudinal_speed as f64, y: 0.0};
        let rotated_ds = rot.rotate_vector(ds);
        self.pose.center += rotated_ds;

        let direction_to_center = Vector2{x:400.0, y:400.0} - self.pose.center.to_vec();
        let direction_rand = rand::thread_rng().gen_range(0.0, 1e-8)*direction_to_center.magnitude2()as f32;
        self.yaw_rate = -direction_to_center.angle(rotated_ds).0.signum() as f32* direction_rand;
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
        if(ros_init_result.is_err()) {
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

fn random_car() -> Car {

    let bb_width = rand::thread_rng().gen_range(100.0, 300.0);
        
    return Car{pose: Pose2DF64 {center: Point2f64{
        x: rand::thread_rng().gen_range(-400.0, 400.0), 
        y: rand::thread_rng().gen_range(-400.0, 400.0)}, 
        yaw: 1.0}, 
        longitudinal_speed: 1.0, 
        yaw_rate: 0.1,
        bb_size : Size2f64::new(bb_width/2.0, bb_width),
        color: random_color()
        }
}

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true).build().expect("Unable to create piston application");

    let mut cars = vec![
        random_car(),
        random_car(),
        random_car(),
    ];
    
    let mut vehicle_state_listeners : Vec<Box<VehicleStatesListener>> = Vec::new();

    let ibeo_publisher = IbeoPublisher::try_new(); 
    if(ibeo_publisher.is_some()) {
        vehicle_state_listeners.push(Box::new(ibeo_publisher.unwrap()));
        println!("Added ROS publisher");
    } else {
        println!("Could not start ROS publisher");
    }

    let previous_frame_end_timestamp = time::Instant::now();
    let previous_msg_stamp = time::Instant::now();

    while let Some(event) = window.next() {
        let now = time::Instant::now();
        let dt = now-previous_frame_end_timestamp;
        let dt_s = (dt.as_millis() as f32)/1000.0f32;
        window.draw_2d(&event, |context, graphics| {
            clear([1.0; 4], graphics);
        });
        for car in &mut cars {
            car.update(dt_s);
        }
        for  listener in &mut vehicle_state_listeners {
            listener.on_vehicle_states(Box::new(cars.iter()));
        }
        for car in &cars {
            draw_car(&mut window, &event, 
                car.pose.center, car.pose.yaw, 
                car.bb_size, car.color);
        }
        if (now-previous_msg_stamp).as_secs() >= 1 {
            // let mut msg = msg::ibeo_msgs::ObjectListEcu::default();
        }

        // Send string message to topic via publisher
        // draw_car(&mut window, &event, Point2f64{x:1.0, y:1.0}, 0.0f64);
        // draw_car(&mut window, &event, Point2f64{x:200.0, y:200.0}, 3.14/2.0f64);
    }
}