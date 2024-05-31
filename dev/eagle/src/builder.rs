use crate::server::EagleServer;

pub struct EagleServerBuilder
{
    address: String,
}

impl EagleServerBuilder
{
    pub fn new() -> Self
    {
        Self
        {
            address: String::new(),
        }
    }

    pub fn address(&mut self, address: &str) -> &mut Self
    {
        self.address = address.to_string();
        self
    }

    pub fn build(&self) -> EagleServer
    {
        EagleServer::new(self.address.clone())
    }
}
