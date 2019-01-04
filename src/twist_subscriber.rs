use super::msg;
use rosrust::api::raii::Publisher;
use rosrust::api::raii::Subscriber;

pub struct TwistSubscriber {
    // twist_sub: Subscriber<msg::geometry_msgs::Twist>,
    twist_sub: Subscriber,
}

impl TwistSubscriber {

    pub fn new<F>(mut callback:  F) -> Option<TwistSubscriber> where F : Fn(f64, f64) -> () + Send + 'static {
        let twist_sub = rosrust::subscribe("roadsim2d/protagonist_twist", move |v: msg::geometry_msgs::Twist| {
            callback(v.linear.x as f64, v.angular.z as f64)
        });
        if twist_sub.is_ok() {
            Some(TwistSubscriber {
                twist_sub: twist_sub.unwrap()
            })
        } else {
            None
        }
    }

}
