use super::car::*;
use super::primitives::*;
use super::sim_id::*;
use cgmath::*;
use conrod::color::*;
// use piston::input::{Button, Key};
use rand::*;
use std::boxed::Box;
use super::input::*;
use super::protagonist::*;
use super::node::*;
use super::physics::*;
use super::primitives::*;
use super::car_controller::*;
use super::car_hl_controller::*;
use super::town::*;
use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};
use nphysics2d::world::World as PWorld;
use conrod::color::*;

use std::time;

use super::debouncer::*;
use std::collections::HashMap;
use super::key_action_mapper::*;
use nalgebra;

pub struct VehicleManagerKeyMapping {
    key_action_map: HashMap<piston_window::Key, Box<Debouncer<VehicleManager>>>
}

pub struct VehicleManager {
    // non playable vehicles
    id_provider: Rc<RefCell<IdProvider>>,
    non_playable_vehicles: Vec<Car>,
    last_spawn_time: time::Instant,
}


// pub fn build_key_mapping_for_vehicle_manager() -> KeyActionMapper<VehicleManager>  {
//     let mut vehicle_manager_key_mapping = KeyActionMapper::<VehicleManager>::new();
//     vehicle_manager_key_mapping.add_action(piston_window::Key::K, 200,  |mgr: &mut VehicleManager| {
//                 mgr.spawn_random_close_to_protagonist();
//             });
//     vehicle_manager_key_mapping.add_action(piston_window::Key::X, 200,  |mgr: &mut VehicleManager| {
//                 mgr.non_playable_vehicles.clear();
//             });
//     vehicle_manager_key_mapping
// }

// impl VehicleManagerKeyMapping {
//     pub fn new() -> VehicleManagerKeyMapping {
//         let mut key_action_map: HashMap<
//             piston_window::Key,
//             Box<Debouncer<VehicleManager>>,
//         > = HashMap::new();
//         // let debouncer: Debouncer<VehicleManager> =
//         //     Debouncer::from_millis(200, |mgr: &mut VehicleManager| {
//         //         mgr.spawn_random_close_to_protagonist();
//         //     });
//         // let debouncer_box = Box::new(debouncer);
//         // key_action_map.insert(piston_window::Key::K, debouncer_box);

//         VehicleManagerKeyMapping {
//             key_action_map: key_action_map
//         }
//     }
// }

impl VehicleManager {
    pub fn get_non_playable_vehicles(&self) -> &Vec<Car> {
        &self.non_playable_vehicles
    }

    // pub fn get_protagonist_vehicle(&self) -> &Car {
    //     &self.protagonist_vehicle
    // }

    pub fn make_protagonist_car(&mut self) -> Car {
        Car {
            id: self.id_provider.borrow_mut().next(),
            wheel_yaw: 0.0,
            wheel_base: 2.5,
            bb_size: Size2f64::new(1.5, 3.0),
            color: rgb(1.0, 0.0, 1.0),
        }
    }

    pub fn new(mut id_provider: Rc<RefCell<IdProvider>>) -> VehicleManager {

        let vehicle_manager = VehicleManager {
            id_provider: id_provider,
            non_playable_vehicles: Vec::new(),
            last_spawn_time: time::Instant::now()
        };
        vehicle_manager
    }

