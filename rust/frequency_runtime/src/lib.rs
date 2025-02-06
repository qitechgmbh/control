use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

// pub struct RoundRobinRuntime {
//     tasks: Vec<Pin<Box<dyn Future<Output = ()> + Send>>>,
//     index: usize,
// }

// impl RoundRobinRuntime {
//     pub fn new() -> Self {
//         RoundRobinRuntime {
//             tasks: Vec::new(),
//             index: 0,
//         }
//     }

//     pub fn spawn<F>(&mut self, task: F)
//     where
//         F: Future<Output = ()> + Send + 'static,
//     {
//         // add future
//         self.tasks.push(Box::pin(task));
//         // set index to last task
//         self.index = self.tasks.len() - 1;
//     }

//     pub fn run(&mut self) {
//         loop {
//             if self.tasks.is_empty() {
//                 println!("No tasks to run.");
//                 break;
//             }

//             let task_count = self.tasks.len();
//             let current_index = self.index;

//             // Poll the current task
//             let task = &mut self.tasks[0];
//             let waker = futures::task::noop_waker_ref();
//             let mut context = Context::from_waker(waker);

//             match task.as_mut().poll(&mut context) {
//                 Poll::Ready(_) => {
//                     println!("Task {} completed.", current_index);
//                     drop(self.tasks.remove(current_index));
//                     // Adjust the index since we removed a task
//                     self.index = current_index % self.tasks.len();
//                 }
//                 Poll::Pending => {
//                     // Move to the next task
//                     self.index = (self.index + 1) % task_count;
//                 }
//             }

//             // set next index to 0 so the oldest task is polled first
//             self.index = 0;

//             // Sleep for a short duration to simulate time passing
//             thread::sleep(Duration::from_millis(100));
//         }
//     }
// }

pub struct FrequencyRuntime {
    frequency: Duration,
    tasks: VecDeque<Task>,
    get_task: Box<dyn Fn() -> Task>,
    index: usize,
}

type Task = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

impl FrequencyRuntime {
    pub fn new(freq: Duration, get_task: Box<dyn Fn() -> Task>) -> Self {
        FrequencyRuntime {
            frequency: freq,
            tasks: VecDeque::new(),
            get_task,
            index: 0,
        }
    }

    pub fn run(&mut self) {
        println!("Running frequency runtime...");
        let started = std::time::Instant::now();
        let mut cycle = 0;
        loop {
            // check if we have to pull a new task
            let new_cycle = started.elapsed().as_nanos() / self.frequency.as_nanos();
            if new_cycle > cycle {
                // pull enw task
                let task = (self.get_task)();
                self.tasks.push_back(task);
                // poll task
                self.poll_task(self.tasks.len() - 1);
            }
            cycle = new_cycle;

            // poll first task
            if !self.tasks.is_empty() {
                self.poll_task(0);
            }

            // after 5 seconds exit
            // if started.elapsed().as_secs() > 5 {
            //     break;
            // }

            // add index
            self.index = match self.tasks.len() {
                0 => 0,
                _ => (self.index + 1) % self.tasks.len(),
            };
        }
    }

    pub fn poll_task(&mut self, i: usize) {
        // println!("Polling task {}... ", i);
        let task = &mut self.tasks[i];
        let waker = futures::task::noop_waker_ref();
        let mut context = Context::from_waker(waker);

        match task.as_mut().poll(&mut context) {
            Poll::Ready(_) => {
                println!("Task {} completed.", i);
                // remove task from list
                let _ = self.tasks.remove(i);
            }
            Poll::Pending => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_frequency_runtime() {
        let get_task = Box::new(|| -> Pin<Box<dyn Future<Output = ()> + Send>> {
            Box::pin(async {
                #[allow(unused_variables)]
                let mut cnt: u64 = 0;
                for i in 0..1000000u64 {
                    cnt += i;
                }
            })
        });

        let mut runtime = FrequencyRuntime::new(Duration::from_millis(100), get_task);

        runtime.run();
    }
}
