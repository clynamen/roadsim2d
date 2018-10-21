// from mythmon/rust-nbodies
// use graphics::context::Context;
// use graphics::Transformed;
// use opengl_graphics::GlGraphics;
extern crate piston_window;
extern crate piston;
use piston::input::{Button, Key};
// use num::Zero;

// use super::vector::Vec2;
// use super::game::{GameObject, UpdateContext};
use piston_window::*;
use super::simulation::*;
use super::primitives::*;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    trans: Vec2f64,
    zoom: f64,
    trans_vel: Vec2f64,
    zoom_vel: f64,
    camera_mode: CameraMode,
    target_trasl: Point2f64,
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

    pub fn set_target_trals(&mut self, trasl: Point2f64) {
        self.target_trasl = trasl;
    }

    pub fn apply<T: Transformed>(&self, transform: T) -> T {
        transform.trans(self.trans.x, self.trans.y).zoom(self.zoom)
    }
}

impl Camera {
    // fn render(&self, _: Context, _: &mut GlGraphics) {}

    pub fn update_cam(&mut self, ctx: &UpdateContext, window_size: piston_window::Size) {
        macro_rules! if_key {
            ($key:path : $ctx:ident $then:block) => {
                if $ctx.buttons.contains(&Button::Keyboard($key)) {
                    $then
                }
            };
        }


        let zoom_amount = 0.001;
        if_key! [ Key::E : ctx { self.zoom_vel += zoom_amount; }];
        if_key! [ Key::Q : ctx { self.zoom_vel -= zoom_amount; }];

        if_key! [ Key::C : ctx { 
            let new_mode = match self.camera_mode {
                CameraMode::Free => CameraMode::FollowTarget,
                _ => CameraMode::Free
            };
            self.camera_mode = new_mode;
        }];

        match self.camera_mode {
            CameraMode::FollowTarget => {
                let screen_width = window_size.width as f64;
                let screen_height = window_size.height as f64;
                self.trans.x = -self.target_trasl.x*self.zoom +screen_width/2.0;
                self.trans.y =  self.target_trasl.y*self.zoom +screen_height/2.0;
            }
            _ => {
                let scroll_speed = 0.7;
                if_key! [ Key::Up : ctx { self.trans_vel = self.trans_vel + Vec2f64{x: 0.0, y: scroll_speed}; }];
                if_key! [ Key::Down : ctx { self.trans_vel = self.trans_vel + Vec2f64{x: 0.0, y: -scroll_speed}; }];
                if_key! [ Key::Left : ctx { self.trans_vel = self.trans_vel + Vec2f64{x: scroll_speed, y: 0.0}; }];
                if_key! [ Key::Right : ctx { self.trans_vel = self.trans_vel + Vec2f64{x: -scroll_speed, y: 0.0}; }];
                self.trans = self.trans + self.trans_vel;
                self.trans_vel = self.trans_vel * 0.9;
            }
        }

        self.zoom *= 1.0 + self.zoom_vel;
        self.zoom_vel *= 0.9;
    }
}
