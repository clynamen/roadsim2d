use super::car::*;

use super::msg;
use rosrust::api::raii::Publisher;
use cgmath::*;

fn publish_tf_trasl_euler(tf_pub: &mut Publisher<msg::tf2_msgs::TFMessage>, frame: &str, child_frame: &str, 
    x: f64, y: f64, z: f64, 
    roll: f64, pitch: f64, yaw: f64) {

    let mut msg = msg::tf2_msgs::TFMessage::default();
    let mut transform = msg::geometry_msgs::TransformStamped::default();

    transform.header.stamp = rosrust::now();
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
    fn on_protagonist_state<'a>(&'a mut self, protagonist: &'a Car);
    fn on_vehicle_states<'a>(&'a mut self, protagonist: &'a Car, vehicles : Box<dyn Iterator<Item = &'a Car> + 'a>);
}

impl VehicleStatesListener for IbeoPublisher {

    fn on_protagonist_state<'a>(&'a mut self, protagonist: &'a Car) {

        // transform.header.stamp = rosrust::now();
        // transform.header.frame_id = String::from("odom");
        // transform.child_frame_id = String::from("base_link");

        let car_center = protagonist.pose.center;
        // transform.transform.translation.x = car_center.x;
        // transform.transform.translation.y = car_center.y;

        // transform.transform.rotation.w = (protagonist.pose.yaw / 2.0).cos();
        // transform.transform.rotation.z = (protagonist.pose.yaw / 2.0).sin();

        // msg.transforms.push(transform);
        // self.tf_pub.send(msg).unwrap();
        publish_tf_trasl_euler(&mut self.tf_pub, "map", "odom", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        publish_tf_trasl_euler(&mut self.tf_pub, "odom", "base_link", car_center.x, car_center.y, 0.0, 0.0, 0.0, protagonist.pose.yaw);
        publish_tf_trasl_euler(&mut self.tf_pub, "base_link", "ibeo", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        {
            let mut msg = msg::nav_msgs::Odometry {
                header: msg::std_msgs::Header {
                    stamp: rosrust::now(),
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

            msg.pose.pose.orientation.w = (protagonist.pose.yaw / 2.0).cos();
            msg.pose.pose.orientation.z = (protagonist.pose.yaw / 2.0).sin();
            msg.twist.twist.linear.x = protagonist.longitudinal_speed as f64;
            msg.twist.twist.angular.z = protagonist.yaw_rate as f64;

            self.protagonist_odom_pub.send(msg).unwrap();
        }

       {
            let mut msg = msg::geometry_msgs::Pose::default();
            // msg.header.stamp = rosrust::now();
            // msg.header.frame_id = "map";
            msg.position.x = car_center.x;
            msg.position.y = car_center.y;

            self.protagonist_pose_pub.send(msg).unwrap();
       }

    }

    fn on_vehicle_states<'a>(&'a mut self, protagonist: &'a Car, vehicles : Box<dyn Iterator<Item = &'a Car> + 'a>) {
        let mut msg = msg::ibeo_msgs::ObjectListEcu::default();
        msg.header.frame_id = String::from("ibeo");
        msg.header.stamp = rosrust::now();
        let protagonist_pose = &protagonist.pose;

        let protagonist_rot : Basis2<_> = Rotation2::<f64>::from_angle(Rad(-protagonist_pose.yaw));

        for vehicle in vehicles {
            let mut object_msg = msg::ibeo_msgs::ObjectListEcuObj::default();
            // note: id is cut to i32 here
            object_msg.id = vehicle.id as i32;

            let rel_center = vehicle.pose.center - protagonist_pose.center;
            let rotated_rel_center = protagonist_rot.rotate_vector(rel_center);

            object_msg.bounding_box.pose.x = rotated_rel_center.x;
            object_msg.bounding_box.pose.y = rotated_rel_center.y;
            object_msg.bounding_box.pose.theta = vehicle.pose.yaw - std::f64::consts::PI / 2.0 -protagonist_pose.yaw;
            object_msg.bounding_box.size.width = vehicle.bb_size.width;
            object_msg.bounding_box.size.height = vehicle.bb_size.height;
            object_msg.abs_vel.x = vehicle.longitudinal_speed as f64;
            msg.objects.push(object_msg);
        }
        self.ibeo_vehicle_pub.send(msg).unwrap();
    }

}
