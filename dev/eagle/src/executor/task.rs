//------------------------------------------------------------------------------
//! # Async task
//!
//! This is the structure of the task handled by the async executor.
//------------------------------------------------------------------------------

use std::future::Future;
use std::pin::Pin;
use std::task::{ Context, Poll };
use std::sync::{ Arc, Mutex };

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;


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
    state: TaskState,
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
            state: TaskState::Ready,
            priority,
        }
    }

    //--------------------------------------------------------------------------
    /// Polls the task.
    //--------------------------------------------------------------------------
    pub(super) fn poll( &mut self, context: &mut Context ) -> Poll<T>
    {
        self.state = TaskState::Running;
        let mut future = self.future.lock().unwrap();
        match future.as_mut().poll(context)
        {
            Poll::Ready(result) =>
            {
                self.state = TaskState::Done;
                Poll::Ready(result)
            },
            Poll::Pending => Poll::Pending
        }
    }
}

impl<T> Task<T>
{
    //--------------------------------------------------------------------------
    /// Gets the state of the task.
    //--------------------------------------------------------------------------
    pub(super) fn state( &self ) -> TaskState
    {
        self.state.clone()
    }

    //--------------------------------------------------------------------------
    /// Gets the priority of the task.
    //--------------------------------------------------------------------------
    pub(super) fn priority( &self ) -> usize
    {
        self.priority
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
