//------------------------------------------------------------------------------
/// # Task queue
//------------------------------------------------------------------------------

use super::task::Task;

use std::collections::BinaryHeap;
use std::sync::{ Arc, RwLock, PoisonError };


//------------------------------------------------------------------------------
/// # TaskQueueError
//------------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) enum TaskQueueError
{
    PoisonError(String),
}

impl<E> From<PoisonError<E>> for TaskQueueError
{
    fn from( error: PoisonError<E> ) -> Self
    {
        Self::PoisonError(format!("{:?}", error))
    }
}


//------------------------------------------------------------------------------
/// # TaskQueue
//------------------------------------------------------------------------------
#[derive(Clone)]
pub(super) struct TaskQueue<T: Clone>
{
    heap: Arc<RwLock<BinaryHeap<Task<T>>>>,
}

impl<T: Clone> TaskQueue<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new TaskQueue.
    //--------------------------------------------------------------------------
    pub(super) fn new() -> Self
    {
        Self
        {
            heap: Arc::new(RwLock::new(BinaryHeap::new())),
        }
    }

    //--------------------------------------------------------------------------
    /// Pushes a task onto the queue.
    //--------------------------------------------------------------------------
    pub(super) fn push( &self, task: Task<T> ) -> Result<(), TaskQueueError>
    {
        self.heap.write()?.push(task);
        Ok(())
    }

    //--------------------------------------------------------------------------
    /// Pops the highest priority task from the queue.
    //--------------------------------------------------------------------------
    pub(super) fn pop( &self ) -> Result<Option<Task<T>>, TaskQueueError>
    {
        Ok(self.heap.write()?.pop())
    }

    //--------------------------------------------------------------------------
    /// Returns the number of tasks in the queue.
    //--------------------------------------------------------------------------
    pub(super) fn len( &self ) -> Result<usize, TaskQueueError>
    {
        let heap = self.heap.read()?;
        Ok(heap.len())
    }
}
