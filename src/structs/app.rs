use std::{
    collections::HashMap,
    sync::{
        self,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    thread,
};

use super::config::AppConfig;

pub struct AtomicF32 {
    storage: AtomicU32,
}
impl AtomicF32 {
    pub fn new(value: f32) -> Self {
        let as_u64 = value.to_bits();
        Self {
            storage: AtomicU32::new(as_u64),
        }
    }
    pub fn store(&self, value: f32, ordering: Ordering) {
        let as_u64 = value.to_bits();
        self.storage.store(as_u64, ordering)
    }
    pub fn load(&self, ordering: Ordering) -> f32 {
        let as_u64 = self.storage.load(ordering);
        f32::from_bits(as_u64)
    }
}

pub struct HeadTracker {
    pub min_cutoff: sync::Arc<AtomicF32>,
    pub beta: sync::Arc<AtomicF32>,

    pub ip_arr_0: String,
    pub ip_arr_1: String,
    pub ip_arr_2: String,
    pub ip_arr_3: String,
    pub port: String,

    pub fps: sync::Arc<AtomicU32>,

    pub camera_list: HashMap<String, i32>,
    pub selected_camera: Option<String>,
    pub hide_camera: bool,

    pub headtracker_thread: Option<thread::JoinHandle<()>>,
    pub run_headtracker: sync::Arc<AtomicBool>,

    pub should_exit: bool,
}
