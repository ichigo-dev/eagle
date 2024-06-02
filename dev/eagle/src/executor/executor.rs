//------------------------------------------------------------------------------
//! # Async executor
//------------------------------------------------------------------------------

use super::mpmc::{ self, Sender };
use super::task::Task;
use super::worker::Worker;

use std::fmt::{ self, Debug, Display, Formatter };
use std::future::Future;
use std::sync::Arc;
use std::sync::{ Condvar, Mutex, PoisonError };


//------------------------------------------------------------------------------
/// # ExecutorError
//------------------------------------------------------------------------------
pub enum ExecutorError<T>
{
    MpmcError(mpmc::MpmcError<Arc<Mutex<Task<T>>>>),
    PoisonError(String),
    NoResult,
}

impl<T> Debug for ExecutorError<T>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> fmt::Result
    {
        match self
        {
            Self::MpmcError(error) => write!(f, "MpmcError: {:?}", error),
            Self::PoisonError(error) => write!(f, "PoisonError: {:?}", error),
            Self::NoResult => write!(f, "NoResult"),
        }
    }
}

impl<T> Display for ExecutorError<T>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> fmt::Result
    {
        match self
        {
            Self::MpmcError(error) => write!(f, "MpmcError: {}", error),
            Self::PoisonError(error) => write!(f, "PoisonError: {}", error),
            Self::NoResult => write!(f, "NoResult"),
        }
    }
}

impl<T> From<mpmc::MpmcError<Arc<Mutex<Task<T>>>>> for ExecutorError<T>
{
    fn from( error: mpmc::MpmcError<Arc<Mutex<Task<T>>>> ) -> Self
    {
        Self::MpmcError(error)
    }
}

impl<T, E> From<PoisonError<E>> for ExecutorError<T>
{
    fn from( error: PoisonError<E> ) -> Self
    {
        Self::PoisonError(error.to_string())
    }
}


//------------------------------------------------------------------------------
/// # Executor
//------------------------------------------------------------------------------
pub(crate) struct Executor<T>
{
    workers: Vec<Worker<T>>,
    sender: Sender<Arc<Mutex<Task<T>>>>,
    is_done: Arc<(Mutex<Option<T>>, Condvar)>,
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
        let is_done = Arc::new((Mutex::new(None), Condvar::new()));

        for id in 0..num_threads
        {
            let receiver = receiver.clone();
            let worker = Worker::new
            (
                id,
                sender.clone(),
                receiver,
                is_done.clone(),
            );
            workers.push(worker);
        }

        Self
        {
            workers,
            sender,
            is_done,
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
    fn spawn( &self, task: Task<T> ) -> Result<(), ExecutorError<T>>
    {
        let task = Arc::new(Mutex::new(task));
        self.sender.send(task).map_err(ExecutorError::MpmcError)
    }

    //--------------------------------------------------------------------------
    /// Blocks the current thread on the given future.
    //--------------------------------------------------------------------------
    pub(crate) fn block_on<F>
    (
        &self,
        future: F,
    ) -> Result<F::Output, ExecutorError<T>>
        where
            F: Future<Output = T> + Send + 'static,
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

impl<T> Drop for Executor<T>
{
    fn drop( &mut self )
    {
        for worker in &self.workers
        {
            worker.stop();
        }
        for worker in &mut self.workers
        {
            if let Some(thread) = worker.join_handle.take()
            {
                let _ = thread.join();
            }
        }
    }
}
