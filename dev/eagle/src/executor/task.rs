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


//------------------------------------------------------------------------------
/// # TaskHandle
//------------------------------------------------------------------------------
pub(crate) struct TaskHandle<T>
{
    task: Arc<Mutex<Task<T>>>,
}

impl<T: Send + 'static> TaskHandle<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new TaskHandle.
    //--------------------------------------------------------------------------
    pub(super) fn new( task: Task<T> ) -> Self
    {
        Self
        {
            task: Arc::new(Mutex::new(task)),
        }
    }

    //--------------------------------------------------------------------------
    /// Polls the task.
    //--------------------------------------------------------------------------
    pub(super) fn poll( &self, context: &mut Context ) -> Poll<T>
    {
        let mut task = self.task.lock().unwrap();
        task.poll(context)
    }

    //--------------------------------------------------------------------------
    /// Gets the state of the task.
    //--------------------------------------------------------------------------
    pub(super) fn state( &self ) -> TaskState
    {
        let task = self.task.lock().unwrap();
        task.state.clone()
    }

    //--------------------------------------------------------------------------
    /// Gets the priority of the task.
    //--------------------------------------------------------------------------
    pub(super) fn priority( &self ) -> usize
    {
        let task = self.task.lock().unwrap();
        task.priority
    }
}

impl<T> Clone for TaskHandle<T>
{
    fn clone( &self ) -> Self
    {
        Self
        {
            task: self.task.clone(),
        }
    }
}

impl<T> PartialEq for TaskHandle<T>
{
    fn eq( &self, other: &Self ) -> bool
    {
        let task = self.task.lock().unwrap();
        let other = other.task.lock().unwrap();
        task.priority == other.priority
    }
}

impl<T> Eq for TaskHandle<T> {}

impl<T> PartialOrd for TaskHandle<T>
{
    fn partial_cmp( &self, other: &Self ) -> Option<std::cmp::Ordering>
    {
        Some(self.cmp(other))
    }
}

impl<T> Ord for TaskHandle<T>
{
    fn cmp( &self, other: &Self ) -> std::cmp::Ordering
    {
        let task = self.task.lock().unwrap();
        let other = other.task.lock().unwrap();
        task.priority.cmp(&other.priority)
    }
}
