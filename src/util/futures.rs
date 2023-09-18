use futures_lite::Future;
use smol::Executor;

pub async fn join_all<F, T>(futs: Vec<F>) -> Vec<T>
where
    F: Future<Output = T> + Send,
    T: Send,
{
    let exec = Executor::new();
    let mut handles = Vec::new();
    for fut in futs {
        handles.push(exec.spawn(fut));
    }
    exec.run(async move {
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await);
        }
        results
    })
    .await
}
