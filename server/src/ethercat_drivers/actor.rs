pub trait Actor: Send + Sync {
    fn act(&mut self, now_ts: u64);
}
