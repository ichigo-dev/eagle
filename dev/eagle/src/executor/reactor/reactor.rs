use crate::executor::task::Task;

use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::sync::{ Arc, Mutex };


//------------------------------------------------------------------------------
/// # Reactor
//------------------------------------------------------------------------------
pub(crate) struct Reactor<T>
{
    epoll_fd: RawFd,
    tasks: Arc<Mutex<HashMap<RawFd, Task<T>>>>,
}

impl<T> Reactor<T>
{
}
