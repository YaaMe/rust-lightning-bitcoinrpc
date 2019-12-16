pub mod ln_mgr;
pub mod node;
pub mod udp_srv;
use futures::future::{Future, FutureExt};
use futures::task::{Context, Poll};

use crate::ln_node::settings::Settings as NodeSettings;
use ln_manager::executor::Larva;
use ln_manager::ln_bridge::settings::Settings as MgrSettings;
use tokio::runtime::Handle;

use std::pin::Pin;

pub type TaskFn = dyn Fn(Vec<Arg>, Probe) -> Result<(), String>;
pub type TaskGen = fn() -> Box<TaskFn>;
// pub type TaskSender = mpsc::UnboundedSender<Box<dyn Future<Output = Result<(), ()>> + Send + Unpin>>;

#[derive(Clone, Debug)]
pub enum Arg {
    MgrConf(MgrSettings),
    NodeConf(NodeSettings),
}

pub struct Action {
    task_gen: TaskGen,
    args: Vec<Arg>,
    exec: Probe,
}

impl Action {
    pub fn new(task_gen: TaskGen, args: Vec<Arg>, exec: Probe) -> Self {
        Action {
            task_gen: task_gen,
            args: args,
            exec: exec,
        }
    }

    pub fn summon(self) -> Result<(), futures::task::SpawnError> {
        self.exec.clone().spawn_task(self)
    }
}

impl Future for Action {
    type Output = Result<(), ()>;

    fn poll(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Self::Output> {
        let task = (self.task_gen)();
        match task(self.args.clone(), self.exec.clone()) {
            Ok(res) => Poll::Ready(Ok(res)),
            Err(_) => Poll::Pending,
        }
    }
}

#[derive(Clone)]
pub struct Probe {
    handle: Handle
}

impl Probe {
    pub fn new(handle: Handle) -> Self {
        Probe { handle }
    }
}

impl Larva for Probe {
    fn spawn_task(
        &self,
        task: impl Future<Output = Result<(), ()>> + Send + 'static,
    ) -> Result<(), futures::task::SpawnError> {
        // let _ = self.sender.unbounded_send(Box::new(Box::pin(task)));
        self.handle.spawn(task.map(|_| ()));
        Ok(())
    }
}
