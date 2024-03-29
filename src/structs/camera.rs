use std::{
    sync::{self, atomic::AtomicBool},
    thread,
};

pub struct ThreadedCamera {
    pub cam_thread: Option<thread::JoinHandle<()>>, // Storing the thread
    pub keep_running: sync::Arc<AtomicBool>,        // Signal to stop the thread
}
