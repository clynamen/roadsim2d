#![feature(duration_as_u128)]
#![feature(fn_traits)]
#![feature(unboxed_closures)] 
extern crate piston_window;
extern crate piston;
extern crate rand;
extern crate euclid;
extern crate conrod;
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
mod vehicle_manager;
mod debouncer;

use std::time;
use piston_window::*;
use self::camera::*;
use self::primitives::*;
use self::simulation::Simulation;
use self::car::*;
use self::ibeo::*;
use self::grid::*;
use self::sim_id::*;
use self::vehicle_manager::*;


fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("carsim2D", [640, 480])
        .exit_on_esc(true).build().expect("Unable to create piston application");

    let id_provider = Box::new(IdProvider::new());

    let mut vehicle_mgr = VehicleManager::new(id_provider);
    
    let mut vehicle_state_listeners : Vec<Box<VehicleStatesListener>> = Vec::new();

    let ibeo_publisher = IbeoPublisher::try_new(); 
    if ibeo_publisher.is_some() {
        vehicle_state_listeners.push(Box::new(ibeo_publisher.unwrap()));
        println!("Added ROS publisher");
    } else {
        println!("Could not start ROS publisher");
    }

    let mut previous_frame_end_timestamp = time::Instant::now();
    let previous_msg_stamp = time::Instant::now();

    let mut grid = Grid{ enabled: false};
    let mut camera = Camera::new( Vec2f64{x: 0.0, y: 0.0}, 40.0);

    // for e in window.events().ups(60).max_fps(60) {
    let mut simulation = Simulation::new();

    let mut vehicle_manager_key_mapping = VehicleManagerKeyMapping::new();

    let mut fps_window = window.max_fps(30);
    while let Some(e) = fps_window.next() {

            if let Some(args) = e.press_args() {
                simulation.key_press(args);

            }

            if let Some(args) = e.release_args() {
                simulation.key_release(args);
            }

            if let Some(args) = e.update_args() {
                grid.update(simulation.get_buttons());
                vehicle_mgr.process_buttons(&mut vehicle_manager_key_mapping, simulation.get_buttons());
                camera.set_target_trals(vehicle_mgr.get_protagonist_vehicle().pose.center);
                simulation.update_camera(&mut camera, args.dt, fps_window.draw_size());
            }

            if let Some(_args) = e.render_args() {
                let now = time::Instant::now();
                let dt = now-previous_frame_end_timestamp;
                let dt_s = (dt.as_millis() as f32)/1000.0f32;

                let protagonist_car = vehicle_mgr.get_protagonist_vehicle();

                fps_window.draw_2d(&e, |context, graphics| {
                    clear([1.0; 4], graphics);
                    let mut context = context;
                    let new_trans = camera.apply(context.transform);
                    context.transform = new_trans;
                    grid.draw(context, graphics);
                    draw_car(context, graphics,
                        protagonist_car.pose.center, protagonist_car.pose.yaw,
                        protagonist_car.bb_size, protagonist_car.color);
                    // println!("Protagonist car pose: {:?}", protagonist_car.pose);
                    let cars = vehicle_mgr.get_non_playable_vehicles();
                    for car in cars.iter() {
                        draw_car(context, graphics,
                            car.pose.center, car.pose.yaw, 
                            car.bb_size, car.color);
                    }
                });

                // &mut protagonist_car.update(dt_s, false);
                // for car in &mut cars.iter() {
                //     car.update(dt_s, true);
                // }
                vehicle_mgr.update(dt_s);

                for listener in &mut vehicle_state_listeners {
                    let cars = vehicle_mgr.get_non_playable_vehicles();
                    let protagonist_car = vehicle_mgr.get_protagonist_vehicle();
                    listener.on_vehicle_states(protagonist_car, Box::new(cars.iter()));
                    listener.on_protagonist_state(protagonist_car);
                }
                if (now-previous_msg_stamp).as_secs() >= 1 {
                    // let mut msg = msg::ibeo_msgs::ObjectListEcu::default();
                }
                previous_frame_end_timestamp = now;
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