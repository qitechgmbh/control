use std::{thread::sleep, time::Duration};

pub fn init_jemalloc_stats() {
    std::thread::Builder::new()
        .name("jemalloc-stats".to_string())
        .spawn(|| {
            let mut last_allocated = 0usize;
            let mut last_time = std::time::Instant::now();

            loop {
                smol::block_on(async {
                    sleep(Duration::from_secs(10));
                    // Advance epoch and get allocated bytes
                    let _ = tikv_jemalloc_ctl::epoch::advance();

                    if let Ok(allocated_bytes) =
                        unsafe { tikv_jemalloc_ctl::raw::read::<usize>(b"stats.allocated\0") }
                    {
                        let current_time = std::time::Instant::now();
                        let time_diff = current_time.duration_since(last_time).as_secs_f64();

                        let change = allocated_bytes as i64 - last_allocated as i64;
                        let change_per_sec = if time_diff > 0.0 {
                            (change as f64 / time_diff) as i64
                        } else {
                            0
                        };

                        let change_str = if change_per_sec >= 0 {
                            format!("+{} KB/s", format_bytes(change_per_sec))
                        } else {
                            format!("-{} KB/s", format_bytes(-change_per_sec))
                        };

                        let formatted = format_bytes(allocated_bytes as i64);

                        tracing::debug!("Memory: {} KB ({})", formatted, change_str);
                        last_allocated = allocated_bytes;
                        last_time = current_time;
                    }
                });
            }
        })
        .expect("Failed to spawn jemalloc-stats thread");
}

fn format_bytes(bytes: i64) -> String {
    let kb = bytes / 1024;
    format!("{}", kb)
        .chars()
        .rev()
        .enumerate()
        .map(|(i, c)| {
            if i > 0 && i % 3 == 0 {
                format!("_{}", c)
            } else {
                c.to_string()
            }
        })
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>()
}
