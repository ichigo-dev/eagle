//------------------------------------------------------------------------------
/// Multi-producer, multi-consumer channel.
//------------------------------------------------------------------------------

use std::fmt::{ self, Debug, Display, Formatter };
use std::sync::{ mpsc, Arc, Mutex, PoisonError };


//------------------------------------------------------------------------------
/// Creates a new MPMC channel.
//------------------------------------------------------------------------------
pub(super) fn channel<T>() -> (Sender<T>, Receiver<T>)
{
    let (sender, receiver) = mpsc::channel();

    let sender = Sender::new(sender);
    let receiver = Receiver::new(receiver);
    (sender, receiver)
}


//------------------------------------------------------------------------------
/// MpmcError
//------------------------------------------------------------------------------
pub(super) enum MpmcError<T>
{
    SendError(mpsc::SendError<T>),
    RecvError(mpsc::RecvError),
    TryRecvError(mpsc::TryRecvError),
    PoisonError,
}

impl<T> Debug for MpmcError<T>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> fmt::Result
    {
        match self
        {
            Self::SendError(error) => write!(f, "SendError: {:?}", error),
            Self::RecvError(error) => write!(f, "RecvError: {:?}", error),
            Self::TryRecvError(error) => write!(f, "TryRecvError: {:?}", error),
            Self::PoisonError => write!(f, "PoisonError"),
        }
    }

}

impl<T> Display for MpmcError<T>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> fmt::Result
    {
        match self
        {
            Self::SendError(error) => write!(f, "SendError: {}", error),
            Self::RecvError(error) => write!(f, "RecvError: {}", error),
            Self::TryRecvError(error) => write!(f, "TryRecvError: {}", error),
            Self::PoisonError => write!(f, "PoisonError"),
        }
    }
}

impl<T> From<mpsc::SendError<T>> for MpmcError<T>
{
    fn from( error: mpsc::SendError<T> ) -> Self
    {
        Self::SendError(error)
    }
}

impl<T> From<mpsc::RecvError> for MpmcError<T>
{
    fn from( error: mpsc::RecvError ) -> Self
    {
        Self::RecvError(error)
    }
}

impl<T> From<mpsc::TryRecvError> for MpmcError<T>
{
    fn from( error: mpsc::TryRecvError ) -> Self
    {
        Self::TryRecvError(error)
    }
}

impl<T, E> From<PoisonError<E>> for MpmcError<T>
{
    fn from( _error: PoisonError<E> ) -> Self
    {
        Self::PoisonError
    }
}


//------------------------------------------------------------------------------
/// Sender
//------------------------------------------------------------------------------
pub(super) struct Sender<T>
{
    inner: Arc<Mutex<mpsc::Sender<T>>>,
}

impl<T> Sender<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Sender.
    //--------------------------------------------------------------------------
    fn new( sender: mpsc::Sender<T> ) -> Self
    {
        Self
        {
            inner: Arc::new(Mutex::new(sender)),
        }
    }

    //--------------------------------------------------------------------------
    /// Clones the receiver.
    //--------------------------------------------------------------------------
    pub(super) fn clone( &self ) -> Self
    {
        Self
        {
            inner: self.inner.clone(),
        }
    }
    //--------------------------------------------------------------------------
    /// Sends a message.
    //--------------------------------------------------------------------------
    pub(super) fn send( &self, t: T ) -> Result<(), MpmcError<T>>
    {
        self.inner.lock()?.send(t)?;
        Ok(())
    }
}


//------------------------------------------------------------------------------
/// Receiver
//------------------------------------------------------------------------------
pub(super) struct Receiver<T>
{
    inner: Arc<Mutex<mpsc::Receiver<T>>>,
}

impl<T> Receiver<T>
{
    //--------------------------------------------------------------------------
    /// Creates a new Receiver.
    //--------------------------------------------------------------------------
    fn new( receiver: mpsc::Receiver<T> ) -> Self
    {
        Self
        {
            inner: Arc::new(Mutex::new(receiver)),
        }
    }

    //--------------------------------------------------------------------------
    /// Clones the receiver.
    //--------------------------------------------------------------------------
    pub(super) fn clone( &self ) -> Self
    {
        Self
        {
            inner: self.inner.clone(),
        }
    }

    //--------------------------------------------------------------------------
    /// Receives a message.
    //--------------------------------------------------------------------------
    pub(super) fn recv( &self ) -> Result<T, MpmcError<T>>
    {
        let message = self.inner.lock()?.recv()?;
        Ok(message)
    }
}
