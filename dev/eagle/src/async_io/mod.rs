pub(crate) mod epoll;
pub(crate) mod io_uring;

use std::io;
use std::os::unix::io::RawFd;

trait AsyncIo
{
    fn new() -> Self;
    fn add( &self, fd: RawFd ) -> io::Result<()>;
    fn wait( &self ) -> io::Result<usize>;
}
