use crate::{
    camera::ThreadedCamera, filter::EuroDataFilter, network::SocketNetwork, pose::ProcessHeadPose,
};
use std::sync::{self, atomic::AtomicBool};

pub struct HeadTracker {
    pub keep_running: sync::Arc<AtomicBool>,
    pub filter: EuroDataFilter,
    pub socket: SocketNetwork,
    pub camera_index: i32,
    // pub head_pose: ProcessHeadPose,
}
