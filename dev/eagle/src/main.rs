mod async_io;
mod builder;
mod server;

use builder::EagleServerBuilder;

fn main()
{
    let server = EagleServerBuilder::new()
        .address("127.0.0.1:5500")
        .build();
    server.run();
}
