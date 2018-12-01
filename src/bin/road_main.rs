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
extern crate nalgebra;

use roadsim2dlib::*;
use nalgebra as nb;

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

fn vecmath2x3_to_nalgebera2x3(in_mat : &graphics::math::Matrix2d) -> nb::Matrix2x3<f64> {
    let out_map = nb::Matrix2x3::new(
        in_mat[0][0], in_mat[0][1], in_mat[0][2],
        in_mat[1][0], in_mat[1][1], in_mat[1][2], 
    );
    out_map
}

fn nalgebera_to_column_slice<T> (in_mat : &nb::Matrix<f64, nb::U2, nb::Dynamic, T>) ->  Vec<[f32; 2]>
    where T: nalgebra::storage::Storage<f64, nalgebra::U2, nalgebra::Dynamic>
{
    let mut array  = vec![ [0.0f32, 0.0f32]; in_mat.ncols()];
    for column_i in 0..in_mat.ncols() {
        array[column_i][0] = in_mat.column(column_i)[0] as f32;
        array[column_i][1] = in_mat.column(column_i)[1] as f32;
    }
    // let out_map = nb::Matrix2x3::new(
    //     in_mat[0][0], in_mat[0][1], in_mat[0][2],
    //     in_mat[1][0], in_mat[1][1], in_mat[1][2], 
    // );
    array
}

fn main() {
    let window: PistonWindow =
        WindowSettings::new("carsim2D - ROAD", [640, 480])
        .exit_on_esc(true).build().expect("Unable to create piston application");

    let mut id_provider = Box::new(IdProvider::new());

    let mut previous_frame_end_timestamp = time::Instant::now();
    let previous_msg_stamp = time::Instant::now();

    let mut grid = Grid::new();
    let mut camera = Camera::new( Vec2f64{x: 0.0, y: 0.0}, 40.0);

    // for e in window.events().ups(60).max_fps(60) {
    let mut simulation = Simulation::new();

    let mut fps_window = window.max_fps(30);

    let road = generate_random_road(&mut id_provider);

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
                grid.set_reference_zoom_level(camera.get_zoom_level());
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


                    println!("transform {:?}", context.transform);
                    let tmat = vecmath2x3_to_nalgebera2x3(&context.transform);
                    println!("n transform  {} ", tmat);

                    let vertices_mat = nb::DMatrix::<f64>::from_row_slice(3, 6, &[
                        0.0, 2.0, 0.0, 4.0, 5.0, 4.0,
                        0.0, 0.0, 2.0, 4.0, 4.0, 5.0,
                        1.0, 1.0, 1.0, 1.0, 1.0, 1.0
                    ]);

                    let transformed_mat = tmat * &vertices_mat;
                    println!("from {} to {}", vertices_mat, transformed_mat);
                    
                    grid.draw(context, graphics);
                    draw_road(context, graphics, &road);
                    graphics.tri_list(&context.draw_state, &[0.5f32, 0.5f32, 0.5f32, 0.5f32], |f|  {
                        let points = nalgebera_to_column_slice(&transformed_mat);
                        println!("points {:?}", points);
                        f(&points[..]);
                    });
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