//------------------------------------------------------------------------------
/// # Priority MPMC Channel
///
/// Multi-producer, multi-consumer channel.
//------------------------------------------------------------------------------

use std::collections::BinaryHeap;
use std::fmt::{ self, Debug, Display, Formatter };
use std::sync::{ Arc, Condvar, Mutex, PoisonError };


//------------------------------------------------------------------------------
/// # channel
///
/// Creates a new MPMC channel.
//------------------------------------------------------------------------------
pub(super) fn channel<T: Send + Ord>() -> (Sender<T>, Receiver<T>)
{
    let queue = PriorityChannel::new();
    let sender = Sender::new(queue.clone());
    let receiver = Receiver::new(queue);
    (sender, receiver)
}


//------------------------------------------------------------------------------
/// # MpmcError
//------------------------------------------------------------------------------
pub(crate) enum MpmcError
{
    PoisonError(String),
    Empty,
}

impl Debug for MpmcError
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> fmt::Result
    {
        match self
        {
            Self::PoisonError(error) => write!(f, "PoisonError: {:?}", error),
            Self::Empty => write!(f, "Queue is empty"),
        }
    }
}

impl Display for MpmcError
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> fmt::Result
    {
        match self
        {
            Self::PoisonError(error) => write!(f, "PoisonError: {}", error),
            Self::Empty => write!(f, "Queue is empty"),
        }
    }
}

impl<E> From<PoisonError<E>> for MpmcError
{
    fn from( error: PoisonError<E> ) -> Self
    {
        Self::PoisonError(format!("{:?}", error))
    }
}


//------------------------------------------------------------------------------
/// # Channle
//------------------------------------------------------------------------------
pub(super) struct PriorityChannel<T: Send + Ord>
{
    inner: Arc<Mutex<BinaryHeap<T>>>,
    cvar: Arc<Condvar>,
}

impl<T: Send + Ord> PriorityChannel<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new PriorityChannel.
    //--------------------------------------------------------------------------
    fn new() -> Self
    {
        Self
        {
            inner: Arc::new(Mutex::new(BinaryHeap::new())),
            cvar: Arc::new(Condvar::new()),
        }
    }

    //--------------------------------------------------------------------------
    /// Clones the channel.
    //--------------------------------------------------------------------------
    pub(super) fn clone( &self ) -> Self
    {
        Self
        {
            inner: self.inner.clone(),
            cvar: self.cvar.clone(),
        }
    }

    //--------------------------------------------------------------------------
    /// Pushes an item.
    //--------------------------------------------------------------------------
    pub(super) fn push( &self, t: T ) -> Result<(), MpmcError>
    {
        let mut inner = self.inner.lock()?;
        inner.push(t);
        self.cvar.notify_one();
        Ok(())
    }

    //--------------------------------------------------------------------------
    /// Pops an item.
    //--------------------------------------------------------------------------
    pub(super) fn pop( &self ) -> Result<T, MpmcError>
    {
        let mut inner = self.inner.lock()?;
        while inner.is_empty()
        {
            inner = self.cvar.wait(inner)?;
        }
        let t = match inner.pop()
        {
            Some(t) => t,
            None => { return Err(MpmcError::Empty); },
        };
        Ok(t)
    }

    //--------------------------------------------------------------------------
    /// Tries to pop an item.
    //--------------------------------------------------------------------------
    pub(super) fn try_pop( &self ) -> Result<T, MpmcError>
    {
        let mut inner = self.inner.lock()?;
        let t = match inner.pop()
        {
            Some(t) => t,
            None => { return Err(MpmcError::Empty); },
        };
        Ok(t)
    }
}


//------------------------------------------------------------------------------
/// # Sender
//------------------------------------------------------------------------------
pub(super) struct Sender<T: Send + Ord>
{
    queue: Arc<PriorityChannel<T>>,
}

impl<T: Send + Ord> Sender<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Sender.
    //--------------------------------------------------------------------------
    fn new( queue: PriorityChannel<T> ) -> Self
    {
        Self { queue: Arc::new(queue) }
    }

    //--------------------------------------------------------------------------
    /// Clones the receiver.
    //--------------------------------------------------------------------------
    pub(super) fn clone( &self ) -> Self
    {
        Self { queue: self.queue.clone() }
    }

    //--------------------------------------------------------------------------
    /// Sends an item.
    //--------------------------------------------------------------------------
    pub(super) fn send( &self, t: T ) -> Result<(), MpmcError>
    {
        self.queue.push(t)
    }
}


//------------------------------------------------------------------------------
/// # Receiver
//------------------------------------------------------------------------------
pub(super) struct Receiver<T: Send + Ord>
{
    queue: Arc<PriorityChannel<T>>,
}

impl<T: Send + Ord> Receiver<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Receiver.
    //--------------------------------------------------------------------------
    fn new( queue: PriorityChannel<T> ) -> Self
    {
        Self { queue: Arc::new(queue) }
    }

    //--------------------------------------------------------------------------
    /// Clones the receiver.
    //--------------------------------------------------------------------------
    pub(super) fn clone( &self ) -> Self
    {
        Self { queue: self.queue.clone() }
    }

    //--------------------------------------------------------------------------
    /// Receives an item.
    //--------------------------------------------------------------------------
    pub(super) fn recv( &self ) -> Result<T, MpmcError>
    {
        self.queue.pop()
    }

    //--------------------------------------------------------------------------
    /// Tries to receive an item.
    //--------------------------------------------------------------------------
    pub(super) fn try_recv( &self ) -> Result<T, MpmcError>
    {
        self.queue.try_pop()
    }
}
