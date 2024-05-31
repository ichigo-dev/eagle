//------------------------------------------------------------------------------
//! Async executor worker
//------------------------------------------------------------------------------

use super::task::Task;
use super::mpmc::{ Receiver, Sender };
use super::waker::waker_fn;

use std::sync::{ Arc, Mutex };
use std::task::{ Context, Poll };
use std::thread;

pub(super) struct Worker<T>
{
    id: usize,
    sender: Sender<Arc<Mutex<Task<T>>>>,
    receiver: Receiver<Arc<Mutex<Task<T>>>>,
}

impl<T: Send + 'static> Worker<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Worker.
    //--------------------------------------------------------------------------
    pub(crate) fn new
    (
        id: usize,
        sender: Sender<Arc<Mutex<Task<T>>>>,
        receiver: Receiver<Arc<Mutex<Task<T>>>>,
    ) -> Self
    {
        Self
        {
            id,
            sender, 
            receiver,
        }
    }

    //--------------------------------------------------------------------------
    /// Runs the Worker.
    //--------------------------------------------------------------------------
    pub(crate) fn run( &self )
    {
        let sender = self.sender.clone();
        let receiver = self.receiver.clone();
        let id = self.id;
        let _ = thread::Builder::new().name(self.id.to_string()).spawn(move ||
        {
            loop
            {
                let task = match receiver.recv()
                {
                    Ok(task) => task,
                    Err(_) => break,
                };
                println!("Worker {} is running", id);

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
                    Poll::Ready(_) => {},
                    Poll::Pending => {},
                };
            }
        });
    }
}
