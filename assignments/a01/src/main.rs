use rand::Rng;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

static NTHREADS: usize = 1000;
static NTASK: usize = 10_000_000;

#[derive(Debug, Clone)]
struct Task {
    id: u32,
    payload: String,
}

fn create_task(id: u32, payload: &str) -> Task {
    return Task {
        id,
        payload: payload.to_string(),
    };
}

struct Worker {
    id: u32,
}

impl Worker {
    fn process_task(&self, task: Task) -> String {
        // println!("Worker {0} got task {1}", self.id, task.id);

        // simulate task processing
        let delay = rand::thread_rng().gen_range(200..=1000);
        // thread::sleep(Duration::from_millis(delay));

        return String::from(format!(
            "[Worker: {0}] Processed {1}::{2} in {3}ms",
            self.id, task.id, task.payload, delay
        ));
    }
}

fn create_worker(id: u32) -> Worker {
    return Worker { id };
}

fn setup_tasks(size: usize) -> Vec<Task> {
    let mut tasks: Vec<Task> = Vec::new();
    for task_id in 0..size {
        let payload = format!("TaskID: {}", task_id);
        tasks.push(create_task(task_id.try_into().unwrap(), &payload));
    }
    return tasks;
}

fn setup_worker_threads(
    num_threads: usize,
    receiver: mpsc::Receiver<Task>,
    sender: mpsc::Sender<String>,
) -> Vec<JoinHandle<()>> {
    // setup threads to handle tasks
    let shared_receiver = Arc::new(Mutex::new(receiver));
    return (0..num_threads)
        .map(|id| {
            let receiver_clone = Arc::clone(&shared_receiver);
            let sender_clone = sender.clone();
            let worker = create_worker(id.try_into().unwrap());
            let worker_handle = thread::spawn(move || loop {
                let mutex_result = receiver_clone.lock();
                match mutex_result {
                    Ok(receiver) => {
                        let message = receiver.recv_timeout(Duration::from_secs(2));
                        drop(receiver);
                        match message {
                            Ok(task) => {
                                let task_result = worker.process_task(task.clone());
                                sender_clone.send(task_result).unwrap_or_else(|_| {
                                    println!(
                                        "Thread {} failed to send task result for {}",
                                        id, task.id
                                    );
                                });
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                println!("Thread {} timed out on receiver", id);
                                continue;
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => {
                                // channel is closed, thread can terminate
                                return;
                            }
                        }
                    }
                    Err(_) => {
                        println!("Thread failed to acquire lock, retrying");
                    }
                }
            });
            worker_handle
        })
        .collect();
}

fn main() {
    println!(
        "Start processing with {0} tasks on {1} worker threads",
        NTASK, NTHREADS
    );

    // setup channel from main thread to worker thread
    let (task_tx, task_rx) = mpsc::channel::<Task>();
    let (result_tx, result_rx) = mpsc::channel::<String>();

    let tasks = setup_tasks(NTASK);
    let worker_handles = setup_worker_threads(NTHREADS, task_rx, result_tx);

    //send tasks
    for task in tasks {
        task_tx.send(task).unwrap_or_else(|_| {
            println!("Failed to submit a task");
        });
    }

    drop(task_tx);

    //receive results
    loop {
        let result = result_rx.recv_timeout(Duration::from_secs(5));
        match result {
            Ok(message) => {
                println!("{}", message);
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!("No more messages");
                break;
            }
            Err(e) => {
                println!("Result read failed {}.", e.to_string());
                break;
            }
        }
    }

    // shutdown threads
    for handle in worker_handles {
        handle.join().unwrap();
    }
}
