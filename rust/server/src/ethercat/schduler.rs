use futures::future::join_all;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

type Task = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>;

pub async fn frequent_timer(task: Task, amount: usize, interval: Duration) {
    // Create communication channels and control flags
    let keep_running = Arc::new(AtomicBool::new(true));

    // Start precision timing thread

    // Prespawn worker tasks
    let task = Arc::new(task);
    let mut workers = Vec::new();
    let mut trigger_txs = Vec::new();
    for i in 0..4 {
        let (trigger_tx, trigger_rx) = mpsc::unbounded_channel::<()>();
        trigger_txs.push(trigger_tx);
        let kr = keep_running.clone();
        workers.push(create_worker(trigger_rx, i, kr, task.clone()));
    }

    start_timing_thread(trigger_txs, keep_running.clone(), interval);

    // Set up external shutdown (example using simple sleep)
    let kr_external = keep_running.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(20)).await;
        kr_external.store(false, Ordering::Relaxed);
    });

    // Wait for all workers to complete
    join_all(workers).await;
}

fn start_timing_thread(
    trigger_txs: Vec<mpsc::UnboundedSender<()>>,
    kr: Arc<AtomicBool>,
    interval: Duration,
) {
    std::thread::spawn(move || {
        let mut next = Instant::now();
        let mut next_i = 0;
        let mut spawn_times = Vec::new();

        loop {
            // Update target time first to account for any drift
            next += interval;

            // Busy-wait with spin loop hint
            while Instant::now() < next {
                std::hint::spin_loop();
            }

            // Check if we should stop sending triggers
            if !kr.load(Ordering::Relaxed) {
                break;
            }

            // Send trigger to workers (non-blocking)
            if let Err(e) = trigger_txs[next_i].send(()) {
                eprintln!("Trigger send failed: {}", e);
                break;
            }
            spawn_times.push(Instant::now());
            next_i = (next_i + 1) % 4;
        }
    });
}
fn create_worker(
    mut trigger_rx: mpsc::UnboundedReceiver<()>,
    i: u8,
    kr: Arc<AtomicBool>,
    task: Arc<Task>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(_) = trigger_rx.recv().await {
            // Execute task and check continuation condition
            (*task)().await;
            // if !(*task)().await {
            //     kr.store(false, Ordering::Relaxed);
            //     break;
            // }
        }
    })
}

async fn async_task() -> bool {
    // Your actual task logic here
    // Return false to stop further execution

    // Example: stop after 1 million executions
    static mut COUNT: u32 = 0;
    unsafe {
        COUNT += 1;
        if COUNT >= 1_000_000 {
            println!("Reached execution limit");
            return false;
        }
    }

    true
}
