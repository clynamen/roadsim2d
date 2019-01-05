#![feature(duration_as_u128)]
#![feature(fn_traits)]
#![feature(unboxed_closures)] 
#[macro_use]
extern crate glium;
extern crate image;


use roadsim2dlib::*;
use image::{GenericImage, ImageBuffer};
use std::ops::Index;


fn main() {

    let gridmap = make_random_town_gridmap(1);

    let image = ImageBuffer::from_fn(1024, 1024, |x, y| {
        // let v = *gridmap.index( (x as usize, y as usize) ) as u8;
        let v = gridmap.has_vertex(&(x as usize, y as usize) ) as u8;
        image::Rgba([255u8 * v, 0u8, 0u8, 0u8])
    });

    show_rgba_image(image); 
}
