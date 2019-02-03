use piston_window::*;
use opengl_graphics::{GlGraphics, GlyphCache};
use specs::{System, ReadStorage, Component, ReadExpect};
use super::camera::Camera;

#[derive(Default)]
pub struct SimInfo {
    pub sim_time: f64, 
    pub fps:      f32
}

pub struct RenderInfoSys<'a, 'b, 'c> {
    pub render_args:  RenderArgs, 
    pub font_glyphs:  &'a mut GlyphCache<'c>,
    pub opengl:       &'b mut GlGraphics
}

impl<'a, 'b, 'c, 'd> System<'a> for RenderInfoSys<'b, 'c, 'd> {
    type SystemData = (ReadExpect<'a, SimInfo>, ReadExpect<'a, Camera>);
    

    fn run(& mut self, (info, camera): Self::SystemData) {
        let font = &mut self.font_glyphs;

        self.opengl.draw(self.render_args.viewport(), |context, graphics| {
            let mut context = context;
            let font_size = 18;
            let tran = context.transform.zoom(1.0);

            let fps_str = format!("{}", info.fps);

            piston_window::text(
                [1.0, 0.0, 0.0, 1.0],
                font_size,
                &fps_str,
                *font,
                tran.trans(00.0, font_size as f64),
                graphics,
            );

            let sim_time_str = format!("{}", info.sim_time);

            piston_window::text(
                [1.0, 0.0, 0.0, 1.0],
                font_size,
                &sim_time_str,
                *font,
                tran.trans(100.0, font_size as f64),
                graphics,
            );
        });

    }
}