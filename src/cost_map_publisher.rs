// use super::town::*;
// use super::msg;
// use rosrust::api::raii::Publisher;
// use std::time;


// // fn publish_tf_trasl_euler(tf_pub: &mut Publisher<msg::tf2_msgs::TFMessage>, frame: &str, child_frame: &str, 
// //     x: f64, y: f64, z: f64, roll: f64, pitch: f64, yaw: f64, time: &rosrust::Time) {

// //     let mut msg = msg::tf2_msgs::TFMessage::default();
// //     let mut transform = msg::geometry_msgs::TransformStamped::default();

// //     transform.header.stamp = time.clone();
// //     transform.header.frame_id = String::from(frame);
// //     transform.child_frame_id = String::from(child_frame);

// //     // let car_center = protagonist.pose.center;
// //     transform.transform.translation.x = x;
// //     transform.transform.translation.y = y;
// //     transform.transform.translation.z = z;

// //     assert!(roll == 0.0);
// //     assert!(pitch == 0.0);

// //     transform.transform.rotation.w = (yaw / 2.0).cos();
// //     transform.transform.rotation.z = (yaw / 2.0).sin();

// //     msg.transforms.push(transform);
// //     tf_pub.send(msg).unwrap();
// // }

// pub fn make_costmap_msg_from_town_gridmap(town_gridmap: &TownGridMap) -> msg::cost_map_msgs::CostMap {
//     let mut msg = msg::cost_map_msgs::CostMap::default();
//     msg.header.stamp = rosrust::now();
//     msg.header.frame_id = String::from("map");

//     msg.layers.push_back(String::from("costmap"));
//     for x in 0..town_gridmap.width {
//         for y in 0..town_gridmap.height {
//             gridmap.has_vertex( &(x as usize, y as usize) ) as u8;

//         }
//     }

//     msg
// }

// pub struct CostMapPublisher {
//     cost_map_pub: Publisher<msg::cost_map_msgs::CostMap>,
// }

// impl CostMapPublisher {
//     pub fn try_new() -> Option<CostMapPublisher> {
//         let cost_map_pub = rosrust::publish("/roadsim2d/costmap").expect(ros_not_available_error_msg);
//         let cost_map_publisher = CostMapPublisher {
//             cost_map_pub: cost_map_pub,
//         };
//         Some(cost_map_publisher)
//     }

//     pub fn publish_town_gridmap_as_costmap(town_gridmap: &TownGridMap) {

//     }
// }