//------------------------------------------------------------------------------
//! # Async task
//!
//! This is the structure of the task handled by the async executor.
//------------------------------------------------------------------------------

use std::future::Future;
use std::pin::Pin;
use std::task::{ Context, Poll };
use std::sync::{ Arc, Mutex, PoisonError };

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;


//------------------------------------------------------------------------------
/// # TaskError
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) enum TaskError
{
    PoisonError(String),
}

impl<E> From<PoisonError<E>> for TaskError
{
    fn from( error: PoisonError<E> ) -> Self
    {
        Self::PoisonError(format!("{:?}", error))
    }
}


//------------------------------------------------------------------------------
/// # TaskState
///
/// - Ready: The task is ready to be polled.
/// - Running: The task is currently running.
/// - Done: The task has completed.
//------------------------------------------------------------------------------
#[derive(Clone)]
pub(crate) enum TaskState
{
    Ready,
    Running,
    Done,
}


//------------------------------------------------------------------------------
/// # Task
//------------------------------------------------------------------------------
#[derive(Clone)]
pub(crate) struct Task<T>
{
    future: Arc<Mutex<BoxFuture<T>>>,
    state: Arc<Mutex<TaskState>>,
    priority: usize,
}

impl<T: Send + Clone + 'static> Task<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Task.
    //--------------------------------------------------------------------------
    pub(super) fn new<F>( future: F ) -> Self
        where F: Future<Output = T> + Send + 'static
    {
        Self::with_priority(future, 0)
    }

    //--------------------------------------------------------------------------
    /// Creates a new Task with a priority.
    //--------------------------------------------------------------------------
    pub(super) fn with_priority<F>( future: F, priority: usize ) -> Self
        where F: Future<Output = T> + Send + 'static
    {
        Self
        {
            future: Arc::new(Mutex::new(Box::pin(future))),
            state: Arc::new(Mutex::new(TaskState::Ready)),
            priority,
        }
    }

    //--------------------------------------------------------------------------
    /// Polls the task.
    //--------------------------------------------------------------------------
    pub(super) fn poll
    (
        &mut self,
        context: &mut Context,
    ) -> Result<Poll<T>, TaskError>
    {
        let mut state = self.state.lock()?;
        *state = TaskState::Running;
        let mut future = self.future.lock()?;
        match future.as_mut().poll(context)
        {
            Poll::Ready(result) =>
            {
                *state = TaskState::Done;
                Ok(Poll::Ready(result))
            },
            Poll::Pending => Ok(Poll::Pending)
        }
    }
}

impl<T> PartialEq for Task<T>
{
    fn eq( &self, other: &Self ) -> bool
    {
        self.priority == other.priority
    }
}

impl<T> Eq for Task<T> {}

impl<T> PartialOrd for Task<T>
{
    fn partial_cmp( &self, other: &Self ) -> Option<std::cmp::Ordering>
    {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Task<T>
{
    fn cmp( &self, other: &Self ) -> std::cmp::Ordering
    {
        self.priority.cmp(&other.priority)
    }
}
