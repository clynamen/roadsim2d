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
mod car;
mod simulation;
mod primitives;
mod color_utils;
mod ibeo;
mod sim_id;
mod grid;

use std::time;
use piston_window::*;
use piston::event_loop::*;
use cgmath::*;
use rand::distributions::{IndependentSample, Range};
use rand::Rng;
//use rosrust::PublisherStream;
use rosrust::api::raii::Publisher;
use euclid::*;
use conrod::color::*;
use self::camera::*;
use self::primitives::*;
use self::simulation::Simulation;
use self::car::*;
use self::car::*;
use self::color_utils::*;
use self::ibeo::*;
use self::grid::*;
use self::sim_id::*;

use piston_window::context::Context;
use piston_window::G2d;
use piston::input::{Input, Event};
use piston::input::Input::*;



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
                simulation.update_camera(&mut camera, args.dt, window.draw_size());
            }

            if let Some(args) = e.render_args() {
                let now = time::Instant::now();
                let dt = now-previous_frame_end_timestamp;
                let dt_s = (dt.as_millis() as f32)/1000.0f32;

                let viewport = args.viewport();

                window.draw_2d(&e, |context, graphics| {
                    clear([1.0; 4], graphics);
                    let mut context = context;
                    let new_trans = camera.apply(context.transform);
                    context.transform = new_trans;
                    grid.draw(context, graphics);
                    draw_car(context, graphics,
                        protagonist_car.pose.center, protagonist_car.pose.yaw,
                        protagonist_car.bb_size, protagonist_car.color);
                    println!("Protagonist car pose: {:?}", protagonist_car.pose);
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
                for listener in &mut vehicle_state_listeners {
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