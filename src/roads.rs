
use super::primitives::*;
use super::sim_id::*;
use std::vec;

// enum RoadType {
//     ROAD,
//     JUNCTION
// } 

struct RoadGeometryBase {
    s: f64,
    origin: Vec2f64,
    yaw: f64,
    length: f64,
}

enum RoadGeometrySub {
    Line{},
    Spiral{curv_start: f32, curv_end: f32},
}

struct RoadGeometry {
    base: RoadGeometryBase,
    sub: RoadGeometrySub,
}

struct QuadrinomialParams {
    a: f32, 
    b: f32, 
    c: f32, 
    d: f32
}

impl QuadrinomialParams {
    fn zero_order(a: f32) -> QuadrinomialParams {
        QuadrinomialParams{a: a, b: 0.0, c: 0.0, d: 0.0}
    }
}

struct Lane {
    id: i32,
    s: f64,
    width_params: QuadrinomialParams,
}

struct LaneSection {
    s: f64,
    lanes: Vec<Lane>
}

impl LaneSection {
    fn new() -> LaneSection {
        LaneSection {s: 0.0, lanes: Vec::<Lane>::new()}
    }
}

struct Road {
    id: u64,
    junction: i64,
    geometries: Vec<RoadGeometry>,
    lane_sections: Vec<LaneSection>,
}

impl Road {

    fn new(id_provider: &mut IdProvider) -> Road {
        Road {
            id : id_provider.next(),
            junction: -1,
            geometries: Vec::<RoadGeometry>::new(),
            lane_sections: Vec::<LaneSection>::new(),
        }
    }
}

fn generate_random_road(id_provider: &mut IdProvider) -> Road {
    let mut road = Road::new(id_provider);
    let mut lane_section = LaneSection::new();

    let lane = Lane{id: 0, s: 0.0, width_params: QuadrinomialParams::zero_order(2.0)};
    lane_section.lanes.push(lane);

    road.lane_sections.push(lane_section);
    road
}