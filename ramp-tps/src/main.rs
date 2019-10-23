//! Ramp up TPS for Tour de SOL until all validators drop out

mod stake;
mod utils;
mod voters;

use clap::{crate_description, crate_name, crate_version, value_t, value_t_or_exit, App, Arg};
use log::{debug, info};
use solana_client::rpc_client::RpcClient;
use solana_sdk::genesis_block::GenesisBlock;
use solana_sdk::signature::read_keypair_file;
use solana_stake_api::config::{id as stake_config_id, Config as StakeConfig};
use std::process::Command;
use std::thread::sleep;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const NUM_BENCH_CLIENTS: usize = 2;
const TDS_ENTRYPOINT: &str = "tds.solana.com";
const TMP_LEDGER_PATH: &str = ".tmp/ledger";
const TPS_ROUND_INCREMENT: u64 = 5000;
const INITIAL_SOL_BALANCE: u64 = 1;
const MINT_KEYPAIR_PATH: &str = "mint-keypair.json";

// TPS increments linearly each round
fn tps_for_round(tps_round: u32) -> u64 {
    u64::from(tps_round) * TPS_ROUND_INCREMENT
}

// Gift will double the staked lamports each round and assume that validators start with 1 SOL.
//   - Award 1 SOL for Round 2
//   - Award 2 SOL for Round 3
//   - Award 4 SOL for Round 4
fn gift_for_round(tps_round: u32) -> u64 {
    if tps_round > 1 {
        INITIAL_SOL_BALANCE * 2u64.pow(tps_round - 2)
    } else {
        0
    }
}

fn main() {
    solana_logger::setup_with_filter("solana=debug");

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
                .help("This tool uses the net path to run commands on the cluster"),
        )
        .arg(
            Arg::with_name("round")
                .long("round")
                .value_name("NUM")
                .takes_value(true)
                .default_value("1")
                .help("The round of TPS ramp up (round 1: 5000, round 2: 10000, etc.)"),
        )
        .arg(
            Arg::with_name("round_minutes")
                .long("round-minutes")
                .value_name("NUM")
                .takes_value(true)
                .default_value("60")
                .help("The duration in minutes of a TPS round"),
        )
        .arg(
            Arg::with_name("entrypoint")
                .short("n")
                .long("entrypoint")
                .value_name("HOST")
                .takes_value(true)
                .default_value(TDS_ENTRYPOINT)
                .validator(utils::is_host)
                .help("Download the genesis block from this entry point"),
        )
        .arg(
            Arg::with_name("stake_activation_epoch")
                .long("stake-activation-epoch")
                .value_name("NUM")
                .takes_value(true)
                .help("The stake activation epoch that must finish warming up before beginning"),
        )
        .get_matches();

    let net_dir = value_t_or_exit!(matches, "net_dir", String);
    let mint_keypair_path = value_t_or_exit!(matches, "mint_keypair_path", String);
    let mint_keypair = read_keypair_file(&mint_keypair_path).unwrap();
    let mut tps_round = value_t_or_exit!(matches, "round", u32).max(1);
    let round_duration =
        Duration::from_secs(value_t_or_exit!(matches, "round_minutes", u64).max(1) * 60);
    let tmp_ledger_path = PathBuf::from(TMP_LEDGER_PATH);
    let _ = fs::remove_dir_all(&tmp_ledger_path);
    fs::create_dir_all(&tmp_ledger_path).expect("failed to create temp ledger path");

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
        info!("Waiting for epochs to warm up...");
        utils::sleep_n_slots(sleep_slots, &genesis_block);
    }

    debug!("Fetching stake config...");
    let stake_config_account = rpc_client
        .get_account(&stake_config_id())
        .expect("failed to fetch stake config");
    let stake_config = StakeConfig::from(&stake_config_account).unwrap();

    // Now that epochs are warmed up, check if stakes are warmed up
    let activation_epoch = if tps_round == 1 {
        // Check an early epoch to make sure initial stake has finished warming up
        Some(
            genesis_block
                .epoch_schedule
                .first_normal_epoch
                .saturating_sub(1),
        )
    } else {
        value_t!(matches, "stake_activation_epoch", u64).ok()
    };

    if let Some(activation_epoch) = activation_epoch {
        let epoch_info = rpc_client.get_epoch_info().unwrap();
        debug!("Current epoch info: {:?}", &epoch_info);
        stake::wait_for_activation(
            activation_epoch,
            epoch_info,
            &rpc_client,
            &stake_config,
            &genesis_block,
        );
    }

    // Start bench-tps
    loop {
        let tps = tps_for_round(tps_round);
        info!("Starting round {} with {} TPS", tps_round, tps);
        info!(
            "Run bench-tps for {} minutes",
            round_duration.as_secs() / 60
        );
        let client_tps = tps / NUM_BENCH_CLIENTS as u64;
        for client_id in 0..NUM_BENCH_CLIENTS {
            Command::new("bash")
                .args(&[
                    "wrapper-bench-tps.sh",
                    &net_dir,
                    &client_id.to_string(),
                    &client_tps.to_string(),
                ])
                .spawn()
                .unwrap();
        }
        sleep(round_duration);

        let remaining_voters = voters::fetch_remaining_voters(&rpc_client);
        info!(
            "End of round {}. There are {} validators remaining",
            tps_round,
            remaining_voters.len()
        );

        if remaining_voters.is_empty() {
            info!("No validators left standing",);
            break;
        }

        tps_round += 1;
        let next_gift = gift_for_round(tps_round);
        info!(
            "Delegate {} SOL in stake to each remaining validator",
            next_gift
        );
        voters::award_stake(&rpc_client, &mint_keypair, remaining_voters, next_gift);

        let epoch_info = rpc_client.get_epoch_info().unwrap();
        debug!("Current epoch info: {:?}", &epoch_info);
        let current_epoch = epoch_info.epoch;
        stake::wait_for_activation(
            current_epoch + 1,
            epoch_info,
            &rpc_client,
            &stake_config,
            &genesis_block,
        );
    }
}
