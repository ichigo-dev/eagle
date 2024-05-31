//------------------------------------------------------------------------------
//! Async executor
//------------------------------------------------------------------------------

use super::mpmc::{ self, Sender };
use super::task::Task;
use super::worker::Worker;

use std::future::Future;
use std::sync::Arc;
use std::sync::{ Condvar, Mutex };


//------------------------------------------------------------------------------
/// Executor
//------------------------------------------------------------------------------
pub(crate) struct Executor<T>
{
    workers: Vec<Worker<T>>,
    sender: Sender<Arc<Mutex<Task<T>>>>,
}

impl<T: Send + 'static> Executor<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Executor.
    //--------------------------------------------------------------------------
    pub(crate) fn new( num_threads: usize ) -> Self
    {
        let (sender, receiver) = mpmc::channel();
        let mut workers = Vec::with_capacity(num_threads);
        for id in 0..num_threads
        {
            let receiver = receiver.clone();
            let worker = Worker::new(id, sender.clone(), receiver);
            workers.push(worker);
        }

        Self
        {
            workers,
            sender,
        }
    }
    
    //--------------------------------------------------------------------------
    /// Runs the worker threads.
    //--------------------------------------------------------------------------
    pub(crate) fn start( &self )
    {
        for worker in &self.workers
        {
            worker.run();
        }
    }

    //--------------------------------------------------------------------------
    /// Spawns a new task.
    //--------------------------------------------------------------------------
    fn spawn( &self, task: Arc<Mutex<Task<T>>> )
    {
        let _ = self.sender.send(task);
    }

    //--------------------------------------------------------------------------
    /// Blocks the current thread on the given future.
    //--------------------------------------------------------------------------
    pub(crate) fn block_on<F>( &self, future: F ) -> F::Output
        where
            F: Future<Output = T> + Send + 'static,
    {
        let future = Box::pin(future);
        let task = Arc::new(Mutex::new(Task::new(future)));

        let lock = Arc::new((Mutex::new(None), Condvar::new()));
        self.spawn(task);

        let (lock, cvar) = &*lock;
        let mut result = lock.lock().unwrap();
        while result.is_none()
        {
            result = cvar.wait(result).unwrap();
        }

        result.take().unwrap()
    }
}
