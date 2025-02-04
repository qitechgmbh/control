use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::{sync::TryLockError, task::JoinSet};

use crate::{signal::Signal, state::State};

/// Schedules a task concurrenlty and evenly based on the average task time.
/// The scheduler is capable of dynamically adjusting the dispatch frequency based on the task duration.
pub struct ResonantScheduler {
    pub state: Arc<State>,
}

impl ResonantScheduler {
    pub fn new(max_concurrent_tasks: usize, initial_avg: Duration, ema_alpha: f64) -> Self {
        Self {
            state: Arc::new(State::new(max_concurrent_tasks, initial_avg, ema_alpha)),
        }
    }

    /// Runs the scheduler along with spawned tasks.
    /// The provided `task_fn` is executed repeatedly in each spawned task.
    /// This function never returns.
    pub async fn run<F, Fut>(&self, task_fn: F) -> Result<(), TryLockError>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Signal> + Send,
    {
        println!("run");
        let mut join_set = JoinSet::new();
        let task_fn = Arc::new(task_fn);

        // Pre-spawn concurrent tasks.
        let available = self.state.semaphore.available_permits();
        for _ in 0..available {
            let task_fn = task_fn.clone();
            let state = self.state.clone();
            join_set.spawn(async move {
                loop {
                    if state.exit.load(Ordering::Relaxed) {
                        break;
                    }
                    let _permit = state.semaphore.acquire().await.unwrap();
                    let start = Instant::now();
                    let feedback = (task_fn)().await;
                    let elapsed = start.elapsed().as_nanos() as u64;
                    Self::update_rolling_average(&state.avg_time, elapsed, &state.ema_alpha);
                    Self::update_count(&state.count);
                    let _ = state.tx.send(feedback);
                }
            });
        }

        join_set.spawn({
            let state = self.state.clone();
            async move {
                let rx = state.rx.clone();
                loop {
                    let avg_duration = Duration::from_nanos(state.avg_time.load(Ordering::Relaxed));
                    let task_interval = avg_duration / state.max_concurrent_tasks as u32;
                    // react to the different feedbacks
                    match rx.lock().await.recv().await {
                        Ok(Signal::Exit) => {
                            state.exit.store(true, Ordering::Relaxed);
                        }
                        Ok(Signal::Continue) => tokio::time::sleep(task_interval).await,
                        Err(_) => {}
                    }
                }
            }
        });

        println!("ResonantScheduler started");

        // join all
        join_set.join_all().await;

        Ok(())
    }

    /// Atomically updates the rolling average using a CAS loop.
    fn update_rolling_average(avg_time: &AtomicU64, elapsed: u64, ema_alpha: &f64) {
        let mut prev_avg = avg_time.load(Ordering::Relaxed);

        loop {
            let new_avg =
                ((ema_alpha * elapsed as f64) + (1.0 - ema_alpha) * prev_avg as f64) as u64;
            println!("new_avg: {}", new_avg);
            match avg_time.compare_exchange_weak(
                prev_avg,
                new_avg,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => prev_avg = actual,
            }
        }
    }

    // update count
    fn update_count(count: &AtomicUsize) {
        count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn signal(&self, feedback: Signal) {
        let state = self.state.clone();
        tokio::spawn(async move {
            let _ = state.tx.send(feedback);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::signal::Signal;

    use super::*;
    use std::println;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn fastest() {
        let task_fn = || async { Signal::Continue };
        let runner = Arc::new(ResonantScheduler::new(16, Duration::from_micros(250), 0.1));

        // start the runner
        tokio::spawn({
            let runner = runner.clone();
            async move {
                runner.run(task_fn).await.unwrap();
            }
        });

        // stop the runner
        let sleep = Duration::from_secs(5);
        tokio::time::sleep(sleep).await;
        runner.signal(Signal::Exit);

        let frequency = runner.state.count.load(Ordering::Relaxed) as f64 / sleep.as_secs_f64();

        println!(
            "Average Task Time: {} ns",
            // nanos to millis
            runner.state.avg_time.load(Ordering::Relaxed) as f64
        );
        println!("Count: {}", runner.state.count.load(Ordering::Relaxed));
        println!("Frequency: {} Hz", frequency);
    }

    #[tokio::test]
    async fn accuracy() {
        let runner = Arc::new(ResonantScheduler::new(16, Duration::from_micros(250), 0.1));

        #[allow(unused)]
        let task_fn = || async {
            tokio::spawn(async {
                let mut cnt = 0;
                for _ in 0..2400 {
                    cnt += 1;
                }
            })
            .await;
            Signal::Continue
        };

        // start the runner
        tokio::spawn({
            let runner = runner.clone();
            async move {
                runner.run(task_fn).await.unwrap();
            }
        });

        // stop the runner
        let sleep = Duration::from_secs(1);
        tokio::time::sleep(sleep).await;
        runner.signal(Signal::Exit);

        let frequency = runner.state.count.load(Ordering::Relaxed) as f64 / sleep.as_secs_f64();

        println!(
            "Average Task Time: {} us",
            // nanos to millis
            runner.state.avg_time.load(Ordering::Relaxed) as f64 / 1_000.0
        );
        println!("Count: {}", runner.state.count.load(Ordering::Relaxed));
        println!("Frequency: {} Hz", frequency);
    }
}
