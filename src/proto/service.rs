use std::io;

use tokio_service::Service;
use futures::{future, Future, BoxFuture};

pub struct Echo;
use super::line::Line;

impl Service for Echo {
    type Request = Line;
    type Response = Line;

    type Error = io::Error;

    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        // In this case, the response is immediate.
        future::ok(req).boxed()
    }
}


