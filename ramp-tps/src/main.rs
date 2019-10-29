//! Ramp up TPS for Tour de SOL until all validators drop out

mod slack;
mod stake;
mod utils;
mod voters;

use clap::{crate_description, crate_name, crate_version, value_t, value_t_or_exit, App, Arg};
use log::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{genesis_block::GenesisBlock, signature::read_keypair_file};
use solana_stake_api::config::{id as stake_config_id, Config as StakeConfig};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::{exit, Command},
    thread::sleep,
    time::Duration,
};

const NUM_BENCH_CLIENTS: usize = 2;
const TDS_ENTRYPOINT: &str = "tds.solana.com";
const TMP_LEDGER_PATH: &str = ".tmp/ledger";
const MINT_KEYPAIR_PATH: &str = "mint-keypair.json";
const PUBKEY_MAP_FILE: &str = "validators/all-username.yml";
const DEFAULT_TX_COUNT_BASELINE: &str = "5000";
const DEFAULT_TX_COUNT_INCREMENT: &str = "5000";
const DEFAULT_TPS_ROUND_MINUTES: &str = "60";
const THREAD_BATCH_SLEEP_MS: &str = "250";
const DEFAULT_INITIAL_SOL_BALANCE: &str = "1";

// Transaction count increments linearly each round
fn tx_count_for_round(tps_round: u32, base: u64, incr: u64) -> u64 {
    base + u64::from(tps_round - 1) * incr
}

// Gift will double the staked lamports each round.
fn gift_for_round(tps_round: u32, initial_balance: u64) -> u64 {
    if tps_round > 1 {
        initial_balance * 2u64.pow(tps_round - 2)
    } else {
        0
    }
}

