use std::io;
use std::os::unix::io::RawFd;

const MAX_EVENTS: usize = 1024;
const EPOLLIN: u32 = 0x001;

#[repr(C)]
struct epoll_event
{
    events: u32,
    data: u64,
}

extern "C"
{
    fn epoll_create1( flags: i32 ) -> RawFd;
    fn epoll_ctl
    (
        epfd: RawFd,
        op: i32,
        fd: RawFd,
        event: *mut epoll_event,
    ) -> i32;
    fn epoll_wait
    (
        epfd: RawFd,
        events: *mut epoll_event,
        maxevents: i32,
        timeout: i32,
    ) -> i32;
    fn close( fd: RawFd ) -> i32;
}

pub(crate) struct Epoll
{
    fd: RawFd,
}

impl Epoll
{
    pub(crate) fn new() -> io::Result<Self>
    {
        let fd = unsafe { epoll_create1(0) };
        if fd < 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { fd })
    }

    pub(crate) fn add( &self, fd: RawFd, events: u32 ) -> io::Result<()>
    {
        let mut ev = epoll_event
        {
            events,
            data: fd as u64,
        };
        let res = unsafe { epoll_ctl(self.fd, 1, fd, &mut ev) };
        if res < 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub(crate) fn wait( &self, events: &mut [epoll_event] ) -> io::Result<usize>
    {
        let res = unsafe
        {
            epoll_wait(self.fd, events.as_mut_ptr(), events.len() as i32, -1)
        };
        if res < 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(res as usize)
    }
}

impl Drop for Epoll
{
    fn drop( &mut self )
    {
        unsafe { close(self.fd) };
    }
}
