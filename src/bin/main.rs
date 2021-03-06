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
extern crate image;
extern crate find_folder;
extern crate roadsim2dlib;

use roadsim2dlib::*;
use std::env;
use std::rc::Rc;

use opengl_graphics::GlGraphics;
use piston_window::{OpenGL, PistonWindow, Size, WindowSettings};

use std::time;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::collections::HashMap;
use std::collections::HashSet;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use nalgebra::Vector2;
use opengl_graphics::GlyphCache;
use nphysics2d::world::World as PWorld;

fn print_commands() {
    let commands = r#"
q:      zoom_out
e:      zoom_in
c:      switch camera mode (move/follow)
k:      spawn vehicle
a:	spawn vehicle that goes up/down
p:	spawn vehicle that goes down/up
g:      hide/show grid
arrows: move camera in 'move' mode
"#;
    println!("{}", commands);
}

fn evaluate_protagonist_car_init_pose()->Pose2DF64{
    let mut scenario : Option<Scenario> = None;

    let all_args : Vec<String> = env::args().collect();
    if all_args.len() > 1 {
        let fname = all_args.get(1).unwrap();
        println!("Loading scenario from {} for car_init_pose", fname);
        let scenario_res = ScenarioLoader::read_from_file(fname);

        if scenario_res.is_ok() {
            scenario = Some(scenario_res.unwrap());
        }
    };

    let init_pose = if scenario.is_some() {
	let protagonist_car_init_pose = scenario.unwrap().protagonist_car_init.unwrap().pose;
        let pose = Pose2DF64{
	    center: Point2f64{
		  x: protagonist_car_init_pose.center.x,
		  y: protagonist_car_init_pose.center.y
	    },
	    yaw: protagonist_car_init_pose.yaw
	};
	println!("Pose x: {}, y: {}, yaw: {}", pose.center.x, pose.center.y, pose.yaw);
	pose
    }
    else {
	println!("Default pose");
        Pose2DF64::default()
    };
    init_pose    
}


fn main() {
    let id_provider = Rc::new(RefCell::new(IdProvider::new()));

    let mut vehicle_mgr = VehicleManager::new(id_provider.clone());
    
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

    let opengl = OpenGL::V3_2;

    let max_fps = 1000;
    let mut fps_window : PistonWindow = WindowSettings::new(
        "roadsim2d",
        [800, 800],
    )
        .opengl(opengl)
        .samples(4)
        .exit_on_esc(true)
        .resizable(true)
        .build()
        .unwrap_or_else(|error| panic!("Failed to build PistonWindow: {}", error));
    // fps_window.set_max_fps(max_fps);
    // fps_window.set_ups(20);

    let mut world = World::new();

    let mut physics_world = PWorld::new();
    physics_world.set_gravity(Vector2::new(0.0, 0.0));


    let mut scenario : Option<Scenario> = None;

    let all_args : Vec<String> = env::args().collect();
    if all_args.len() > 1 {
        let fname = all_args.get(1).unwrap();
        println!("Loading scenario from {} for gridmap", fname);
        let scenario_res = ScenarioLoader::read_from_file(fname);

        if scenario_res.is_ok() {
            scenario = Some(scenario_res.unwrap());
        }
    }

    let gridmap = if scenario.is_some() {
        let town_image = &scenario.as_ref().unwrap().town_image;
        if  town_image.is_some() {
	    println!("Loading image from scenario");
            load_town_from_file(&town_image.as_ref().unwrap().as_str())
        } else {
	    println!("Generating random image");
            make_random_town_gridmap(0)
        }
    } else {
        println!("Generating random image because scenario not ok");
        make_random_town_gridmap(0)
    };

    let gridmap_texture = town_gridmap_to_texture(&mut fps_window, &gridmap);

    let mut simulation_time = 0.0f64;

    world.register::<Car>();
    world.register::<Camera>();
    world.register::<Grid>();
    world.register::<ProtagonistTag>();
    world.register::<roadsim2dlib::Node>();
    world.register::<PhysicsComponent>();
    world.register::<CarController>();
    world.register::<CarHighLevelControllerState>();
    world.register::<CarPathControllerState>();
    world.register::<CarCmdListState>();

    world.add_resource(InputEvents::new());
    world.add_resource(InputState::new());
    world.add_resource(UpdateDeltaTime { dt: 1.0, sim_time: 0.0 });
    world.add_resource(SimInfo::default());
    world.add_resource(IbeoSensorState::new());
    world.add_resource(grid);
    world.add_resource(gridmap);


    let protagonist_car = vehicle_mgr.make_protagonist_car();

    world.create_entity()
        .with(Node{pose: evaluate_protagonist_car_init_pose()})
        .with(make_physics_for_car(&mut physics_world, &protagonist_car, &evaluate_protagonist_car_init_pose()))
        .with(protagonist_car)
        .with(ProtagonistTag{}).build();
    world.add_resource(camera);


    print_commands();
    let mut gl = GlGraphics::new(opengl);

    // let mut fonts = load_font();
    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    println!("{:?}", assets);
    let ref font = assets.join("FiraSans-Regular.ttf");
    let mut fonts = GlyphCache::new(font, (), TextureSettings::new()).expect("unable to load font");
    let mut fps_counter = FPSCounter::new();

    let all_args : Vec<String> = env::args().collect();
    if all_args.len() > 1 {
        let fname = all_args.get(1).unwrap();
        println!("Loading scenario from {} for cars", fname);
        let scenario_res = ScenarioLoader::read_from_file(fname);

        if scenario_res.is_ok() {
            scenario = Some(scenario_res.unwrap());
        }
    }

    if scenario.is_some() {
	println!("Scenario ok, taking the cars..");
        for car in scenario.unwrap().cars {
            CarCmdListController::create_car(&mut world, &mut physics_world, id_provider.clone(), 
            car.pose, car.cmds, car.rgb);
        }
    }

    // if all_args.len() > 1 {
        // let fname = all_args.get(1).unwrap();
        // println!("Loading scenario from {}", fname);
        // let scenario = ScenarioLoader::read_from_file(fname);
        // town_image_fname = scenario.town_image;

        // if scenario.is_ok() {
        //     for car in scenario.unwrap().cars {
        //         CarCmdListController::create_car(&mut world, &mut physics_world, id_provider.clone(), 
        //         car.pose, car.cmds, car.rgb);
        //     }

        // }

    // }


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
                simulation_time += args.dt;
                update_delta_time.dt = args.dt;
                update_delta_time.sim_time = simulation_time;
            };
            let window_size = fps_window.draw_size();

            CarPathControllerSys{}.run_now(&mut world.res);
            CarCmdListSys{}.run_now(&mut world.res);
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
            let () = {
                let mut sim_info = world.write_resource::<SimInfo>();
                sim_info.sim_time = simulation_time;
                sim_info.fps = fps_counter.tick() as f32;
            };

            fps_window.draw_2d(&e, |context, graphics| {
                clear([1.0; 4], graphics);
                let zoom = context.transform.zoom(1.0);
            });

            RenderTownSys{fps_window: &mut fps_window, town_gridmap_texture: &gridmap_texture, render_event: &e, render_args: _args}.run_now(&mut world.res);
            RenderGridSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            RendererCarHighLevelControllerSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            RenderCarSys{fps_window: &mut fps_window, render_event: &e, render_args: _args}.run_now(&mut world.res);
            RenderInfoSys{render_args: _args, font_glyphs: &mut fonts, opengl: &mut gl}.run_now(&mut world.res);
            world.maintain();

        }
    }

}