    pub fn process_buttons(&mut self, vehicle_manager_key_mappings : &mut VehicleManagerKeyMapping, 
            buttons: &HashSet<piston_window::Button>) {

        if buttons.contains( &piston_window::Button::Keyboard(piston_window::Key::K) ) {
            let action = vehicle_manager_key_mappings.key_action_map.get_mut(&piston_window::Key::K);
            action.unwrap().debounce(self);
        }

        if buttons.contains( &piston_window::Button::Keyboard(piston_window::Key::A) ) {
            let action = vehicle_manager_key_mappings.key_action_map.get_mut(&piston_window::Key::A);
            action.unwrap().debounce(self);
        }

        if buttons.contains( &piston_window::Button::Keyboard(piston_window::Key::P) ) {
            let action = vehicle_manager_key_mappings.key_action_map.get_mut(&piston_window::Key::P);
            action.unwrap().debounce(self);
        }

        if buttons.contains( &piston_window::Button::Keyboard(piston_window::Key::X) ) {
            self.non_playable_vehicles.clear();
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_constructor() {
        let mut id_provider = Box::new(IdProvider::new());
        let vehicle_manager = VehicleManager::new(id_provider);
        assert_eq!(0, vehicle_manager.non_playable_vehicles.len());
    }

    #[test]
    fn spawn_one_car() {
        let mut id_provider = Box::new(IdProvider::new());
        let mut vehicle_manager = VehicleManager::new(id_provider);

        vehicle_manager.spawn_random_close_to_protagonist();

        assert_eq!(1, vehicle_manager.non_playable_vehicles.len());
    }

}




pub struct SpawnNewCarSys<'a> {
    pub vehicle_mgr: &'a mut VehicleManager,
    pub physics_world: &'a mut PWorld<f64>
}


impl <'a, 'b> System<'a> for SpawnNewCarSys<'b> {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, InputState>,
        WriteStorage<'a, Car>,
        WriteStorage<'a, Node>,
        WriteStorage<'a, PhysicsComponent>,
        WriteStorage<'a, CarController>,
        WriteStorage<'a, CarHighLevelControllerState>,
        ReadStorage<'a, ProtagonistTag>,
        ReadExpect<'a, TownGridMap>,
        Read<'a, LazyUpdate>
    );

