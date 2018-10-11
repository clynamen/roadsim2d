use super::car::*;
use super::primitives::*;
use super::sim_id::*;
use conrod::color::*;
use std::collections::HashSet;
use piston::input::{Button, Key};
use std::boxed::Box;
use rand::*;
use cgmath::*;

use std::time;

pub struct VehicleManager {
    // non playable vehicles 
    id_provider: Box<IdProvider>, 
    non_playable_vehicles: Vec<Car>,
    protagonist_vehicle: Car,
    last_spawn_time : time::Instant,
}


impl VehicleManager {

    pub fn get_non_playable_vehicles(&self) -> &Vec<Car> {
        &self.non_playable_vehicles
    }

    pub fn get_protagonist_vehicle(&self) -> &Car {
        &self.protagonist_vehicle
    }

    pub fn new(mut id_provider: Box<IdProvider>) -> VehicleManager {

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
        macro_rules! if_key {
            ($key:path : $buttons:ident $then:block) => {
                if $buttons.contains(&Button::Keyboard($key)) {
                    $then
                }
            };
        }


        if_key! [ Key::K : buttons { self.spawn_random_close_to_protagonist(); }];
    }

    pub fn spawn_random_close_to_protagonist(&mut self) {
        let mut new_car = random_car(&mut *self.id_provider);

        let protagonist_trasl = self.protagonist_vehicle.pose.center;
        let mut new_car_pose = Pose2DF64::default();

        new_car_pose.center.x = protagonist_trasl.x + thread_rng().gen_range(100.0, 300.0);
        new_car_pose.center.y = protagonist_trasl.y + thread_rng().gen_range(-300.0, 300.0);

        let angle = (protagonist_trasl - new_car_pose.center).angle(Vec2f64::unit_y());
        println!("Angle: {}", angle.0);

        new_car_pose.yaw = std::f64::consts::PI/2.0 * angle.0.signum();

        new_car.pose = new_car_pose;

        self.non_playable_vehicles.push(new_car);
    }

    pub fn update(&mut self, dt_s: f32) {
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
        let vehicle_manager = VehicleManager::default(id_provider);
        assert_eq!(0, vehicle_manager.non_playable_vehicles.len());
    }

    #[test]
    fn spawn_one_car() {
        let mut id_provider = Box::new(IdProvider::new());
        let mut vehicle_manager = VehicleManager::default(id_provider);

        vehicle_manager.spawn_random_close_to_protagonist();

        assert_eq!(1, vehicle_manager.non_playable_vehicles.len());
    }


}