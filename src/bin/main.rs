#![feature(duration_as_u128)]
#![feature(fn_traits)]
#![feature(unboxed_closures)] 
#[macro_use]
extern crate specs_derive;

// #[macro_use]
// extern crate rosrust;

extern crate piston_window;
extern crate piston;
extern crate rand;
extern crate euclid;
extern crate conrod;
extern crate specs;


extern crate roadsim2dlib;

use roadsim2dlib::*;

use std::time;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::collections::HashSet;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};


#[derive(Default)]
pub struct Twist2D {
    x: f64,
    y: f64,
    z_rot: f64
}


struct UpdateGridSys;


impl<'a> System<'a> for UpdateGridSys   {
    type SystemData = (
        ReadExpect<'a, UpdateDeltaTime>, 
        ReadExpect<'a, Camera>,
        ReadExpect<'a, InputState>,
        WriteExpect<'a, Grid>,
    );

    fn run(&mut self, (update_delta_time, camera, input_state, mut grid): Self::SystemData) {
        grid.update(&input_state.buttons_held);
        grid.set_reference_zoom_level(camera.get_zoom_level());
    }
}



pub struct UpdateCameraSys<'a> {
    window_size: piston_window::Size,
    camera_key_mapping: &'a mut KeyActionMapper<Camera>
} 

impl <'a, 'b> System<'a> for UpdateCameraSys<'b> {
    type SystemData = (
        WriteExpect<'a, InputState>,
        ReadExpect<'a, UpdateDeltaTime>, 
        WriteExpect<'a, Camera>,
        ReadStorage<'a, Car>, 
        ReadStorage<'a, ProtagonistTag>, 
    );


    fn run(&mut self, (mut input_state, update_delta_time, mut camera, cars, protagonists): Self::SystemData) {
        camera.update_cam(update_delta_time.dt, &input_state.buttons_held, self.window_size);
        self.camera_key_mapping.process_buttons(&input_state.buttons_held, &mut camera);

        for (car, protagonist) in (&cars, &protagonists).join() {
            camera.set_target_trals(car.pose.center);
        }

    }

}

pub struct SpawnNewCarSys<'a> {
    vehicle_mgr: &'a mut VehicleManager
}


impl <'a, 'b> System<'a> for SpawnNewCarSys<'b> {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, InputState>,
        Read<'a, LazyUpdate>
    );

    fn run(&mut self, (entities, mut input_state, updater): Self::SystemData) {
        if input_state.buttons_pressed.contains(&piston_window::Button::Keyboard(piston_window::Key::K)) {
            let new_entity = entities.create();
            let new_car = self.vehicle_mgr.spawn_random_close_to_protagonist();
            updater.insert(
                new_entity,
                new_car
            );
        }
    }
}

pub struct ControlProtagonistSys<'a> {
   target_protagonist_twist: &'a Twist2D
}

impl <'a, 'b> System<'a> for ControlProtagonistSys<'b> {
    type SystemData = (
        WriteStorage<'a, Car>,
        ReadStorage<'a, ProtagonistTag>,
    );

    fn run(&mut self, (mut cars, protagonists): Self::SystemData) {
        for (car, _protagonist) in (&mut cars, &protagonists).join() {
            car.longitudinal_speed = self.target_protagonist_twist.x as f32;
            car.yaw_rate = self.target_protagonist_twist.z_rot as f32;
        }
    }
}

pub struct IbeoSensorSys<'a> {
    vehicle_state_listeners : &'a mut Vec<Box<VehicleStatesListener>>
}

impl <'a, 'b> System<'a> for IbeoSensorSys<'b> {
    type SystemData = (
        ReadStorage<'a, Car>,
        ReadStorage<'a, ProtagonistTag>,
    );

    fn run(&mut self, (mut cars, protagonists): Self::SystemData) {
        let mut other_cars = Vec::<&Car>::new(); 
        for (car, ()) in (&cars, !&protagonists).join() {
            other_cars.push(car);
        }

        for (car, _protagonist) in (&cars, &protagonists).join() {
            let protagonist_car = car;
            for listener in &mut (self.vehicle_state_listeners).iter_mut() {
                listener.on_vehicle_states(protagonist_car, &other_cars);
                listener.on_protagonist_state(protagonist_car);
            }
        }

    }
}


#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct ProtagonistTag;


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

    let mut vehicle_manager_key_mapping = build_key_mapping_for_vehicle_manager();
    let mut camera_key_mapping = build_key_mapping_for_camera_manager();

    let mut fps_window = window.max_fps(30);


    let mut world = World::new();

    world.register::<Car>();
    world.register::<Camera>();
    world.register::<Grid>();
    world.register::<ProtagonistTag>();

    world.add_resource(InputEvents::new());
    world.add_resource(InputState::new());
    world.add_resource(UpdateDeltaTime { dt: 1.0 });
    world.add_resource(grid);

    let protagonist_car = vehicle_mgr.make_protagonist_car();

    world.create_entity().with(protagonist_car).with(ProtagonistTag{}).build();
    world.add_resource(camera);

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

            UpdateInputStateSys{}.run_now(&mut world.res);

            UpdateCameraSys{window_size, camera_key_mapping: &mut camera_key_mapping}.run_now(&mut world.res);
            UpdateGridSys{}.run_now(&mut world.res);

            SpawnNewCarSys{vehicle_mgr: &mut vehicle_mgr}.run_now(&mut world.res);
            let target_protagonist_twist_locked = target_protagonist_twist.lock().unwrap();
            ControlProtagonistSys{target_protagonist_twist: &target_protagonist_twist_locked}.run_now(&mut world.res);
            UpdateCarsSys.run_now(&mut world.res);
            IbeoSensorSys{vehicle_state_listeners: &mut vehicle_state_listeners}.run_now(&mut world.res);

            // rosrust::sleep(rosrust::Duration::from_nanos(1e6 as i64 ));
        }

        if let Some(_args) = e.render_args() {

            fps_window.draw_2d(&e, |context, graphics| {
                clear([1.0; 4], graphics);
            });
            RenderGridSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            RenderCarSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            world.maintain();

            // for listener in &mut vehicle_state_listeners {
            //     let cars = vehicle_mgr.get_non_playable_vehicles();
            //     let protagonist_car = vehicle_mgr.get_protagonist_vehicle();
            //     listener.on_vehicle_states(protagonist_car, Box::new(cars.iter()));
            //     listener.on_protagonist_state(protagonist_car);
            // }
            // if (now-previous_msg_stamp).as_secs() >= 1 {
            // }
            // previous_frame_end_timestamp = now;
        }
    }

}
