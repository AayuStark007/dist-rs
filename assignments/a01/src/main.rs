use rand::Rng;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

static NTHREADS: u32 = 12;
static NTASK: u32 = 100;

struct Task {
    id: u32,
    payload: String,
}

struct Worker {
    id: u32,
}

fn create_task(id: u32, payload: &str) -> Task {
    return Task {
        id,
        payload: payload.to_string(),
    };
}

impl Worker {
    fn process_task(&self, task: Task) -> String {
        // println!("Worker {0} got task {1}", self.id, task.id);

        // simulate task processing
        let delay = rand::thread_rng().gen_range(200..=1000);
        thread::sleep(Duration::from_millis(delay));

        return String::from(format!(
            "[Worker: {0}] Processed {1}::{2} in {3}ms",
            self.id, task.id, task.payload, delay
        ));
    }
}

fn create_worker(id: u32) -> Worker {
    return Worker { id };
}

fn setup_tasks(size: u32) -> Vec<Task> {
    let mut tasks: Vec<Task> = Vec::new();
    for task_id in 0..size {
        let payload = format!("TaskID: {}", task_id);
        tasks.push(create_task(task_id, &payload));
    }
    return tasks;
}

fn setup_worker_threads(
    num_threads: u32,
    receiver: Arc<Mutex<mpsc::Receiver<Task>>>,
    sender: mpsc::Sender<String>,
) -> Vec<JoinHandle<()>> {
    let mut worker_handles = Vec::new();

    // setup threads to handle tasks
    for id in 0..num_threads {
        let task_rx_clone = Arc::clone(&receiver);
        let res_tx_clone = sender.clone();
        let worker = create_worker(id);
        let worker_handle = thread::spawn(move || loop {
            let message = task_rx_clone.lock().unwrap().recv();
            match message {
                Ok(task) => {
                    let task_result = worker.process_task(task);
                    res_tx_clone.send(task_result).unwrap();
                }
                Err(_) => {
                    break;
                }
            }
        });
        worker_handles.push(worker_handle);
    }
    return worker_handles;
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
    let worker_handles = setup_worker_threads(NTHREADS, Arc::new(Mutex::new(task_rx)), result_tx);

    //send tasks
    for task in tasks {
        task_tx.send(task).unwrap();
    }

    drop(task_tx);

    //receive results
    loop {
        let result = result_rx.recv_timeout(Duration::from_secs(5));
        match result {
            Ok(message) => {
                println!("{}", message);
            }
            Err(e) => {
                println!("No more messages. {}", e.to_string());
                break;
            }
        }
    }

    // shutdown threads
    for handle in worker_handles {
        handle.join().unwrap();
    }
}
