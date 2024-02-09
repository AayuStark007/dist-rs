use rand::Rng;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
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

fn main() {
    // setup channel from main thread to worker thread
    let (task_tx, task_rx): (Sender<Task>, Receiver<Task>) = mpsc::channel::<Task>();
    let (res_tx, res_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let shared_task_rx = Arc::new(Mutex::new(task_rx));

    let mut thread_stash = Vec::new();

    let mut tasks: Vec<Task> = Vec::new();
    for task_id in 0..NTASK {
        let payload = format!("TaskID: {}", task_id);
        tasks.push(create_task(task_id, &payload));
    }

    println!(
        "Start processing with {0} tasks on {1} worker threads",
        NTASK, NTHREADS
    );

    // setup threads to handle tasks
    for id in 0..NTHREADS {
        let task_rx_clone = Arc::clone(&shared_task_rx);
        let res_tx_clone = res_tx.clone();
        let thread = thread::spawn(move || loop {
            let task = task_rx_clone.lock().unwrap().recv().unwrap();
            let task_result = create_worker(id).process_task(task);
            res_tx_clone.send(task_result).unwrap();
        });
        thread_stash.push(thread);
    }

    //send tasks
    for task in tasks {
        task_tx.send(task).unwrap();
    }

    //receive results
    for result in res_rx {
        println!("{}", result);
    }
}