#[allow(clippy::cognitive_complexity)]
fn main() {
    solana_logger::setup_with_filter("solana=debug");
    let slack_logger = slack::Logger::new();

    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("mint_keypair_path")
                .long("mint-keypair-path")
                .short("k")
                .value_name("PATH")
                .takes_value(true)
                .default_value(MINT_KEYPAIR_PATH)
                .help("Path to the mint keypair for stake award distribution"),
        )
        .arg(
            Arg::with_name("net_dir")
                .long("net-dir")
                .value_name("DIR")
                .takes_value(true)
                .help("The directory used for running commands on the cluster"),
        )
        .arg(
            Arg::with_name("pubkey_map_file")
                .long("pubkey-map-file")
                .value_name("FILE")
                .default_value(PUBKEY_MAP_FILE)
                .takes_value(true)
                .help("YAML file that maps validator identity pubkeys to keybase user id"),
        )
        .arg(
            Arg::with_name("round")
                .long("round")
                .value_name("NUM")
                .takes_value(true)
                .default_value("1")
                .help("The starting round of TPS ramp up"),
        )
        .arg(
            Arg::with_name("round_minutes")
                .long("round-minutes")
                .value_name("NUM")
                .takes_value(true)
                .default_value(DEFAULT_TPS_ROUND_MINUTES)
                .help("The duration in minutes of a TPS round"),
        )
        .arg(
            Arg::with_name("tx_count_baseline")
                .long("tx-count-baseline")
                .value_name("NUM")
                .takes_value(true)
                .default_value(DEFAULT_TX_COUNT_BASELINE)
                .help("The tx-count of round 1"),
        )
        .arg(
            Arg::with_name("tx_count_increment")
                .long("tx-count-increment")
                .value_name("NUM")
                .takes_value(true)
                .default_value(DEFAULT_TX_COUNT_INCREMENT)
                .help("The tx-count increment for the next round"),
        )
        .arg(
            Arg::with_name("initial_balance")
                .long("initial-balance")
                .value_name("SOL")
                .takes_value(true)
                .default_value(DEFAULT_INITIAL_SOL_BALANCE)
                .help("The number of SOL that each partipant started with"),
        )
        .arg(
            Arg::with_name("entrypoint")
                .short("n")
                .long("entrypoint")
                .value_name("HOST")
                .takes_value(true)
                .default_value(TDS_ENTRYPOINT)
                .validator(utils::is_host)
                .help("The entrypoint used for RPC calls"),
        )
        .arg(
            Arg::with_name("stake_activation_epoch")
                .long("stake-activation-epoch")
                .value_name("NUM")
                .takes_value(true)
                .help("The stake activated in this epoch must fully warm up before the first round begins"),
        )
        .arg(
            Arg::with_name("destake_net_nodes_epoch")
                .long("destake-net-nodes-epoch")
                .value_name("NUM")
                .takes_value(true)
                .default_value("8")
                .help("The epoch for which to run destake-net-nodes.sh at"),
        )
        .get_matches();

    let pubkey_map_file = value_t_or_exit!(matches, "pubkey_map_file", String);
    let pubkey_map: HashMap<String, String> =
        serde_yaml::from_reader(fs::File::open(&pubkey_map_file).unwrap_or_else(|err| {
            eprintln!(
                "Error: Unable to open --pubkey-map-file {}: {}",
                pubkey_map_file, err
            );
            exit(1);
        }))
        .unwrap_or_else(|err| {
            eprintln!(
                "Error: Unable to parse --pubkey-map-file {}: {}",
                pubkey_map_file, err
            );
            exit(1);
        });
    let pubkey_to_keybase = |pubkey: &solana_sdk::pubkey::Pubkey| -> String {
        let pubkey = pubkey.to_string();
        match pubkey_map.get(&pubkey) {
            Some(keybase) => format!("{} ({})", keybase, pubkey),
            None => pubkey,
        }
    };

    let net_dir = value_t_or_exit!(matches, "net_dir", String);

    let mint_keypair_path = value_t_or_exit!(matches, "mint_keypair_path", String);
    let mint_keypair = read_keypair_file(&mint_keypair_path)
        .unwrap_or_else(|err| panic!("Unable to read {}: {}", mint_keypair_path, err));
    let mut tps_round = value_t_or_exit!(matches, "round", u32).max(1);
    let tx_count_baseline = value_t_or_exit!(matches, "tx_count_baseline", u64);
    let tx_count_increment = value_t_or_exit!(matches, "tx_count_increment", u64);
    let round_minutes = value_t_or_exit!(matches, "round_minutes", u64).max(1);
    let round_duration = Duration::from_secs(round_minutes * 60);
    let initial_balance = value_t_or_exit!(matches, "initial_balance", u64);
    let tmp_ledger_path = PathBuf::from(TMP_LEDGER_PATH);
    let _ = fs::remove_dir_all(&tmp_ledger_path);
    fs::create_dir_all(&tmp_ledger_path).expect("failed to create temp ledger path");

    while !slack_logger.connected() {
        info!("Waiting for slack connection");
        sleep(Duration::from_secs(1));
    }
    slack_logger.info("Hi!");

    let entrypoint_str = matches.value_of("entrypoint").unwrap();
    debug!("Connecting to {}", entrypoint_str);
    let entrypoint_addr = solana_netutil::parse_host_port(&format!("{}:8899", entrypoint_str))
        .expect("failed to parse entrypoint address");
    utils::download_genesis(&entrypoint_addr, &tmp_ledger_path).expect("genesis download failed");
    let genesis_block = GenesisBlock::load(&tmp_ledger_path).expect("failed to load genesis block");

    debug!("Fetching current slot...");
    let rpc_client = RpcClient::new_socket_with_timeout(entrypoint_addr, Duration::from_secs(10));
    let current_slot = rpc_client.get_slot().expect("failed to fetch current slot");
    debug!("Current slot: {}", current_slot);
    let first_normal_slot = genesis_block.epoch_schedule.first_normal_slot;
    debug!("First normal slot: {}", first_normal_slot);
    let sleep_slots = first_normal_slot.saturating_sub(current_slot);
    if sleep_slots > 0 {
        slack_logger.info(&format!(
            "Waiting for warm-up epochs to complete (first normal epoch={})",
            genesis_block.epoch_schedule.first_normal_epoch
        ));
        utils::sleep_n_slots(sleep_slots, &genesis_block);
    }

    debug!("Fetching stake config...");
    let stake_config_account = rpc_client
        .get_account(&stake_config_id())
        .expect("failed to fetch stake config");
    let stake_config = StakeConfig::from(&stake_config_account).unwrap();

    // Check if destake-net-nodes.sh should be run
    {
        let epoch_info = rpc_client.get_epoch_info().unwrap();
        let destake_net_nodes_epoch = value_t_or_exit!(matches, "destake_net_nodes_epoch", u64);

        if epoch_info.epoch >= destake_net_nodes_epoch {
            info!(
                "Current epoch {} >= destake_net_nodes_epoch of {}, skipping destake-net-nodes.sh",
                epoch_info.epoch, destake_net_nodes_epoch
            );
        } else {
            let slots_per_epoch = genesis_block.epoch_schedule.slots_per_epoch;
            let sleep_epochs = destake_net_nodes_epoch - epoch_info.epoch;
            let sleep_slots = sleep_epochs * slots_per_epoch - epoch_info.slot_index;
            info!(
                "Waiting for destake-net-nodes epoch {}",
                destake_net_nodes_epoch
            );
            utils::sleep_n_slots(sleep_slots, &genesis_block);

            info!("Destaking net nodes...");
            Command::new("bash")
                .args(&["destake-net-nodes.sh", &net_dir])
                .spawn()
                .unwrap();
            info!("Done destaking net nodes");
        }
    }

    // Wait for the next epoch, or --stake-activation-epoch
    {
        let epoch_info = rpc_client.get_epoch_info().unwrap();
        let activation_epoch =
            if let Some(activation_epoch) = value_t!(matches, "stake_activation_epoch", u64).ok() {
                activation_epoch
            } else {
                epoch_info.epoch - 1
            };

        let epoch_info = rpc_client.get_epoch_info().unwrap();
        debug!("Current epoch info: {:?}", &epoch_info);
        debug!("Activation epoch is: {:?}", activation_epoch);
        stake::wait_for_activation(
            activation_epoch,
            epoch_info,
            &rpc_client,
            &stake_config,
            &genesis_block,
            &slack_logger,
        );
    }

    loop {
        slack_logger.info(&format!("Round {}!", tps_round));

        let slot = rpc_client.get_slot().unwrap_or_else(|err| {
            utils::bail(
                &slack_logger,
                &format!("Error: get_slot RPC call 1 failed: {}", err),
            );
        });
        sleep(Duration::from_secs(5));
        let latest_slot = rpc_client.get_slot().unwrap_or_else(|err| {
            utils::bail(
                &slack_logger,
                &format!("Error: get_slot RPC call 2 failed: {}", err),
            );
        });
        if slot == latest_slot {
            utils::bail(
                &slack_logger,
                &format!("Slot is not advancing from {}", slot),
            );
        }

        let remaining_voters = voters::fetch_remaining_voters(&rpc_client);
        slack_logger.info(&format!(
            "There are {} validators present:",
            remaining_voters.len()
        ));
        for (node_pubkey, _) in remaining_voters {
            slack_logger.info(&format!("* {}", pubkey_to_keybase(&node_pubkey)));
        }

        let tx_count = tx_count_for_round(tps_round, tx_count_baseline, tx_count_increment);
        let client_tx_count = tx_count / NUM_BENCH_CLIENTS as u64;
        slack_logger.info(&format!(
            "Starting transactions for {} minutes (batch size={})",
            round_minutes, tx_count,
        ));
        info!(
            "Running bench-tps={}='--tx_count={} --thread-batch-sleep-ms={}'",
            NUM_BENCH_CLIENTS, client_tx_count, THREAD_BATCH_SLEEP_MS
        );
        for client_id in 0..NUM_BENCH_CLIENTS {
            Command::new("bash")
                .args(&[
                    "wrapper-bench-tps.sh",
                    &net_dir,
                    &client_id.to_string(),
                    &client_tx_count.to_string(),
                    THREAD_BATCH_SLEEP_MS,
                ])
                .spawn()
                .unwrap();
        }
        sleep(round_duration);
        for client_id in 0..NUM_BENCH_CLIENTS {
            Command::new("bash")
                .args(&[
                    "wrapper-bench-tps.sh",
                    &net_dir,
                    &client_id.to_string(),
                    "0", // Setting txCount to 0 will kill bench-tps
                    THREAD_BATCH_SLEEP_MS,
                ])
                .spawn()
                .unwrap();
        }

        let remaining_voters: Vec<_> = voters::fetch_remaining_voters(&rpc_client)
            .into_iter()
            .map(|(node_pubkey, vote_account_pubkey)| {
                (pubkey_to_keybase(&node_pubkey), vote_account_pubkey)
            })
            .collect();

        if remaining_voters.is_empty() {
            utils::bail(&slack_logger, "Transactions stopped. No validators remain");
        }
        slack_logger.info(&format!(
            "Transactions stopped. There are {} validators remaining",
            remaining_voters.len()
        ));

        // Idle for 5 minutes before awarding stake to let the cluster come back together before
        // issuing RPC calls.
        // This should not be necessary once https://github.com/solana-labs/solana/pull/6538 lands
        slack_logger.info("5 minute cool down");
        sleep(Duration::from_secs(60 * 5));

        tps_round += 1;
        let next_gift = gift_for_round(tps_round, initial_balance);
        voters::award_stake(
            &rpc_client,
            &mint_keypair,
            remaining_voters,
            next_gift,
            &slack_logger,
        );

        let epoch_info = rpc_client.get_epoch_info().unwrap();
        debug!("Current epoch info: {:?}", &epoch_info);
        let current_epoch = epoch_info.epoch;
        stake::wait_for_activation(
            current_epoch,
            epoch_info,
            &rpc_client,
            &stake_config,
            &genesis_block,
            &slack_logger,
        );
    }
}
