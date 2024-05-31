//------------------------------------------------------------------------------
//! Server module
//------------------------------------------------------------------------------

use crate::executor::Executor;

use std::io;
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

        let executor = Executor::new(10);
        executor.start();
        executor.block_on(async
        {
            handle_connection().await;
            handle_connection().await;
        });
        Ok(())
    }
}

async fn inner_fn()
{
    println!("inner");
}

async fn handle_connection()
{
    println!("hello");
    inner_fn().await;
    println!("world");
    inner_fn().await;
    println!("!!!");
    std::thread::sleep(std::time::Duration::from_secs(1));
}
