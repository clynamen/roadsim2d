use super::car::*;
use super::primitives::*;
use super::sim_id::*;
use cgmath::*;
use conrod::color::*;
// use piston::input::{Button, Key};
use rand::*;
use std::boxed::Box;
use std::collections::HashSet;

use std::time;

use super::debouncer::*;
use std::collections::HashMap;

pub struct VehicleManagerKeyMapping {
    key_action_map: HashMap<piston_window::Key, Box<Debouncer<VehicleManager>>>
}

pub struct VehicleManager {
    // non playable vehicles
    id_provider: Box<IdProvider>,
    non_playable_vehicles: Vec<Car>,
    protagonist_vehicle: Car,
    last_spawn_time: time::Instant,
}

impl VehicleManagerKeyMapping {
    pub fn new() -> VehicleManagerKeyMapping {
        let mut key_action_map: HashMap<
            piston_window::Key,
            Box<Debouncer<VehicleManager>>,
        > = HashMap::new();
        let debouncer: Debouncer<VehicleManager> =
            Debouncer::from_millis(200, |mgr: &mut VehicleManager| {
                mgr.spawn_random_close_to_protagonist();
            });
        let debouncer_box = Box::new(debouncer);
        key_action_map.insert(piston_window::Key::K, debouncer_box);

        VehicleManagerKeyMapping {
            key_action_map: key_action_map
        }
    }
}

impl VehicleManager {
    pub fn get_non_playable_vehicles(&self) -> &Vec<Car> {
        &self.non_playable_vehicles
    }

    pub fn get_protagonist_vehicle(&self) -> &Car {
        &self.protagonist_vehicle
    }

    pub fn new(mut id_provider: Box<IdProvider>) -> VehicleManager {
        let protagonist_car = Car {
            id: id_provider.next(),
            pose: Pose2DF64 {
                center: Point2f64 { x: 0.0, y: 0.0 },
                yaw: 0.0,
            },
            longitudinal_speed: 0.0,
            yaw_rate: 0.0,
            bb_size: Size2f64::new(1.5, 3.0),
            color: rgb(1.0, 0.0, 1.0),
        };


        let vehicle_manager = VehicleManager {
            id_provider: id_provider,
            non_playable_vehicles: Vec::new(),
            protagonist_vehicle: protagonist_car,
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

        if buttons.contains( &piston_window::Button::Keyboard(piston_window::Key::C) ) {
            self.non_playable_vehicles.clear();
        }
    }

    pub fn set_protagonist_speed(&mut self, speed: f64, yaw_rate: f64) {
        self.protagonist_vehicle.longitudinal_speed = speed as f32;
        self.protagonist_vehicle.yaw_rate = yaw_rate as f32;
    }

    pub fn spawn_random_close_to_protagonist(&mut self) {
        let mut new_car = random_car(&mut *self.id_provider);

        let protagonist_trasl = self.protagonist_vehicle.pose.center;
        let mut new_car_pose = Pose2DF64::default();

        new_car_pose.center.x = protagonist_trasl.x + thread_rng().gen_range(10.0, 20.0);
        new_car_pose.center.y = protagonist_trasl.y + thread_rng().gen_range(-20.0, 20.0);

        let protagonist_ds = protagonist_trasl - new_car_pose.center;
        let angle = Vec2f64::unit_x().angle(protagonist_ds);

        new_car_pose.yaw = std::f64::consts::PI / 2.0 * angle.sin().signum() + thread_rng().gen_range(-1.0, 1.0);

        new_car.pose = new_car_pose;

        self.non_playable_vehicles.push(new_car);
        self.last_spawn_time = time::Instant::now();
    }

    pub fn update(&mut self, dt_s: f32) {
        let protagonist_car_center = self.protagonist_vehicle.pose.center;
        self.non_playable_vehicles.retain(|vehicle| vehicle.pose.center.distance(protagonist_car_center) < 1.0e3);

        &mut self.protagonist_vehicle.update(dt_s);
        for car in &mut self.non_playable_vehicles {
            car.update(dt_s);
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
