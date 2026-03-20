use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct AtomicF64 {
    inner: AtomicU64,
}

impl AtomicF64 {
    fn new(value: f64) -> Self {
        Self {
            inner: AtomicU64::new(value.to_bits()),
        }
    }

    fn load(&self, ordering: Ordering) -> f64 {
        f64::from_bits(self.inner.load(ordering))
    }

    fn store(&self, value: f64, ordering: Ordering) {
        self.inner.store(value.to_bits(), ordering);
    }
}