
use conrod::color::*;
// use rand::Rng;

pub fn random_color() -> Color {
    rgb(::rand::random(), ::rand::random(), ::rand::random())
}

pub fn to_rgba(c : &Color, a: f32) -> [f32; 4] {
    [c.red(), c.green(), c.blue(), a]
} 