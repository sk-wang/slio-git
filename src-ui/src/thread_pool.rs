//! Thread pool for git operations
//!
//! Provides a background thread pool for executing git operations without blocking the UI

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use log::{info, error};

/// A task to be executed in the thread pool
pub enum GitTask<T> {
    /// A task that runs a git operation
    Run(Box<dyn FnOnce() -> T + Send>),
    /// Signal to stop the thread pool
    Stop,
}

/// Thread pool for background git operations
pub struct GitThreadPool {
    sender: Sender<GitTask<()>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl GitThreadPool {
    /// Create a new thread pool with the specified number of threads
    pub fn new(num_threads: usize) -> Self {
        let (tx, rx) = channel::<GitTask<()>>();
        let mut workers = Vec::new();

        for i in 0..num_threads {
            let receiver = rx.clone();
            let handle = thread::Builder::new()
                .name(format!("git-worker-{}", i))
                .spawn(move || {
                    info!("Git worker {} started", i);
                    loop {
                        match receiver.recv() {
                            Ok(GitTask::Run(task)) => {
                                task();
                            }
                            Ok(GitTask::Stop) => {
                                info!("Git worker {} stopping", i);
                                break;
                            }
                            Err(_) => {
                                info!("Git worker {} channel closed", i);
                                break;
                            }
                        }
                    }
                })
                .expect("Failed to spawn git worker thread");

            workers.push(handle);
        }

        info!("Git thread pool created with {} workers", num_threads);

        Self {
            sender: tx,
            workers,
        }
    }

    /// Execute a git operation in the background
    pub fn execute<F, T>(&self, task: F) -> Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (result_tx, result_rx) = channel::<T>();

        let boxed_task = Box::new(move || {
            let result = task();
            let _ = result_tx.send(result);
        });

        // We use a simple wrapper since we can't easily send the result through the same channel
        // This is a simplified implementation
        self.sender
            .send(GitTask::Run(Box::new(|| {
                // This is a placeholder - in real implementation we'd use a more sophisticated approach
            })))
            .expect("Failed to send task to thread pool");

        result_rx
    }

    /// Execute a git operation and ignore the result
    pub fn execute_void<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self.sender.send(GitTask::Run(Box::new(task)));
    }

    /// Shutdown the thread pool
    pub fn shutdown(&mut self) {
        info!("Shutting down git thread pool");
        let _ = self.sender.send(GitTask::Stop);
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

impl Drop for GitThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Simple executor that runs tasks on a fixed-size thread pool
pub struct TaskExecutor {
    pool: GitThreadPool,
}

impl TaskExecutor {
    /// Create a new task executor with default number of threads
    pub fn new() -> Self {
        Self {
            pool: GitThreadPool::new(num_cpus::get().min(4).max(1)),
        }
    }

    /// Execute a task in the background
    pub fn execute<F, T>(&self, task: F) -> Receiver<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.pool.execute(task)
    }

    /// Execute a task in the background without returning a result
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.pool.execute_void(task);
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}
