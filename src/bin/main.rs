#![feature(duration_as_u128)]
#![feature(fn_traits)]
#![feature(unboxed_closures)] 
#[macro_use]
extern crate specs_derive;
extern crate nphysics2d;

// #[macro_use]
// extern crate rosrust;

extern crate piston_window;
extern crate piston;
extern crate rand;
extern crate euclid;
extern crate conrod;
extern crate specs;
extern crate nalgebra;
extern crate ncollide2d;

use nphysics2d::object::RigidBody;
use nphysics2d::object::BodyHandle;
use nphysics2d::world::World as PWorld;


extern crate roadsim2dlib;

use roadsim2dlib::*;

use std::time;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use  nalgebra::Vector2;

fn print_commands() {
    let commands = r#"
q:      zoom_out
e:      zoom_in
c:      switch camera mode (move/follow)
k:      spawn vehicle
g:      hide/show grid
arrows: move camera in 'move' mode
"#;
    println!("{}", commands);
}

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

    let mut target_protagonist_twist = Arc::new(Mutex::new(Twist2D::default()));
    let mut target_protagonist_twist_clone = target_protagonist_twist.clone();

    let twist_subscriber = TwistSubscriber::new( move |x, z_rot| {
        let mut target_protagonist_twist_locked = target_protagonist_twist_clone.lock().unwrap();
        target_protagonist_twist_locked.x = x;
        target_protagonist_twist_locked.z_rot = z_rot;
    });

    let mut previous_frame_end_timestamp = time::Instant::now();
    let previous_msg_stamp = time::Instant::now();

    let mut grid = Grid::new();
    let mut camera = Camera::new( Vec2f64{x: 0.0, y: 0.0}, 40.0);

    let mut simulation = Simulation::new();

    let mut camera_key_mapping = build_key_mapping_for_camera_manager();

    let mut fps_window = window.max_fps(30);


    let mut world = World::new();

    let mut physics_world = PWorld::new();
    physics_world.set_gravity(Vector2::new(0.0, 0.0));

    world.register::<Car>();
    world.register::<Camera>();
    world.register::<Grid>();
    world.register::<ProtagonistTag>();
    world.register::<roadsim2dlib::Node>();
    world.register::<PhysicsComponent>();
    world.register::<CarController>();

    world.add_resource(InputEvents::new());
    world.add_resource(InputState::new());
    world.add_resource(UpdateDeltaTime { dt: 1.0 });
    world.add_resource(IbeoSensorState::new());
    world.add_resource(grid);


    let protagonist_car = vehicle_mgr.make_protagonist_car();

    world.create_entity()
        .with(Node{pose: Pose2DF64::default() })
        .with(make_physics_for_car(&mut physics_world, &protagonist_car, &Pose2DF64::default()))
        .with(protagonist_car)
        .with(ProtagonistTag{}).build();
    world.add_resource(camera);

    print_commands();

    while let Some(e) = fps_window.next() {
        if let Some(args) = e.press_args() {
            world.write_resource::<InputEvents>().events
                .push_back(InputEvent::PressEvent(args));
            HandleInputEventSys{}.run_now(&mut world.res);
        }

        if let Some(args) = e.release_args() {
            world.write_resource::<InputEvents>().events
                .push_back(InputEvent::ReleaseEvent(args));
            HandleInputEventSys{}.run_now(&mut world.res);
        }

        if let Some(args) = e.update_args() {
            let () = {
                let mut update_delta_time = world.write_resource::<UpdateDeltaTime>();
                update_delta_time.dt = args.dt;
            };
            let window_size = fps_window.draw_size();

            CarControllerSys{physics_world: &mut physics_world}.run_now(&mut world.res);
            let target_protagonist_twist_locked = target_protagonist_twist.lock().unwrap();
            ControlProtagonistSys{physics_world: &mut physics_world, target_protagonist_twist: &target_protagonist_twist_locked}.run_now(&mut world.res);

            physics_world.set_timestep(args.dt);
            physics_world.step();

            PhysicsUpdateNodeSys{physics_world:  &physics_world}.run_now(&mut world.res);
            UpdateInputStateSys{}.run_now(&mut world.res);

            UpdateCameraSys{window_size, camera_key_mapping: &mut camera_key_mapping}.run_now(&mut world.res);
            UpdateGridSys{}.run_now(&mut world.res);

            SpawnNewCarSys{physics_world: &mut physics_world, vehicle_mgr: &mut vehicle_mgr}.run_now(&mut world.res);
            UpdateCarsSys{physics_world: &mut physics_world}.run_now(&mut world.res);
            IbeoSensorSys::new(&mut vehicle_state_listeners, &mut physics_world).run_now(&mut world.res);

        }

        if let Some(_args) = e.render_args() {

            fps_window.draw_2d(&e, |context, graphics| {
                clear([1.0; 4], graphics);
            });
            RenderGridSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            RenderCarSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            world.maintain();

        }
    }

}
