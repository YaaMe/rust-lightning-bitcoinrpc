use futures::future::Future;
use futures::Stream;

pub trait Larva: Clone + Sized + Send + Sync {
    fn spawn_task(
        &self,
        task: impl Future<Item = (), Error = ()> + Send,
    ) -> Result<(), futures::future::ExecuteError<Box<dyn Future<Item = (), Error = ()> + Send>>>;
}
