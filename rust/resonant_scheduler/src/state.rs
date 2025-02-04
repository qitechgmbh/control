use std::{
    sync::{
        atomic::{AtomicU64, AtomicUsize},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    Mutex, Semaphore,
};

use crate::signal::Signal;

/// Shared state for managing concurrent tasks and updating the rolling average.
pub struct State {
    pub semaphore: Arc<Semaphore>,
    /// nanoseconds
    pub avg_time: Arc<AtomicU64>,
    pub tx: Arc<Sender<Signal>>,
    pub rx: Arc<Mutex<Receiver<Signal>>>,
    pub count: Arc<AtomicUsize>,
    pub max_concurrent_tasks: usize,
    pub ema_alpha: f64,
}

impl State {
    pub fn new(max_concurrent_tasks: usize, initial_avg: Duration, ema_alpha: f64) -> Self {
        let (tx, rx) = broadcast::channel::<Signal>(max_concurrent_tasks);
        State {
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            avg_time: Arc::new(AtomicU64::new(initial_avg.as_nanos() as u64)),
            tx: Arc::new(tx),
            rx: Arc::new(Mutex::new(rx)),
            count: Arc::new(AtomicUsize::new(0)),
            max_concurrent_tasks,
            ema_alpha,
        }
    }
}
