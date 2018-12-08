use super::car::*;
use super::protagonist::*;
use super::primitives::*;
use super::physics::*;
use super::node::*;

use super::msg;
use rosrust::api::raii::Publisher;
use cgmath::*;
use nphysics2d::world::World as PWorld;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};

enum IbeoClassification {
    UNCLASSIFIED,
    UNKNOWN_SMALL,
    UNKNOWN_BIG,
    PEDESTRIAN,
    BIKE,
    CAR,
    TRUCK,
    UNDERIVABLE,
}

struct IbeoVehicleState {
    id: i32,
    pose: Pose2DF64,
    bb_size: Size2f64,
    longitudinal_speed: f64,
}

fn publish_tf_trasl_euler(tf_pub: &mut Publisher<msg::tf2_msgs::TFMessage>, frame: &str, child_frame: &str, 
    x: f64, y: f64, z: f64, roll: f64, pitch: f64, yaw: f64, time: &rosrust::Time) {

    let mut msg = msg::tf2_msgs::TFMessage::default();
    let mut transform = msg::geometry_msgs::TransformStamped::default();

    transform.header.stamp = time.clone();
    transform.header.frame_id = String::from(frame);
    transform.child_frame_id = String::from(child_frame);

    // let car_center = protagonist.pose.center;
    transform.transform.translation.x = x;
    transform.transform.translation.y = y;
    transform.transform.translation.z = z;

    assert!(roll == 0.0);
    assert!(pitch == 0.0);

    transform.transform.rotation.w = (yaw / 2.0).cos();
    transform.transform.rotation.z = (yaw / 2.0).sin();

    msg.transforms.push(transform);
    tf_pub.send(msg).unwrap();
}


pub struct IbeoPublisher {
    ibeo_vehicle_pub: Publisher<msg::ibeo_msgs::ObjectListEcu>,
    tf_pub: Publisher<msg::tf2_msgs::TFMessage>,
    protagonist_odom_pub: Publisher<msg::nav_msgs::Odometry>,
    protagonist_pose_pub: Publisher<msg::geometry_msgs::Pose>,
}

impl IbeoPublisher {
    pub fn try_new() -> Option<IbeoPublisher> {
        let ros_not_available_error_msg = "roscore not started or it is not possible to connect to it";
        let ros_init_result = rosrust::try_init("roadsim2d");
        if ros_init_result.is_err() {
            None            
        } else {
            let ibeo_vehicle_pub = rosrust::publish("/roadsim2d/vehicle_ibeo").expect(ros_not_available_error_msg);
            let tf_pub = rosrust::publish("/tf").expect(ros_not_available_error_msg);
            let protagonist_odom_pub = rosrust::publish("/odom").expect(ros_not_available_error_msg);
            let protagonist_pose_pub = rosrust::publish("/roadsim2d/pose").expect(ros_not_available_error_msg);
            let ibeo_publisher = IbeoPublisher {
                ibeo_vehicle_pub: ibeo_vehicle_pub,
                tf_pub: tf_pub,
                protagonist_odom_pub: protagonist_odom_pub,
                protagonist_pose_pub: protagonist_pose_pub
            };
            Some(ibeo_publisher)
        }
    }
}

pub trait VehicleStatesListener { 
    fn on_protagonist_state<'a>(&'a mut self, protagonist_pose: &'a Pose2DF64, protagonist_speed : f64, protagonist_yaw_rate: f64);
    fn on_vehicle_states<'a>(&'a mut self, protagonist: &'a Car, vehicle_states : &'a Vec<IbeoVehicleState>);
}

impl VehicleStatesListener for IbeoPublisher {

    fn on_protagonist_state<'a>(&'a mut self, protagonist_pose: &'a Pose2DF64, protagonist_speed : f64, protagonist_yaw_rate: f64) {
        // transform.header.stamp = rosrust::now();
        // transform.header.frame_id = String::from("odom");
        // transform.child_frame_id = String::from("base_link");

        let car_center = protagonist_pose.center;
        // transform.transform.translation.x = car_center.x;
        // transform.transform.translation.y = car_center.y;

        // transform.transform.rotation.w = (protagonist.pose.yaw / 2.0).cos();
        // transform.transform.rotation.z = (protagonist.pose.yaw / 2.0).sin();

        // msg.transforms.push(transform);
        // self.tf_pub.send(msg).unwrap();

        let publish_time = rosrust::now();

        publish_tf_trasl_euler(&mut self.tf_pub, "map", "odom", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, &publish_time);
        publish_tf_trasl_euler(&mut self.tf_pub, "odom", "base_link", car_center.x, car_center.y, 0.0, 0.0, 0.0, protagonist_pose.yaw, &publish_time);
        publish_tf_trasl_euler(&mut self.tf_pub, "base_link", "ibeo", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, &publish_time);

