pub mod settings;
use crate::ln_cmd::tasks::node;
use crate::ln_cmd::tasks::{Action, Arg, Probe};
use crate::ln_node::settings::Settings as NodeSettings;
use ln_manager::ln_bridge::settings::Settings as MgrSettings;

pub fn run(ln_conf: MgrSettings, node_conf: NodeSettings) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    // let (tx, mut rx) = futures::channel::mpsc::unbounded();
    let handle = rt.handle();
    let runner = Probe::new(handle.clone());
    let init_node: Action = Action::new(
        node::gen,
        vec![Arg::MgrConf(ln_conf), Arg::NodeConf(node_conf)],
        runner,
    );
    let _ = init_node.summon();

    let _ = rt.block_on(node::run_forever());
}
