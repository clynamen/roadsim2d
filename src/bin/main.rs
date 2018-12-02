#![feature(duration_as_u128)]
#![feature(fn_traits)]
#![feature(unboxed_closures)] 

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
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, ReadExpect, WriteExpect, RunNow};


#[derive(Default)]
pub struct Twist2D {
    x: f64,
    y: f64,
    z_rot: f64
}

struct RenderSys {
    fps_window: PistonWindow 
}

struct RenderCarSys<'a> {
    fps_window: &'a mut PistonWindow,
    render_event: &'a Event,
    render_args:  RenderArgs, 
}

struct RenderGridSys<'a> {
    fps_window: &'a mut PistonWindow,
    render_event: &'a Event,
    render_args:  RenderArgs, 
}

impl<'a, 'b> System<'a> for RenderGridSys<'b> {
    type SystemData = (ReadExpect<'a, Grid>, ReadExpect<'a, Camera>);

    fn run(&mut self, (grid, camera): Self::SystemData) {
        self.fps_window.draw_2d(self.render_event, |context, graphics| {
            let mut context = context;
            let new_trans = camera.apply(context.transform);
            context.transform = new_trans;
            grid.draw(context, graphics);
        });
    }
}

impl<'a, 'b> System<'a> for RenderCarSys<'b> {
    type SystemData = (ReadStorage<'a, Car>, ReadExpect<'a, Camera>);

    fn run(&mut self, (car, camera): Self::SystemData) {
        use specs::Join;

        self.fps_window.draw_2d(self.render_event, |context, graphics| {
            let mut context = context;
            let new_trans = camera.apply(context.transform);
            context.transform = new_trans;

            // grid.draw(context, graphics);
            // draw_car(context, graphics,
            //     protagonist_car.pose.center, protagonist_car.pose.yaw,
            //     protagonist_car.bb_size, protagonist_car.color);

            // let cars = vehicle_mgr.get_non_playable_vehicles();
            for car in car.join() {
                draw_car(context, graphics,
                    car.pose.center, car.pose.yaw, 
                    car.bb_size, car.color);
            }

            // rectangle( [0.5f32, 0.0f32, 1.0f32, 0.5f32], 
            //             [-1.0, 
            //             -1.0, 
            //             200.0, 
            //             200.0],
            //             context.transform,
            //             graphics);

        });

    }
}

struct UpdateCars;

impl<'a> System<'a> for UpdateCars {
    type SystemData = (ReadStorage<'a, Car>);

    fn run(&mut self, (car): Self::SystemData) {
        use specs::Join;

        for car in car.join() {
            println!("update car");
        }
    }
}

impl<'a> System<'a> for RenderSys   {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {


    }
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


// credits to https://github.com/andreivasiliu/stacked-worlds on how to handle specs and input

pub enum InputEvent {
    PressEvent(Button),
    ReleaseEvent(Button),
    MotionEvent(f64, f64),
}


pub struct InputEvents {
    pub events: VecDeque<InputEvent>,
}

impl InputEvents {
    pub fn new() -> Self {
        InputEvents {
            events: VecDeque::with_capacity(32),
        }
    }
}

pub struct InputState {
    pub buttons_held: HashSet<piston_window::Button>
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            buttons_held: HashSet::new(),
        }
    }

    pub fn button_press(&mut self, button: piston_window::Button) {
        self.buttons_held.insert(button);
    }

    pub fn button_release(&mut self, button: piston_window::Button) {
        self.buttons_held.remove(&button);
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
    );


    fn run(&mut self, (mut input_state, update_delta_time, mut camera): Self::SystemData) {
        camera.update_cam(update_delta_time.dt, &input_state.buttons_held, self.window_size);
        self.camera_key_mapping.process_buttons(&input_state.buttons_held, &mut camera);
    }

}

pub struct HandleInputEventSys;

pub struct UpdateDeltaTime {
    pub dt: f64,
}


impl <'a> System<'a> for HandleInputEventSys {
    type SystemData = (
        WriteExpect<'a, InputEvents>,
        WriteExpect<'a, InputState>,
    );