        {
            let mut msg = msg::nav_msgs::Odometry {
                header: msg::std_msgs::Header {
                    stamp: publish_time,
                    frame_id: String::from("map"),
                    seq: 0,
                },
                child_frame_id: String::from("odom"),
                pose: msg::geometry_msgs::PoseWithCovariance {
                    pose: msg::geometry_msgs::Pose::default(),
                    covariance: [0.0; 36]
                },
                twist: msg::geometry_msgs::TwistWithCovariance {
                    twist: msg::geometry_msgs::Twist::default(),
                    covariance: [0.0; 36]
                }
            };
            msg.pose.pose.position.x = car_center.x;
            msg.pose.pose.position.y = car_center.y;

            msg.pose.pose.orientation.w = (protagonist_pose.yaw / 2.0).cos();
            msg.pose.pose.orientation.z = (protagonist_pose.yaw / 2.0).sin();

            msg.twist.twist.linear.x = protagonist_speed;
            msg.twist.twist.angular.z = protagonist_yaw_rate;

            self.protagonist_odom_pub.send(msg).unwrap();
        }

       {
            let mut msg = msg::geometry_msgs::Pose::default();
            msg.position.x = car_center.x;
            msg.position.y = car_center.y;

            self.protagonist_pose_pub.send(msg).unwrap();
       }

    }

    fn on_vehicle_states<'a>(&'a mut self, protagonist: &'a Car, vehicle_states : &'a Vec<IbeoVehicleState>) {
        let mut msg = msg::ibeo_msgs::ObjectListEcu::default();
        msg.header.frame_id = String::from("ibeo");
        msg.header.stamp = rosrust::now();
        let protagonist_pose = &protagonist.pose;

        let protagonist_rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(-protagonist_pose.yaw));

        for vehicle_state in vehicle_states {
            let mut object_msg = msg::ibeo_msgs::ObjectListEcuObj::default();
            // note: id is cut to i32 here
            object_msg.id = vehicle_state.id;

            let rel_center = vehicle_state.pose.center - protagonist_pose.center;
            let rotated_rel_center = protagonist_rot.rotate_vector(rel_center);

            object_msg.classification = IbeoClassification::CAR as i32;
            object_msg.bounding_box.pose.x = rotated_rel_center.x;
            object_msg.bounding_box.pose.y = rotated_rel_center.y;
            object_msg.bounding_box.pose.theta = vehicle_state.pose.yaw - std::f64::consts::PI / 2.0 -protagonist_pose.yaw;
            object_msg.bounding_box.size.width = vehicle_state.bb_size.width;
            object_msg.bounding_box.size.height = vehicle_state.bb_size.height;

            // TODO: set speed here
            object_msg.abs_vel.x = vehicle_state.longitudinal_speed;

            msg.objects.push(object_msg);
        }
        self.ibeo_vehicle_pub.send(msg).unwrap();
    }

}


pub struct IbeoSensorSys<'a> {
    pub vehicle_state_listeners : &'a mut Vec<Box<VehicleStatesListener>>,
    pub physics_world: &'a mut PWorld<f64>,
}

impl <'a, 'b> System<'a> for IbeoSensorSys<'b> {
    type SystemData = (
        ReadStorage<'a, Car>,
        ReadStorage<'a, Node>,
        ReadStorage<'a, PhysicsComponent>,
        ReadStorage<'a, ProtagonistTag>,
    );

    fn run(&mut self, (mut cars, nodes, physics_components, protagonists): Self::SystemData) {
        let mut other_car_states = Vec::<IbeoVehicleState>::new(); 

        for (car, node, physics_component, ()) in (&cars, &nodes, &physics_components, !&protagonists).join() {
            let mut rigid_body = self.physics_world.rigid_body_mut(physics_component.body_handle).expect("car rigid body not found");
            let current_speed = rigid_body.velocity().linear.norm();
            let current_yaw_rate = rigid_body.velocity().angular;

            other_car_states.push(IbeoVehicleState{
                id: car.id as i32,
                pose: node.pose.clone(),
                bb_size: car.bb_size,
                longitudinal_speed: current_speed,
            });
        }

        for (car, node, physics_component, _protagonist) in (&cars, &nodes, &physics_components, &protagonists).join() {
            let protagonist_car = car;
            for listener in &mut (self.vehicle_state_listeners).iter_mut() {
                listener.on_vehicle_states(protagonist_car, &other_car_states);


                let mut rigid_body = self.physics_world.rigid_body_mut(physics_component.body_handle).expect("car rigid body not found");
                let current_speed = rigid_body.velocity().linear.norm();
                let current_yaw_rate = rigid_body.velocity().angular;

                listener.on_protagonist_state(&node.pose, current_speed, current_yaw_rate);
            }
        }

    }
}

