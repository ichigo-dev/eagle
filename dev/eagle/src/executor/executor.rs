//------------------------------------------------------------------------------
//! # Async executor
//------------------------------------------------------------------------------

use super::reactor::Reactor;
use super::task::Task;
use super::task_queue::{ TaskQueue, TaskQueueError };
use super::worker::Worker;

use std::future::Future;
use std::sync::Arc;
use std::sync::{ Condvar, Mutex, PoisonError };


//------------------------------------------------------------------------------
/// # ExecutorError
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) enum ExecutorError
{
    TaskQueueError(TaskQueueError),
    PoisonError(String),
    NoResult,
}

impl From<TaskQueueError> for ExecutorError
{
    fn from( error: TaskQueueError ) -> Self
    {
        Self::TaskQueueError(error)
    }
}

impl<E> From<PoisonError<E>> for ExecutorError
{
    fn from( error: PoisonError<E> ) -> Self
    {
        Self::PoisonError(error.to_string())
    }
}


//------------------------------------------------------------------------------
/// # Executor
//------------------------------------------------------------------------------
pub(crate) struct Executor<T: Clone>
{
    workers: Vec<Worker<T>>,
    queue: TaskQueue<T>,
    is_done: Arc<(Mutex<Option<T>>, Condvar)>,
    reactor: Reactor,
}

impl<T: Send + Clone + 'static> Executor<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Executor.
    //--------------------------------------------------------------------------
    pub(crate) fn new( num_threads: usize ) -> Self
    {
        let queue = TaskQueue::new();
        let mut workers = Vec::with_capacity(num_threads);
        let is_done = Arc::new((Mutex::new(None), Condvar::new()));

        for id in 0..num_threads
        {
            let worker = Worker::new
            (
                id,
                (&queue).clone(),
                is_done.clone(),
            );
            workers.push(worker);
        }

        Self
        {
            workers,
            queue,
            is_done,
            reactor: Reactor::new(),
        }
    }
    
    //--------------------------------------------------------------------------
    /// Runs the worker threads.
    //--------------------------------------------------------------------------
    pub(crate) fn start( &mut self )
    {
        for worker in &mut self.workers
        {
            worker.run();
        }
    }

    //--------------------------------------------------------------------------
    /// Spawns a new task.
    //--------------------------------------------------------------------------
    fn spawn( &self, task: Task<T> ) -> Result<(), ExecutorError>
    {
        self.queue.push(task)?;
        Ok(())
    }

    //--------------------------------------------------------------------------
    /// Blocks the current thread on the given future.
    //--------------------------------------------------------------------------
    pub(crate) fn block_on<F>
    (
        &self,
        future: F,
    ) -> Result<F::Output, ExecutorError>
        where
            F: Future<Output = T> + Send + 'static,
            T: Clone,
    {
        let task = Task::new(future);
        self.spawn(task)?;

        let (lock, cvar) = &*self.is_done;
        let mut result = lock.lock()?;
        while result.is_none()
        {
            result = cvar.wait(result)?;
        }

        let result = match result.take()
        {
            Some(result) => result,
            None =>
            {
                return Err(ExecutorError::NoResult);
            }
        };
        Ok(result)
    }
}

impl<T: Clone> Drop for Executor<T>
{
    fn drop( &mut self )
    {
        for worker in &mut self.workers
        {
            worker.stop();
            if let Some(thread) = worker.join_handle.take()
            {
                let _ = thread.join();
            }
        }
    }
}
