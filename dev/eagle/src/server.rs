//------------------------------------------------------------------------------
//! Server module
//------------------------------------------------------------------------------

use crate::executor::Executor;

use std::io::{ self, Read, Write };
use std::net::TcpListener;


//------------------------------------------------------------------------------
/// Eagle server
//------------------------------------------------------------------------------
pub struct EagleServer
{
    address: String,
}

impl EagleServer
{
    //--------------------------------------------------------------------------
    /// Creates a new server.
    //--------------------------------------------------------------------------
    pub fn new( address: String ) -> Self
    {
        Self
        {
            address,
        }
    }

    //--------------------------------------------------------------------------
    /// Starts the server.
    //--------------------------------------------------------------------------
    pub fn run( &self ) -> io::Result<()>
    {
        let listener = TcpListener::bind(&self.address)?;
        listener.set_nonblocking(true)?;

        println!("Server is running on {}", self.address);

        let executor = Executor::new(10);
        executor.start();
        executor.block_on(async move
        {
            loop
            {
                let (mut stream, _addr) = match listener.accept()
                {
                    Ok((stream, addr)) => (stream, addr),
                    Err(_) => continue,
                };

                let mut buffer = [0; 1024];
                let _ = stream.read(&mut buffer);
                let _ = stream.write(b"HTTP/1.1 200 OK\r\n\r\nHello, World\n");
                let _ = stream.flush();
            }
        });
        Ok(())
    }
}
