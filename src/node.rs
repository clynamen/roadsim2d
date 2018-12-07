use super::primitives::*;
use specs::{System, DispatcherBuilder, World, Builder, ReadStorage, WriteStorage,
 Read, ReadExpect, WriteExpect, RunNow, Entities, LazyUpdate, Join, VecStorage, Component};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Node {
    pub pose: Pose2DF64
}