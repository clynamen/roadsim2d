use nalgebra::*;
use glium::*;
use cgmath;
use euclid;
use rand;
use rand::Rng;
use piston_window::*;
use ::image::{GenericImage, ImageBuffer};
use super::camera::Camera;
use super::primitives::*;
use num::clamp;

use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Add;
use std::collections::VecDeque;
use cgmath::Rotation;
use specs::{System, ReadStorage, Component, ReadExpect};
use pathfinding::grid;

// pub type TownGridMap = Matrix<i32, Dynamic, Dynamic, MatrixVec<i32, Dynamic, Dynamic>>;
pub type TownGridMap = grid::Grid;
use num::traits::Pow;

struct TownGrid {

}

struct TownTurtle {
    pos: Vec2i32,
    theta: f32
}

pub fn vec2f32_2_vec2i32(v: Vec2f32) -> Vec2i32 {
    Vec2i32::new(v.x as i32, v.y as i32)
}

pub fn vec2i32_2_vec2f32_center(v: Vec2i32) -> Vec2f32 {
    Vec2f32::new(v.x as f32 + 0.5f32, v.y as f32 + 0.5f32)
}

const TOWN_SIZE : usize = 1000usize;
const TOWN_ZOOM : f64 = 1.0;
const TURTLE_DRAW_RADIUS : i32 = 4;

fn gridmap_xy_to_world(pos: Vec2i32) -> Vec2f32 {
    (vec2i32_2_vec2f32_center(pos)  - Vec2f32::new(TOWN_SIZE as f32 /2.0f32, TOWN_SIZE as f32 /2.0f32) ) / TOWN_ZOOM as f32
}

fn world_to_gridmap_xy(pos: Vec2f32) -> Vec2i32 {
    vec2f32_2_vec2i32(pos * TOWN_ZOOM as f32 + Vec2f32::new(TOWN_SIZE as f32 /2.0f32, TOWN_SIZE as f32 /2.0f32))
}

fn gridmap_enforce_bounds(v: Vec2i32) -> Vec2i32 {
    Vec2i32::new(clamp(v.x, 0i32, TOWN_SIZE as i32), clamp(v.y, 0, TOWN_SIZE as i32))
}

fn world_to_gridmap_xy_enforce_bounds(pos: Vec2f32) -> Vec2i32 {
    gridmap_enforce_bounds(world_to_gridmap_xy(pos))
}

fn vec2i32_distance2(a: Vec2i32, b: Vec2i32) -> i32 {
    let dist2 = (a.x-b.x).pow(2) + (a.y -b.y).pow(2);
    dist2 as i32
}

const MAX_SEARCH_ITER : i32 = 2000;

fn duple_to_vec2i32(dup : (usize, usize) ) -> Vec2i32 {
    Vec2i32::new(dup.0 as i32, dup.1 as i32)
}

fn vec2i32_2_duple(vec: Vec2i32) -> (usize, usize) {
    (vec.x as usize, vec.y as usize)
}

pub fn find_shortest_path(gridmap: &TownGridMap, start_point: Vec2f32, end_point: Vec2f32) -> Option<VecDeque<Vec2f32>> {
    let start_point_grid = world_to_gridmap_xy_enforce_bounds(start_point);
    let end_point_grid = world_to_gridmap_xy_enforce_bounds(end_point);

    let result = pathfinding::directed::astar::astar(&start_point_grid,
                   |&point|  {
                       let neigh: Vec<(Vec2i32, i32)> = gridmap.neighbours(&vec2i32_2_duple(point)).iter().map(
                           | (vert_x, vert_y) | (duple_to_vec2i32( (*vert_x, *vert_y) ), 1) ).collect();
                        neigh
                   },
                   |&point| {
                        // absdiff(x, GOAL.0) + absdiff(y, GOAL.1)
                        vec2i32_distance2(point, end_point_grid)
                        // 0i32
                   },
                   |&point| point == end_point_grid);
    match result {
        Some( (points_and_dists, dist) ) => Some(points_and_dists.iter().map(| point |  {
            gridmap_xy_to_world(*point)
        }).collect()),
        None => None
    }
}

pub fn find_free_space_close_to(gridmap: &TownGridMap, query_point: Vec2f32) -> Option<Vec2f32> {
    let query_grid_pos = world_to_gridmap_xy_enforce_bounds(query_point);

    // println!("query world: {:?} query grid: {:?}", query_point, query_grid_pos);


    let closest_option = gridmap.iter().min_by_key(| vert | {
        let vert_pos = Vec2i32::new(vert.0 as i32, vert.1 as i32);
        vec2i32_distance2(vert_pos, query_grid_pos)
    });

    if closest_option.is_some() {
        let closest_grid_pos = closest_option.unwrap();
        let closest_point_world = gridmap_xy_to_world(duple_to_vec2i32(closest_grid_pos));
        // let closest_point_world = Vec2f32::new(closest_grid_pos.0 as f32 - TOWN_SIZE as f32 / 2.0f32, 
        //                                        closest_grid_pos.1 as f32 - TOWN_SIZE as f32 / 2.0f32);
        // println!("found world: {:?} found grid: {:?}", closest_point_world, closest_grid_pos);
        Some(closest_point_world)
    } else {
        None
    }
}

