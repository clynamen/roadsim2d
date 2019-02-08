use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::error::Error;

use super::primitives::*;

type SimTimeStamp = f64; 
use std::collections::VecDeque;
use std::vec::Vec;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScriptedCar {
    pub rgb  : (f32, f32, f32),
    pub pose : Pose2DF64,
    pub cmds : VecDeque<CarActionState>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CarActionState {
    pub stamp:   SimTimeStamp,
    pub lon_vel: f32,
    pub yaw:     f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub town_image : Option<String>,
    pub cars : Vec<ScriptedCar>
}

pub struct ScenarioLoader {

}

impl ScenarioLoader {

    pub fn read_from_file(fname: &str) -> Result<Scenario, Box<std::error::Error>> {
        let mut file = File::open(fname)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data: Scenario = serde_yaml::from_str(&contents)?;
        Ok(data)
    }

}
