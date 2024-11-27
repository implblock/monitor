use std::future::Future;

// Something that can be probed--
// Useful for getting information
// from a resource
pub trait Probe {
    fn probe(&self) -> impl Future<Output = Result<Self::Output, Self::Error>>;

    type Output;

    type Error;
}
