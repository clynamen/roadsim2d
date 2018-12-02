extern crate piston_window;
extern crate piston;

use piston_window::*;
use super::simulation::*;
use super::primitives::*;
use super::key_action_mapper::*;
use std::collections::HashSet;
use specs::{Component, VecStorage};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    trans: Vec2f64,
    zoom: f64,
    trans_vel: Vec2f64,
    zoom_vel: f64,
    camera_mode: CameraMode,
    target_trasl: Point2f64,
}

impl Component for Camera {
    type Storage = VecStorage<Self>;
}


#[derive(Debug, Clone, Copy)]
enum CameraMode {
    Free,
    FollowTarget
}

impl Camera {
    pub fn new(trans: Vec2f64, zoom: f64) -> Camera {
        Camera {
            trans: trans,
            zoom: zoom,
            trans_vel: zero_vec2f64(),
            zoom_vel: 0.0,
            camera_mode: CameraMode::FollowTarget,
            target_trasl: Point2f64{x: 0.0, y:0.0}
        }
    }

    pub fn get_zoom_level(&self) -> f64 {
        self.zoom
    }

    pub fn set_target_trals(&mut self, trasl: Point2f64) {
        self.target_trasl = trasl;
    }

    pub fn apply<T: Transformed>(&self, transform: T) -> T {
        transform.trans(self.trans.x, self.trans.y).zoom(self.zoom)
    }
}

pub fn build_key_mapping_for_camera_manager() -> KeyActionMapper<Camera>  {
    let mut camera_manager_key_mapping = KeyActionMapper::<Camera>::new();
    camera_manager_key_mapping.add_action(piston_window::Key::C, 200,  |camera: &mut Camera| {
                let new_mode = match camera.camera_mode {
                    CameraMode::Free => CameraMode::FollowTarget,
                    _ => CameraMode::Free
                };
                camera.camera_mode = new_mode;
            });
    camera_manager_key_mapping
}

impl Camera {
    // fn render(&self, _: Context, _: &mut GlGraphics) {}


    pub fn update_cam(&mut self, dt: f64, buttons: &HashSet<piston_window::Button>, window_size: piston_window::Size) {
        macro_rules! if_key {
            ($key:path : $buttons:ident $then:block) => {
                if $buttons.contains(&piston_window::Button::Keyboard($key)) {
                    $then
                }
            };
        }


        let zoom_amount = 0.001;
        if_key! [ piston_window::Key::E : buttons { self.zoom_vel += zoom_amount; }];
        if_key! [ piston_window::Key::Q : buttons { self.zoom_vel -= zoom_amount; }];

        // if_key! [ piston_window::Key::C : buttons { 
        //     let new_mode = match self.camera_mode {
        //         CameraMode::Free => CameraMode::FollowTarget,
        //         _ => CameraMode::Free
        //     };
        //     self.camera_mode = new_mode;
        // }];

        match self.camera_mode {
            CameraMode::FollowTarget => {
                let screen_width = window_size.width as f64;
                let screen_height = window_size.height as f64;
                self.trans.x = -self.target_trasl.x*self.zoom +screen_width/2.0;
                self.trans.y =  self.target_trasl.y*self.zoom +screen_height/2.0;
            }
            _ => {
                let scroll_speed = 0.7;
                if_key! [ piston_window::Key::Up : buttons { self.trans_vel = self.trans_vel + Vec2f64{x: 0.0, y: scroll_speed}; }];
                if_key! [ piston_window::Key::Down : buttons { self.trans_vel = self.trans_vel + Vec2f64{x: 0.0, y: -scroll_speed}; }];
                if_key! [ piston_window::Key::Left : buttons { self.trans_vel = self.trans_vel + Vec2f64{x: scroll_speed, y: 0.0}; }];
                if_key! [ piston_window::Key::Right : buttons { self.trans_vel = self.trans_vel + Vec2f64{x: -scroll_speed, y: 0.0}; }];
                self.trans = self.trans + self.trans_vel;
                self.trans_vel = self.trans_vel * 0.9;
            }
        }

        self.zoom *= 1.0 + self.zoom_vel;
        self.zoom_vel *= 0.9;
    }
}
