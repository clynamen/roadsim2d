use super::primitives::*;
use super::sim_id::*;
use piston_window::*;
// use std::vec;

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

pub struct Road {
    id: u64,
    junction: i64,
    geometries: Vec<RoadGeometry>,
    lane_sections: Vec<LaneSection>,
}


pub fn draw_road(context: Context, graphics: &mut G2d, 
                    road: &Road)  {
        // note: some vector must be reversed due to the fact that piston_2d Y points toward bottom of screen
        //let reverse_y_center = Point2f64{x: center.x, y: -center.y};
        //let reverse_y_rot = -rot;
        //let car_center = reverse_y_center + Vec2f64{x: car_size.height/2.0, y: car_size.width/2.0};
        //let center = context.transform.trans(car_center.x, car_center.y);
        // let square = rectangle::square(0.0, 0.0, 100.0);

        for geometry in &road.geometries {
            match geometry.sub {
                RoadGeometrySub::Line{} => {
                    let start = context.transform.trans(geometry.base.origin.x,
                        geometry.base.origin.y);
                    for lane_section in &road.lane_sections {
                        let lane_y_start : f32 = lane_section.lanes.iter()
                            .filter(|x| x.id < 0)
                            .map(|x| x.width_params.a).sum();
                        let mut lane_y_sum = -lane_y_start;
                        let mut lane_index = 0;
                        for lane in &lane_section.lanes {
                            let color = [0.2 * (lane_index as f32), 0.2f32, 0.2f32, 1.0f32];
                            rectangle( color, 
                                        [0.0, 
                                        lane_y_sum as f64, 
                                        geometry.base.length as f64,
                                        lane.width_params.a as f64],
                                        start.rot_rad(geometry.base.yaw),
                                        graphics);
                            lane_y_sum += lane.width_params.a;
                            lane_index += 1;
                        }
                    }
                    
                }
                _ => {

                }
            }
        };

        // rectangle( [0.0, 1.0, 0.0, 1.0],
        //             [-1.0, -1.0, 2.0, 2.0], center, graphics);
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

pub fn generate_random_road(id_provider: &mut IdProvider) -> Road {
    let mut road = Road::new(id_provider);
    let mut lane_section = LaneSection::new();

    let geometry = RoadGeometry {
        base: RoadGeometryBase {
            s: 0.0,
            origin: Vec2f64{x: 0.0, y: 0.0},
            yaw: 1.0,
            length: 5.0
        },
        sub: RoadGeometrySub::Line {

        } 
    };

    road.geometries.push(geometry);

    lane_section.lanes.push(Lane{id: -1, s: 0.0, width_params: QuadrinomialParams::zero_order(0.5)});
    lane_section.lanes.push(Lane{id: 0, s: 0.0, width_params: QuadrinomialParams::zero_order(0.0)});
    lane_section.lanes.push(Lane{id: 1, s: 0.0, width_params: QuadrinomialParams::zero_order(2.0)});

    road.lane_sections.push(lane_section);

    road
}