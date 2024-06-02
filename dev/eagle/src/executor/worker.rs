//------------------------------------------------------------------------------
//! # Async executor worker
//------------------------------------------------------------------------------

use super::task::Task;
use super::mpmc::{ Receiver, Sender };
use super::waker::waker_fn;

use std::sync::{ Arc, Condvar, Mutex };
use std::sync::atomic::{ AtomicBool, Ordering };
use std::task::{ Context, Poll };
use std::thread::{ self, JoinHandle };


//------------------------------------------------------------------------------
/// # Worker
//------------------------------------------------------------------------------
pub(super) struct Worker<T>
{
    id: usize,
    sender: Sender<Arc<Mutex<Task<T>>>>,
    receiver: Receiver<Arc<Mutex<Task<T>>>>,
    is_done: Arc<(Mutex<Option<T>>, Condvar)>,
    is_stopped: Arc<AtomicBool>,
    pub(super) join_handle: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> Worker<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Worker.
    //--------------------------------------------------------------------------
    pub(super) fn new
    (
        id: usize,
        sender: Sender<Arc<Mutex<Task<T>>>>,
        receiver: Receiver<Arc<Mutex<Task<T>>>>,
        is_done: Arc<(Mutex<Option<T>>, Condvar)>,
    ) -> Self
    {
        Self
        {
            id,
            sender, 
            receiver,
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
        let sender = self.sender.clone();
        let receiver = self.receiver.clone();
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

                    let task = match receiver.recv()
                    {
                        Ok(task) => task,
                        Err(_) => break,
                    };

                    let cloned_task = task.clone();
                    let waker =
                    {
                        let sender = sender.clone();
                        waker_fn(move ||
                        {
                            let _ = sender.send(cloned_task.clone());
                        })
                    };
                    let mut context = Context::from_waker(&waker);

                    let mut guard = match task.lock()
                    {
                        Ok(guard) => guard,
                        Err(_) => continue,
                    };
                    match guard.poll(&mut context)
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

impl<T> Worker<T>
{
    //--------------------------------------------------------------------------
    /// Stops the Worker.
    //--------------------------------------------------------------------------
    pub(super) fn stop( &self )
    {
        self.is_stopped.store(true, Ordering::SeqCst);
    }
}

impl<T> Drop for Worker<T>
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
