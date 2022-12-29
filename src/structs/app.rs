use std::sync::{
    self,
    atomic::{AtomicBool, Ordering, AtomicU32},
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
    pub keep_running: sync::Arc<AtomicBool>,
    pub beta: sync::Arc<AtomicF32>,
    pub min_cutoff: sync::Arc<AtomicF32>,
    pub camera_index: i32,
    pub cfg : AppConfig
}
