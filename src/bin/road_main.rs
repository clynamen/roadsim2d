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

extern crate roadsim2dlib;

use roadsim2dlib::*;

// mod camera;
// mod car;
// mod simulation;
// mod primitives;
// mod color_utils;
// mod ibeo;
// mod sim_id;
// mod grid;
// mod vehicle_manager;
// mod debouncer;
// mod roads;

use std::time;
use piston_window::*;

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("carsim2D - ROAD", [640, 480])
        .exit_on_esc(true).build().expect("Unable to create piston application");

    let id_provider = Box::new(IdProvider::new());

    let mut previous_frame_end_timestamp = time::Instant::now();
    let previous_msg_stamp = time::Instant::now();

    let mut grid = Grid{ enabled: false};
    let mut camera = Camera::new( Vec2f64{x: 0.0, y: 0.0}, 40.0);

    // for e in window.events().ups(60).max_fps(60) {
    let mut simulation = Simulation::new();

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
                simulation.update_camera(&mut camera, args.dt, fps_window.draw_size());
            }

            if let Some(_args) = e.render_args() {
                let now = time::Instant::now();
                let dt = now-previous_frame_end_timestamp;
                let dt_s = (dt.as_millis() as f32)/1000.0f32;


                fps_window.draw_2d(&e, |context, graphics| {
                    clear([1.0; 4], graphics);
                    let mut context = context;
                    let new_trans = camera.apply(context.transform);
                    context.transform = new_trans;
                    grid.draw(context, graphics);
                });

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
}