    fn run(&mut self, (entities, mut input_state, mut cars, mut nodes, mut physics_components, 
                             mut car_controllers, mut car_hl_controller_states,
                             protagonist_tags, town_gridmap, updater): Self::SystemData) {

        if input_state.buttons_pressed.contains(&piston_window::Button::Keyboard(piston_window::Key::K)) {

            for (node, car, _protagonist) in (&nodes, &cars, &protagonist_tags).join() {
                let protagonist_car = car;
                let protagonist_node = node;

                let new_entity = entities.create();


                let mut new_car = random_car(&mut self.vehicle_mgr.id_provider.borrow_mut());
                let protagonist_trasl = node.pose.center;
                let mut new_car_pose = Pose2DF64::default();

                let mut new_car_random_search_point = Vec2f64::new(0.0, 0.0);
                new_car_random_search_point.x = protagonist_trasl.x + thread_rng().gen_range(-20.0, 20.0) * thread_rng().choose(&vec![-1.0, 1.0]).unwrap();
                new_car_random_search_point.y = protagonist_trasl.y + thread_rng().gen_range(-20.0, 20.0) * thread_rng().choose(&vec![-1.0, 1.0]).unwrap();

                let free_point = find_free_space_close_to(&town_gridmap, Vec2f32::new(new_car_random_search_point.x as f32,
                     new_car_random_search_point.y as f32));

                if(free_point.is_some()) {
                    new_car_pose.center.x = free_point.unwrap().x as f64;
                    new_car_pose.center.y = free_point.unwrap().y as f64;
                } else {
                    println!("no freepoint in map");
                    return
                }

                let protagonist_ds = protagonist_trasl - new_car_pose.center;
                let angle = Vec2f64::unit_x().angle(protagonist_ds);

                new_car_pose.yaw = std::f64::consts::PI / 2.0 * angle.sin().signum() + thread_rng().gen_range(-1.0, 1.0);

                let new_node = Node { pose: new_car_pose };

                let new_physics = make_physics_for_car(&mut self.physics_world, &new_car, &new_node.pose);

                let mut rigid_body = self.physics_world.rigid_body_mut(new_physics.body_handle).expect("protagonist rigid body not found");


                let mut car_high_level_controller_state = CarHighLevelControllerState::new();
                car_high_level_controller_state.target_long_speed = thread_rng().gen_range(10.0, 20.0);

                let mut car_path_controller_state = CarPathControllerState::new();

                updater.insert(
                    new_entity,
                    new_car, 
                );
                updater.insert(
                    new_entity,
                    new_node, 
                );
                updater.insert(
                    new_entity,
                    new_physics, 
                );
                updater.insert(
                    new_entity,
                    CarController{}, 
                );
                updater.insert(
                    new_entity,
                    car_high_level_controller_state
                );
                updater.insert(
                    new_entity,
                    car_path_controller_state
                );

            }

        }

	if input_state.buttons_pressed.contains(&piston_window::Button::Keyboard(piston_window::Key::A)) {

            for (node, car, _protagonist) in (&nodes, &cars, &protagonist_tags).join() {
                let protagonist_car = car;
                let protagonist_node = node;

                let new_entity = entities.create();

                let mut new_car = random_car(&mut self.vehicle_mgr.id_provider.borrow_mut());

		new_car.color = rgb(0.9, 0.9, 0.1);

                let mut new_car_pose = Pose2DF64{
		    center: Point2f64{
			  x: -45.0,
			  y: 40.0
		    },
		    yaw: 1.57
		};

                let free_point = find_free_space_close_to(&town_gridmap, Vec2f32::new(new_car_pose.center.x as f32,
                     new_car_pose.center.y as f32));

                if(free_point.is_some()) {
                    new_car_pose.center.x = free_point.unwrap().x as f64;
                    new_car_pose.center.y = free_point.unwrap().y as f64;
                } else {
                    println!("no freepoint in map");
                    return
                }

                let new_node = Node { pose: new_car_pose };

                let new_physics = make_physics_for_car(&mut self.physics_world, &new_car, &new_node.pose);

                let mut rigid_body = self.physics_world.rigid_body_mut(new_physics.body_handle).expect("protagonist rigid body not found");


                let mut car_high_level_controller_state = CarHighLevelControllerState::new();
                car_high_level_controller_state.target_long_speed = thread_rng().gen_range(10.0, 20.0);

                let mut car_path_controller_state = CarPathControllerState::new();

                updater.insert(
                    new_entity,
                    new_car, 
                );
                updater.insert(
                    new_entity,
                    new_node, 
                );
                updater.insert(
                    new_entity,
                    new_physics, 
                );
                updater.insert(
                    new_entity,
                    CarController{}, 
                );
                updater.insert(
                    new_entity,
                    car_high_level_controller_state
                );
                updater.insert(
                    new_entity,
                    car_path_controller_state
                );

            }

        }

	if input_state.buttons_pressed.contains(&piston_window::Button::Keyboard(piston_window::Key::P)) {

            for (node, car, _protagonist) in (&nodes, &cars, &protagonist_tags).join() {
                let protagonist_car = car;
                let protagonist_node = node;

                let new_entity = entities.create();

                let mut new_car = random_car(&mut self.vehicle_mgr.id_provider.borrow_mut());

		new_car.color = rgb(0.1, 0.1, 0.9);

                let mut new_car_pose = Pose2DF64{
		    center: Point2f64{
			  x: -35.0,
			  y: -40.0
		    },
		    yaw: -1.57
		};

                let free_point = find_free_space_close_to(&town_gridmap, Vec2f32::new(new_car_pose.center.x as f32,
                     new_car_pose.center.y as f32));

                if(free_point.is_some()) {
                    new_car_pose.center.x = free_point.unwrap().x as f64;
                    new_car_pose.center.y = free_point.unwrap().y as f64;
                } else {
                    println!("no freepoint in map");
                    return
                }

                let new_node = Node { pose: new_car_pose };

                let new_physics = make_physics_for_car(&mut self.physics_world, &new_car, &new_node.pose);

                let mut rigid_body = self.physics_world.rigid_body_mut(new_physics.body_handle).expect("protagonist rigid body not found");


                let mut car_high_level_controller_state = CarHighLevelControllerState::new();
                car_high_level_controller_state.target_long_speed = thread_rng().gen_range(10.0, 20.0);

                let mut car_path_controller_state = CarPathControllerState::new();

                updater.insert(
                    new_entity,
                    new_car, 
                );
                updater.insert(
                    new_entity,
                    new_node, 
                );
                updater.insert(
                    new_entity,
                    new_physics, 
                );
                updater.insert(
                    new_entity,
                    CarController{}, 
                );
                updater.insert(
                    new_entity,
                    car_high_level_controller_state
                );
                updater.insert(
                    new_entity,
                    car_path_controller_state
                );

            }

        }

    }
}
