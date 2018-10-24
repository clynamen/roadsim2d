use super::car::*;

use super::msg;
use rosrust::api::raii::Publisher;

pub struct IbeoPublisher {
    ibeo_vehicle_pub: Publisher<msg::ibeo_msgs::ObjectListEcu>,
    protagonist_tf_pub: Publisher<msg::tf2_msgs::TFMessage>,
}

impl IbeoPublisher {
    pub fn try_new() -> Option<IbeoPublisher> {
        let ros_not_available_error_msg = "roscore not started or it is not possible to connect to it";
        let ros_init_result = rosrust::try_init("roadsim2d");
        if ros_init_result.is_err() {
            None            
        } else {
            let ibeo_vehicle_pub = rosrust::publish("/roadsim2d/vehicle_ibeo").expect(ros_not_available_error_msg);
            let protagonist_tf_pub = rosrust::publish("/tf").expect(ros_not_available_error_msg);
            let ibeo_publisher = IbeoPublisher {
                ibeo_vehicle_pub: ibeo_vehicle_pub,
                protagonist_tf_pub: protagonist_tf_pub,
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
        let mut msg = msg::tf2_msgs::TFMessage::default();
        let mut transform = msg::geometry_msgs::TransformStamped::default();
        transform.header.stamp = rosrust::now();
        transform.header.frame_id = String::from("odom");
        transform.child_frame_id = String::from("base_link");
        let car_center = protagonist.pose.center;
        transform.transform.translation.x = car_center.x;
        transform.transform.translation.y = car_center.y;
        transform.transform.rotation.w = 1.0;
        msg.transforms.push(transform);
        self.protagonist_tf_pub.send(msg).unwrap();
    }

    fn on_vehicle_states<'a>(&'a mut self, protagonist: &'a Car, vehicles : Box<dyn Iterator<Item = &'a Car> + 'a>) {
        println!("vehicle state");
        let mut msg = msg::ibeo_msgs::ObjectListEcu::default();
        msg.header.frame_id = String::from("ibeo");
        msg.header.stamp = rosrust::now();
        let protagonist_pose = &protagonist.pose;
        for vehicle in vehicles {
            let mut object_msg = msg::ibeo_msgs::ObjectListEcuObj::default();
            // note: id is cut to i32 here
            object_msg.id = vehicle.id as i32;

            let rel_center = vehicle.pose.center - protagonist_pose.center;
            

            object_msg.bounding_box.pose.x = rel_center.x;
            object_msg.bounding_box.pose.y = rel_center.y;
            object_msg.bounding_box.pose.theta = vehicle.pose.yaw - std::f64::consts::PI / 2.0;
            object_msg.bounding_box.size.width = vehicle.bb_size.width;
            object_msg.bounding_box.size.height = vehicle.bb_size.height;
            object_msg.abs_vel.x = vehicle.longitudinal_speed as f64;
            msg.objects.push(object_msg);
        }
        self.ibeo_vehicle_pub.send(msg).unwrap();
    }

}
