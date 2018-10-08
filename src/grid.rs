use piston_window::*;
use piston::input::{Button, Key};
use piston::event_loop::*;
use std::collections::HashSet;
use piston::input::Input::*;

pub struct Grid {
    pub enabled: bool
}

pub fn draw_circle<G>(color: [f32; 4], radius: f64, transform: [[f64; 3]; 2], 
    g: &mut G) where G : piston_window::Graphics{

        Ellipse::new(color).resolution(10)
            .draw([10.0, 10.0, 10.0, 10.0], &Default::default(), transform, g);
}

impl Grid {
    pub fn update(&mut self, buttons: &HashSet<Button>) {
        macro_rules! if_key {
            ($key:path : $buttons:ident $then:block) => {
                if $buttons.contains(&Button::Keyboard($key)) {
                    $then
                }
            };
        }
        if_key! [ Key::G : buttons { self.enabled = !self.enabled; }];
    }

    pub fn draw(&self, context: Context, graphics: &mut G2d) {
        // let center = context.transform.trans(ix as f64 *100.0, iy as f64 *100.0);
        // let square = rectangle::square(0.0, 0.0, 100.0);
        // draw_circle( [0.25, 0.25, 0.25, 0.5], // red
        if (!self.enabled) {
            return;
        }
         let color = [0.2, 0.2, 0.2, 0.8];
        let grid_size = 32;
        let grid_dist = 100.0;
        let center = context.transform.trans( -grid_size as f64 / 2.0 * grid_dist,
                                              -grid_size as f64 / 2.0 * grid_dist);
        for ix in 0..grid_size {
            for iy in 0..grid_size {
                let center = context.transform.trans(ix as f64 *100.0, iy as f64 *100.0);
                draw_circle( color, // red
                            10.0, 
                            center,
                            graphics);
            }
        }
        // rectangle( color, // red
        //             [-100.0, 
        //             -100.0, 
        //             100.0, 
        //             100.0],
        //             center,
        //             graphics);
    }
}