    fn run(&mut self, (mut input_events, mut input_state): Self::SystemData) {
        while let Some(input_event) = input_events.events.pop_front() {
            match input_event {
                InputEvent::PressEvent(button) => {
                    input_state.button_press(button);
                },
                InputEvent::ReleaseEvent(button) => {
                    input_state.button_release(button);

                },
                InputEvent::MotionEvent(x, y) => {

                },
            };

        }
    }
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

    // let protagonist_twist_subscriber = rosrust::subscribe("roadsim2d/protagonist_twist", move |v: msg::geometry_msgs::Twist| {
    //     target_protagonist_twist_locked.x = v.linear.x;
    //     target_protagonist_twist_locked.z_rot = v.angular.z;
    // }).unwrap();

    let mut previous_frame_end_timestamp = time::Instant::now();
    let previous_msg_stamp = time::Instant::now();

    let mut grid = Grid::new();
    let mut camera = Camera::new( Vec2f64{x: 0.0, y: 0.0}, 40.0);

    // for e in window.events().ups(60).max_fps(60) {
    let mut simulation = Simulation::new();

    //let mut vehicle_manager_key_mapping = VehicleManagerKeyMapping::new();
    let mut vehicle_manager_key_mapping = build_key_mapping_for_vehicle_manager();
    let mut camera_key_mapping = build_key_mapping_for_camera_manager();

    let mut fps_window = window.max_fps(30);


    let mut world = World::new();
    world.register::<Car>();
    world.register::<Camera>();
    world.register::<Grid>();
    // world.register::<Position>();
    world.add_resource(InputEvents::new());
    world.add_resource(InputState::new());
    world.add_resource(UpdateDeltaTime { dt: 1.0 });
    world.add_resource(grid);

    let protagonist_car = vehicle_mgr.make_protagonist_car();

    world.create_entity().with(protagonist_car).build();
    world.add_resource(camera);



    // world.create_entity().with(Position { x: 4.0, y: 7.0 }).build();
    // let mut dispatcher = DispatcherBuilder::new()
    //     .with(UpdateCars, "draw_car", &[])
    //     .with_thread_local(RenderSys{fps_window})
    //     .build();


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
            UpdateCameraSys{window_size, camera_key_mapping: &mut camera_key_mapping}.run_now(&mut world.res);
            UpdateGridSys{}.run_now(&mut world.res);
            // rosrust::sleep(rosrust::Duration::from_nanos(1e6 as i64 ));
            // grid.update(simulation.get_buttons());

            // let target_protagonist_twist_locked = target_protagonist_twist.lock().unwrap();
            // vehicle_mgr.set_protagonist_speed(
            //     target_protagonist_twist_locked.x, 
            //     target_protagonist_twist_locked.z_rot
            //     );


            // vehicle_manager_key_mapping.process_buttons(simulation.get_buttons(), &mut vehicle_mgr);
            // camera_key_mapping.process_buttons(simulation.get_buttons(), &mut camera);

            // grid.set_reference_zoom_level(camera.get_zoom_level());
            // camera.set_target_trals(vehicle_mgr.get_protagonist_vehicle().pose.center);
            // simulation.update_camera(&mut camera, args.dt, fps_window.draw_size());
        }

        if let Some(_args) = e.render_args() {

            fps_window.draw_2d(&e, |context, graphics| {
                clear([1.0; 4], graphics);
            });
            RenderGridSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            RenderCarSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            world.maintain();
            // let now = time::Instant::now();
            // let dt = now-previous_frame_end_timestamp;
            // let dt_s = (dt.as_millis() as f32)/1000.0f32;

            // let protagonist_car = vehicle_mgr.get_protagonist_vehicle();

            // fps_window.draw_2d(&e, |context, graphics| {
            //     clear([1.0; 4], graphics);
            //     let mut context = context;
            //     let new_trans = camera.apply(context.transform);
            //     context.transform = new_trans;
            //     grid.draw(context, graphics);
            //     draw_car(context, graphics,
            //         protagonist_car.pose.center, protagonist_car.pose.yaw,
            //         protagonist_car.bb_size, protagonist_car.color);
            //     let cars = vehicle_mgr.get_non_playable_vehicles();
            //     for car in cars.iter() {
            //         draw_car(context, graphics,
            //             car.pose.center, car.pose.yaw, 
            //             car.bb_size, car.color);
            //     }
            // });

            // vehicle_mgr.update(dt_s);

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
