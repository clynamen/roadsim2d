use nalgebra::*;
use glium::*;

type TownGridMap = Matrix<i32, Dynamic, Dynamic, MatrixVec<i32, Dynamic, Dynamic>>;

struct TownGrid {

}

fn display_map() {
    let mut events_loop = glium::glutin::EventsLoop::new();
    let window = glium::glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(1024.0, 768.0))
        .with_title("TownGrid");
    let context = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop).unwrap();
}