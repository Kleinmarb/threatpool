use std::thread;
use std::sync::{mpsc, Arc, Mutex};

pub struct ThreadPool {
    /// A `Vec` of workers which execute the jobs
    workers: Vec<Worker>,

    sender: mpsc::Sender<Job>,
}

/// The actual `Job` executed by a `Worker`
type Job = Option<Box<(dyn FnOnce() + Send + 'static)>>;

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(Worker::new(Arc::clone(&receiver)));
        }

        Self {
            workers,
            sender,
        }
    }

    /// Executes a given job
    ///
    /// # Example
    ///
    /// ```
    /// use threatpool::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4);
    ///
    /// pool.execute(|| {
    ///     // Simulate some task
    ///     println!("Hello, World");
    /// });
    /// ```
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job: Job = Some(Box::new(f));
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            match job {
                Some(job) => {
                    job();
                }

                None => {
                    break; // Breaks if the given job is `None`
                }
            }
        });

        Self {
            thread: Some(thread),
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(None).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[test]
fn main() {
    let pool = ThreadPool::new(10);

    for _ in 0..10000000 {
        pool.execute(|| {
            println!("Hello, World!"); // Simulates some heavy task
        });
    }
}