pub fn make_square_town_gridmap() -> TownGridMap {
    let mut gridmap = TownGridMap::new(TOWN_SIZE, TOWN_SIZE);
    gridmap.enable_diagonal_mode();


    for x in -0..10 {
        for y in -0..10 {
            gridmap.add_vertex( ( (TOWN_SIZE as i32 / 2 + x) as usize, (TOWN_SIZE as i32 / 2 + y) as usize) );
        }
    }


    gridmap
}

pub fn make_random_town_gridmap(seed: u32) -> TownGridMap {
    let mut gridmap = TownGridMap::new(TOWN_SIZE, TOWN_SIZE);
    gridmap.enable_diagonal_mode();

    let start_point = Vec2i32::new(TOWN_SIZE as i32/ 2, TOWN_SIZE as i32/ 2);

    let mut rng = rand::thread_rng();
    let mut first_turtle = TownTurtle {pos: start_point, theta: rng.gen_range(0f32,  2.0f32 * std::f32::consts::PI) };

    let radius = TURTLE_DRAW_RADIUS;

    let mut turtle_deque : VecDeque<TownTurtle> = VecDeque::new();
    let MAX_TURTLE = 15;
    let mut turtle_counter = 1u32;

    turtle_deque.push_back(first_turtle);


    loop {
        if(turtle_deque.is_empty()) {
            break;
        } else {
            let mut turtle = turtle_deque.pop_front().unwrap();

            'turtle_end: loop {
                for dx in -radius..radius {
                    for dy in -radius..radius {
                        let mark_point = turtle.pos + Vec2i32::new(dx, dy);
                        if (mark_point.x <= 0 || mark_point.x >= TOWN_SIZE as i32 - 1) || 
                            (mark_point.y <= 0 || mark_point.y >= TOWN_SIZE as i32- 1) {
                                break 'turtle_end;
                        }
                        // gridmap[(mark_point.x as usize, mark_point.y as usize)] = 1;
                        gridmap.add_vertex((mark_point.x as usize, mark_point.y as usize));
                    }
                } 
                if rng.gen_range(0,  255) > 250 {
                    turtle.theta += rng.gen_range(-1.0f32,  1.0f32)
                }

                let rot : cgmath::Basis2<f32> = cgmath::Rotation2::<f32>::from_angle(cgmath::Rad(turtle.theta));
                let turtle_pos_increment = rot.rotate_vector(Vec2f32::new( TURTLE_DRAW_RADIUS as f32, 0f32));
                turtle.pos +=  vec2f32_2_vec2i32(turtle_pos_increment);
                if rng.gen_range(0,  255) > 240 && turtle_counter < 15 {
                    let mut new_turtle = TownTurtle {pos: turtle.pos, 
                        theta: rng.gen_range(0f32,  2.0f32 * std::f32::consts::PI as f32) };
                    turtle_deque.push_back(new_turtle);
                    turtle_counter += 1u32;
                }
            }

        }
    }


    gridmap
}

fn town_gridmap_to_image(gridmap : &TownGridMap) -> ::image::RgbaImage {
    let image = ImageBuffer::from_fn(TOWN_SIZE as u32, TOWN_SIZE as u32, |x, y| {
        // let v = *gridmap.has_vertex( &(x as usize, y as usize) ) as u8;
        let v = gridmap.has_vertex( &(x as usize, TOWN_SIZE - 1 - y as usize) ) as u8;
        let gray_value = 30u8;
        ::image::Rgba([gray_value, gray_value, gray_value, 255u8*v])
    });
    image
}

pub fn town_gridmap_to_texture(window: &mut PistonWindow, townGridMap : &TownGridMap) -> G2dTexture {
    let img = town_gridmap_to_image(townGridMap);
    Texture::from_image(&mut window.factory, &img, &TextureSettings::new()).unwrap()
}

// pub fn draw_town_texture(window: &mut PistonWindow, tex: &Texture) {
    // window.draw_2d(&e, |c, g| {
    //     clear([1.0; 4], g);
    //     image(tex, c.transform, g);
    // });
// }
pub struct RenderTownSys<'a> {
    pub fps_window: &'a mut PistonWindow,
    pub town_gridmap_texture: &'a G2dTexture,
    pub render_event: &'a Event,
    pub render_args:  RenderArgs, 
}



impl<'a, 'b> System<'a> for RenderTownSys<'b> {
    type SystemData = (ReadExpect<'a, Camera>);

    fn run(&mut self, (camera): Self::SystemData) {
        let texture_copy = self.town_gridmap_texture;

        self.fps_window.draw_2d(self.render_event, |context, graphics| {
            let mut context = context;
            let new_trans = camera.apply(context.transform);
            context.transform = new_trans.zoom(TOWN_ZOOM).trans(TOWN_SIZE as f64 / -2.0, TOWN_SIZE as f64 / -2.0);
            image(texture_copy, context.transform, graphics);

        });

    }
}