use super::car::*;

//rosmsg_include!();
use super::msg;
use rosrust::api::raii::Publisher;

pub struct IbeoPublisher {
    ibeo_vehicle_pub: Publisher<msg::ibeo_msgs::ObjectListEcu>,
}

impl IbeoPublisher {
    pub fn try_new() -> Option<IbeoPublisher> {
        let ros_not_available_error_msg = "roscore not started or it is not possible to connect to it";
        let ros_init_result = rosrust::try_init("roadsim2d");
        if ros_init_result.is_err() {
            None            
        } else {
            let ibeo_vehicle_pub = rosrust::publish("/roadsim2d/vehicle_ibeo").expect(ros_not_available_error_msg);
            let ibeo_publisher = IbeoPublisher {
                ibeo_vehicle_pub: ibeo_vehicle_pub
            };
            Some(ibeo_publisher)
        }
    }
}

pub trait VehicleStatesListener { 
    fn on_vehicle_states<'a>(&'a mut self, vehicles : Box<dyn Iterator<Item = &'a Car> + 'a>);
}

impl VehicleStatesListener for IbeoPublisher {

    fn on_vehicle_states<'a>(&'a mut self, vehicles : Box<dyn Iterator<Item = &'a Car> + 'a>) {
        let mut msg = msg::ibeo_msgs::ObjectListEcu::default();
        msg.header.frame_id = String::from("base_link");
        for vehicle in vehicles {
            let mut object_msg = msg::ibeo_msgs::ObjectListEcuObj::default();
            object_msg.id = 0;
            object_msg.bounding_box.pose.x = vehicle.pose.center.x;
            object_msg.bounding_box.pose.y = vehicle.pose.center.y;
            msg.objects.push(object_msg);
        }
        self.ibeo_vehicle_pub.send(msg).unwrap();
    }

}
