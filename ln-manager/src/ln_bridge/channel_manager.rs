use std::sync::Arc;
use std::fs;
use std::collections::HashMap;

use bitcoin::network::constants::Network;
use bitcoin_hashes::sha256d::Hash;

use lightning::chain::keysinterface::{KeysInterface, InMemoryChannelKeys};
use lightning::chain::chaininterface::{FeeEstimator, BroadcasterInterface, ChainListener, BlockNotifier};
use lightning::chain::transaction::OutPoint;
use lightning::ln::channelmanager::{ChannelManager, ChannelManagerReadArgs};
use lightning::ln::channelmonitor::{ChannelMonitor, ManyChannelMonitor};
use lightning::util::ser::ReadableArgs;
use lightning::util::config::UserConfig;
use lightning::util::logger::{Logger};

use super::Restorable;

const FEE_PROPORTIONAL_MILLIONTHS: u32 = 10;
const ANNOUNCE_CHANNELS: bool = true;

pub struct RestoreArgs {
    data_path: String,
    monitors_loaded: Vec<(OutPoint, ChannelMonitor)>,
    network: Network,
    fee_estimator: Arc<dyn FeeEstimator>,
    monitor: Arc<dyn ManyChannelMonitor>,
    block_notifier: Arc<BlockNotifier>,
    tx_broadcaster: Arc<dyn BroadcasterInterface>,
    logger: Arc<dyn Logger>,
    keys_manager: Arc<dyn KeysInterface<ChanKeySigner = InMemoryChannelKeys>>,
    current_block_height: usize,
}

impl RestoreArgs {
    pub fn new(
        data_path: String,
        monitors_loaded: Vec<(OutPoint, ChannelMonitor)>,
        network: Network,
        fee_estimator: Arc<dyn FeeEstimator>,
        monitor: Arc<dyn ManyChannelMonitor>,
        block_notifier: Arc<BlockNotifier>,
        tx_broadcaster: Arc<dyn BroadcasterInterface>,
        logger: Arc<dyn Logger>,
        keys_manager: Arc<dyn KeysInterface<ChanKeySigner = InMemoryChannelKeys>>,
        current_block_height: usize,
    ) -> Self {
        RestoreArgs {
            data_path, monitors_loaded, network, fee_estimator,
            monitor, block_notifier, tx_broadcaster,
            logger, keys_manager, current_block_height,
        }
    }
}

impl Restorable<RestoreArgs, Arc<ChannelManager<InMemoryChannelKeys>>> for ChannelManager<InMemoryChannelKeys> {
    fn try_restore(mut args: RestoreArgs) -> Arc<ChannelManager<InMemoryChannelKeys>> {
        let mut config = UserConfig::default();
        config.channel_options.fee_proportional_millionths = FEE_PROPORTIONAL_MILLIONTHS;
        config.channel_options.announced_channel = ANNOUNCE_CHANNELS;

        if let Ok(mut f) = fs::File::open(args.data_path + "/manager_data") {
            let (_last_block_hash, manager) = {
                let mut monitors_refs = HashMap::<OutPoint, &mut ChannelMonitor>::new();
                for (outpoint, monitor) in args.monitors_loaded.iter_mut() {
                    monitors_refs.insert(*outpoint, monitor);
                }
                <(Hash, ChannelManager<InMemoryChannelKeys>)>::read(&mut f, ChannelManagerReadArgs {
                    keys_manager: args.keys_manager,
                    fee_estimator: args.fee_estimator,
                    monitor: args.monitor.clone(),
                    tx_broadcaster: args.tx_broadcaster,
                    logger: args.logger,
                    default_config: config,
                    channel_monitors: &mut monitors_refs,
                }).expect("Failed to deserialize channel manager")
            };

            let mut mut_monitors_loaded = args.monitors_loaded;
            for (outpoint, drain_monitor) in mut_monitors_loaded.drain(..) {
                if let Err(_) = args.monitor.add_update_monitor(outpoint, drain_monitor) {
                    panic!("Failed to load monitor that deserialized");
                }
            }
            //TODO: Rescan
            let manager = Arc::new(manager);
            let manager_as_listener: Arc<dyn ChainListener> = manager.clone();
            args.block_notifier.register_listener(Arc::downgrade(&manager_as_listener));
            manager
        } else {
            if !args.monitors_loaded.is_empty() {
                panic!("Found some channel monitors but no channel state!");
            }
            ChannelManager::new(
                args.network,
                args.fee_estimator,
                args.monitor,
                args.tx_broadcaster,
                args.logger,
                args.keys_manager,
                config,
                args.current_block_height,
            ).unwrap()
        }
    }
}
