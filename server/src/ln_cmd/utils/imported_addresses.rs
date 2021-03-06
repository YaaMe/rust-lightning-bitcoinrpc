use bitcoin::network::constants;
use bitcoin::util::address::Address;
use ln_manager::ln_bridge;

pub fn get(data_path: String, network: constants::Network) -> Vec<String> {
    let node_seed = ln_bridge::key::get_key_seed(data_path.clone());
    let (channel_monitor_claim_key, cooperative_close_key) =
        ln_bridge::key::get_import_secret_keys(network, &node_seed);
    let pub_key_1 = ln_bridge::key::get_pub_from_secret(network, channel_monitor_claim_key);
    let pub_key_2 = ln_bridge::key::get_pub_from_secret(network, cooperative_close_key);

    vec![
        String::from(format!("{}", &Address::p2pkh(&pub_key_1, network))),
        String::from(format!("{}", &Address::p2pkh(&pub_key_2, network))),
    ]
}
