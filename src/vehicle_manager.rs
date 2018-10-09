use super::car::*;
use super::primitives::*;
use super::sim_id::*;
use conrod::color::*;
use std::collections::HashSet;
use piston::input::{Button, Key};
use std::boxed::Box;

use std::time;

struct VehicleManager {
    // non playable vehicles 
    id_provider: Box<IdProvider>, 
    non_playable_vehicles: Vec<Car>,
    protagonist_vehicle: Car,
    last_spawn_time : time::Instant,
}


impl VehicleManager {

    pub fn default(mut id_provider: Box<IdProvider>) -> VehicleManager {

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
        VehicleManager {
            id_provider: id_provider,
            non_playable_vehicles : Vec::new(),
            protagonist_vehicle: protagonist_car,
            last_spawn_time : time::Instant::now(),
        }
    }

    pub fn process_buttons(&mut self, buttons: &HashSet<Button>) {

    }

    pub fn spawn_random_close_to_protagonist() {

    }

    pub fn update(&mut self, dt_s: f32) {
        &mut self.protagonist_vehicle.update(dt_s, false);
        for car in &mut self.non_playable_vehicles {
            car.update(dt_s, true);
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut id_provider = Box::new(IdProvider::new());
        let vehicle_manager = VehicleManager::default(id_provider);
        assert_eq!(0, vehicle_manager.non_playable_vehicles.len());
    }
}