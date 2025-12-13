use core::future::Future;

// Extension trait for iterators of futures
pub trait FutureIteratorExt<F>: Iterator<Item = F> + Sized
where
    F: Future,
{
    /// Await all futures and return their outputs
    fn join_all(self) -> impl Future<Output = Vec<F::Output>>;
}

impl<I, F> FutureIteratorExt<F> for I
where
    I: Iterator<Item = F> + Sized,
    F: Future,
{
    async fn join_all(self) -> Vec<F::Output> {
        let mut out = Vec::new();

        for fut in self {
            out.push(fut.await);
        }

        out
    }
}
