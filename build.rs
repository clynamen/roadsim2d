#[macro_use]
extern crate rosrust_codegen;

// ibeo_msgs/BoundingBox2D
// ibeo_msgs/ObjectListEcu
// ibeo_msgs/ObjectListEcuObj
// ibeo_msgs/Size2D
// ibeo_msgs/Vector2D

rosmsg_main!("ibeo_msgs/ObjectListEcu", "tf2_msgs/TFMessage", "geometry_msgs/Twist", "geometry_msgs/Pose"); 
    //"nav_msgs/Odometry", "geometry_msgs/PoseWithCovariance", "geometry_msgs/TwistWithCovariance"