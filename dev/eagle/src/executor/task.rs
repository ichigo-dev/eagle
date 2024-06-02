//------------------------------------------------------------------------------
//! # Async task
//!
//! This is the structure of the task handled by the async executor.
//------------------------------------------------------------------------------

use std::future::Future;
use std::pin::Pin;
use std::task::{ Context, Poll };

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;


//------------------------------------------------------------------------------
/// # TaskState
///
/// - Ready: The task is ready to be polled.
/// - Running: The task is currently running.
/// - Done: The task has completed.
//------------------------------------------------------------------------------
pub(crate) enum TaskState
{
    Ready,
    Running,
    Done,
}


//------------------------------------------------------------------------------
/// # Task
//------------------------------------------------------------------------------
pub(crate) struct Task<T>
{
    future: BoxFuture<T>,
    state: TaskState,
    priority: usize,
}

impl<T: Send + 'static> Task<T>
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
            future: Box::pin(future),
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
        match self.future.as_mut().poll(context)
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
    pub(super) fn state( &self ) -> &TaskState
    {
        &self.state
    }

    //--------------------------------------------------------------------------
    /// Gets the priority of the task.
    //--------------------------------------------------------------------------
    pub(super) fn priority( &self ) -> usize
    {
        self.priority
    }
}
