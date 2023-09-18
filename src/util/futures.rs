use futures_lite::Future;
use smol::Executor;

pub async fn join_all<F, T>(futs: Vec<F>) -> Vec<T>
where
    F: Future<Output = T>,
    T: Send + Sync + 'static,
{
    let exec = Executor::new();
    exec.run(async move {
        let mut results = Vec::new();
        for handle in futs {
            results.push(handle.await);
        }
        results
    })
    .await
}
