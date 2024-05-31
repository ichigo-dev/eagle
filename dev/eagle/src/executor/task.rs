//------------------------------------------------------------------------------
//! Async task
//------------------------------------------------------------------------------

use std::future::Future;
use std::pin::Pin;
use std::task::{ Context, Poll };

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub(super) struct Task<T>
{
    future: BoxFuture<T>,
}

impl<T: Send + 'static> Task<T>
{
    pub(super) fn new( future: BoxFuture<T> ) -> Self
    {
        Self { future }
    }

    pub(super) fn poll( &mut self, context: &mut Context ) -> Poll<T>
    {
        self.future.as_mut().poll(context)
    }
}
