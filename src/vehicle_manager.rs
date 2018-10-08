use super::car::*;

use std::time;

struct VehicleManager {
    // non playable vehicles 
    non_playable_vehicles: Vec<Car>,
    protagonist_vehicle: Car,
    last_spawn_time : time::Instant,
}


impl VehicleManager {

    pub default() {
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
            non_playable_vehicles : Vec::new(),
            protagonist_vehicle: protagonist_car,
            last_spawn_time : time::Instant::now()
        }
    }

    pub process_buttons(mut &self, buttons: &HashSet<Button>) {

    }

    pub spawn_random_close_to_protagonist() {

    }

    pub update(mut &self, dt: f32) {
        &mut protagonist_vehicle.update(dt_s, false);
        for car in &mut non_playable_vehicles {
            car.update(dt_s, true);
        }
    }

}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}