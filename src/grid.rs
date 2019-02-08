use piston_window::*;
use super::global_resources::*;
use super::global_resources::*;
use super::input::*;
use std::collections::HashSet;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};

use super::camera::Camera;

pub struct Grid {
    pub enabled: bool,
    reference_zoom_level: f64,
}

impl Component for Grid {
    type Storage = VecStorage<Self>;
}

// pub fn draw_circle<G>(color: [f32; 4], _radius: f64, transform: [[f64; 3]; 2], 
//     g: &mut G) where G : piston_window::Graphics{

//         Ellipse::new(color).resolution(10)
//             .draw([10.0, 10.0, 10.0, 10.0], &Default::default(), transform, g);
// }

const GRID_SIZE : u32 = 200u32;

impl Grid {
    pub fn new() -> Grid {
        return Grid {
            enabled: true,
            reference_zoom_level: 1.0
        }
    }

    pub fn update(&mut self, buttons: &HashSet<piston_window::Button>) {
        macro_rules! if_key {
            ($key:path : $buttons:ident $then:block) => {
                if $buttons.contains(&piston_window::Button::Keyboard($key)) {
                    $then
                }
            };
        }
        if_key! [ piston_window::Key::G : buttons { self.enabled = !self.enabled; }];
    }

    pub fn set_reference_zoom_level(&mut self, reference_zoom_level: f64) {
        self.reference_zoom_level = reference_zoom_level;
    }

    pub fn draw(&self, context: Context, graphics: &mut G2d) {
        if !self.enabled {
            return;
        }

        let mut color = [0.2, 0.2, 0.2, 0.2];

        let grid_size = 3;
        let mut grid_unit = 1.0;
        let mut line_thickness = 0.05;
        if self.reference_zoom_level < 1.0 {
            grid_unit = 1000.0;
            line_thickness = 2.0;
        } else if self.reference_zoom_level < 7.0 {
            grid_unit = 100.0;
            line_thickness = 0.5;
        } else if self.reference_zoom_level < 12.0 {
            grid_unit = 10.0;
            line_thickness = 0.05;
        // } else if self.reference_zoom_level >= 12.0 {
        //     grid_unit = 1.0;
        //     line_thickness = 0.01;
        }


        let grid_line = piston_window::Line::new(color, line_thickness);
        let grid_corner_dist = GRID_SIZE as f64 /2.0 * grid_unit;
        let center = context.transform.trans(-grid_corner_dist, -grid_corner_dist) ;
        graphics::grid::Grid {cols: GRID_SIZE, rows: GRID_SIZE, units: grid_unit}.draw(&grid_line, 
            &context.draw_state, center, graphics);
        let center_size = 2.0/ self.reference_zoom_level;
        ellipse([1.0, 0.0, 0.0, 1.0], [0.0, 0.0, center_size, center_size], context.transform, graphics);
        ellipse([1.0, 0.0, 0.0, 1.0], [0.0, 0.0, center_size, center_size], 
            context.transform.trans(-grid_corner_dist, -grid_corner_dist), graphics);

    }
}

pub struct RenderGridSys<'a> {
    pub fps_window: &'a mut PistonWindow,
    pub render_event: &'a Event,
    pub render_args:  RenderArgs, 
}

impl<'a, 'b> System<'a> for RenderGridSys<'b> {
    type SystemData = (ReadExpect<'a, Grid>, ReadExpect<'a, Camera>);

    fn run(&mut self, (grid, camera): Self::SystemData) {
        self.fps_window.draw_2d(self.render_event, |context, graphics| {
            let mut context = context;
            let new_trans = camera.apply(context.transform);
            context.transform = new_trans;
            grid.draw(context, graphics);
        });
    }
}

pub struct UpdateGridSys;


impl<'a> System<'a> for UpdateGridSys   {
    type SystemData = (
        ReadExpect<'a, UpdateDeltaTime>, 
        ReadExpect<'a, Camera>,
        ReadExpect<'a, InputState>,
        WriteExpect<'a, Grid>,
    );

    fn run(&mut self, (update_delta_time, camera, input_state, mut grid): Self::SystemData) {
        grid.update(&input_state.buttons_held);
        grid.set_reference_zoom_level(camera.get_zoom_level());
    }
}

