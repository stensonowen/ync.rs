use std::io;

use tokio_service::Service;
use futures::{future, Future, BoxFuture};

pub struct Echo;
use super::Request;

impl Service for Echo {
    // These types must match the corresponding protocol types:
    type Request = Request;
    type Response = Request;

    // For non-streaming protocols, service errors are always io::Error
    type Error = io::Error;

    // The future for computing the response; box it for simplicity.
    type Future = BoxFuture<Self::Response, Self::Error>;

    // Produce a future for computing a response from a request.
    fn call(&self, req: Self::Request) -> Self::Future {
        // In this case, the response is immediate.
        future::ok(req).boxed()
    }
}


