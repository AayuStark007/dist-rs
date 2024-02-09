use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

static NTHREADS: u32 = 2;

struct Task<'a> {
    id: u32,
    payload: &'a str,
}

struct Worker {
    id: u32,
}

fn create_task(id: u32, payload: &str) -> Task {
    return Task { id, payload };
}

impl Worker {
    fn process_task(&self, task: Task) -> String {
        // simulate task processing
        println!("Worker {0} got task {1}", self.id, task.id);
        return String::from(format!("processed: {0} => {1}", task.id, task.payload));
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

    let tasks: Vec<Task> = vec![
        create_task(0, "zero!"),
        create_task(1, "one!"),
        create_task(2, "two!"),
        create_task(3, "three!"),
        create_task(4, "four!"),
        create_task(5, "five!"),
    ];

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
        println!("Received result: {}", result);
    }
}
