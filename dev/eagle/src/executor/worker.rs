//------------------------------------------------------------------------------
//! # Async executor worker
//------------------------------------------------------------------------------

use super::task_queue::TaskQueue;
use super::waker::waker_fn;

use std::sync::{ Arc, Condvar, Mutex };
use std::sync::atomic::{ AtomicBool, Ordering };
use std::task::{ Context, Poll };
use std::thread::{ self, JoinHandle };


//------------------------------------------------------------------------------
/// # Worker
//------------------------------------------------------------------------------
pub(super) struct Worker<T: Clone>
{
    id: usize,
    queue: TaskQueue<T>,
    is_done: Arc<(Mutex<Option<T>>, Condvar)>,
    is_stopped: Arc<AtomicBool>,
    pub(super) join_handle: Option<JoinHandle<()>>,
}

impl<T: Send + Clone + 'static> Worker<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Worker.
    //--------------------------------------------------------------------------
    pub(super) fn new
    (
        id: usize,
        queue: TaskQueue<T>,
        is_done: Arc<(Mutex<Option<T>>, Condvar)>,
    ) -> Self
    {
        Self
        {
            id,
            queue,
            is_done,
            is_stopped: Arc::new(AtomicBool::new(false)),
            join_handle: None,
        }
    }

    //--------------------------------------------------------------------------
    /// Runs the Worker.
    //--------------------------------------------------------------------------
    pub(super) fn run( &mut self )
    {
        let queue = self.queue.clone();
        let is_done = self.is_done.clone();
        let is_stopped = self.is_stopped.clone();

        let join_handle = thread::Builder::new()
            .name(self.id.to_string())
            .spawn(move ||
            {
                loop
                {
                    if is_stopped.load(Ordering::SeqCst)
                    {
                        break;
                    }

                    let mut task = match queue.pop()
                    {
                        Ok(task) =>
                        {
                            match task
                            {
                                Some(task) => task,
                                None => continue,
                            }
                        }
                        Err(_) => break,
                    };

                    let cloned_task = task.clone();
                    let waker =
                    {
                        let queue = queue.clone();
                        waker_fn(move ||
                        {
                            let _ = queue.push(cloned_task.clone());
                        })
                    };
                    let mut context = Context::from_waker(&waker);

                    match task.poll(&mut context)
                    {
                        Poll::Ready(result) =>
                        {
                            let (lock, cvar) = &*is_done;
                            let mut done = match lock.lock()
                            {
                                Ok(lock) => lock,
                                Err(_) => continue,
                            };
                            *done = Some(result);
                            cvar.notify_one();
                        },
                        Poll::Pending => {},
                    };
                }
            });
        if let Ok(join_handle) = join_handle
        {
            self.join_handle = Some(join_handle);
        }
    }
}

impl<T: Clone> Worker<T>
{
    //--------------------------------------------------------------------------
    /// Stops the Worker.
    //--------------------------------------------------------------------------
    pub(super) fn stop( &self )
    {
        self.is_stopped.store(true, Ordering::SeqCst);
    }
}

impl<T: Clone> Drop for Worker<T>
{
    fn drop( &mut self )
    {
        self.stop();
        if let Some(join_handle) = self.join_handle.take()
        {
            let _ = join_handle.join();
        }
    }
}
