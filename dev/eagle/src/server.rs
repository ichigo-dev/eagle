use std::io;
use std::net::TcpListener;

pub struct EagleServer
{
    address: String,
}

impl EagleServer
{
    pub fn new( address: String ) -> Self
    {
        Self
        {
            address,
        }
    }

    pub fn run( &self ) -> io::Result<()>
    {
        let listener = TcpListener::bind(&self.address)?;
        listener.set_nonblocking(true)?;
        Ok(())
    }
}
