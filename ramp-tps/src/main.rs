//! Ramp up TPS for Tour de SOL until all validators drop out

mod stake;
mod utils;

use clap::{crate_description, crate_name, crate_version, value_t_or_exit, App, Arg};
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_sdk::genesis_block::GenesisBlock;
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

fn tps_params(tps_round: u32) -> (u64, u64) {
    let tps = u64::from(tps_round) * TPS_ROUND_INCREMENT;
    let gift = if tps_round > 1 {
        INITIAL_SOL_BALANCE * 2u64.pow(tps_round - 1)
    } else {
        0
    };

    (tps, gift)
}

fn main() {
    solana_logger::setup_with_filter("solana=info");

    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
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
        .get_matches();

    let net_dir = value_t_or_exit!(matches, "net_dir", String);
    let mut tps_round = value_t_or_exit!(matches, "round", u32).max(1);
    let round_duration =
        Duration::from_secs(value_t_or_exit!(matches, "round_minutes", u64).max(1) * 60);
    let tmp_ledger_path = PathBuf::from(TMP_LEDGER_PATH);
    let _ = fs::remove_dir_all(&tmp_ledger_path);
    fs::create_dir_all(&tmp_ledger_path).expect("failed to create temp ledger path");

    let entrypoint_str = matches.value_of("entrypoint").unwrap();
    info!("Connecting to {}", entrypoint_str);
    let entrypoint_addr = solana_netutil::parse_host_port(&format!("{}:8899", entrypoint_str))
        .expect("failed to parse entrypoint address");
    utils::download_genesis(&entrypoint_addr, &tmp_ledger_path).expect("genesis download failed");
    let genesis_block = GenesisBlock::load(&tmp_ledger_path).expect("failed to load genesis block");

    info!("Fetching current slot...");
    let rpc_client = RpcClient::new_socket_with_timeout(entrypoint_addr, Duration::from_secs(10));
    let current_slot = rpc_client.get_slot().expect("failed to fetch current slot");
    info!("Current slot: {}", current_slot);
    let first_normal_slot = genesis_block.epoch_schedule.first_normal_slot;
    info!("First normal slot: {}", first_normal_slot);
    let sleep_slots = first_normal_slot.saturating_sub(current_slot);
    if sleep_slots > 0 {
        info!("Waiting for epochs to warm up...");
        utils::sleep_n_slots(sleep_slots, &genesis_block);
    }

    info!("Fetching stake config...");
    let stake_config_account = rpc_client
        .get_account(&stake_config_id())
        .expect("failed to fetch stake config");
    let stake_config = StakeConfig::from(&stake_config_account).unwrap();

    if tps_round == 1 {
        // Now that epochs are warmed up, check if stakes are warmed up
        let first_normal_epoch = genesis_block
            .epoch_schedule
            .first_normal_epoch
            .saturating_sub(1);
        let epoch_info = rpc_client.get_epoch_info().unwrap();
        info!("Current epoch info: {:?}", &epoch_info);
        stake::wait_for_activation(
            first_normal_epoch,
            epoch_info,
            &rpc_client,
            &stake_config,
            &genesis_block,
        );
    }

    // Start bench-tps
    loop {
        let (tps, _next_gift) = tps_params(tps_round);
        info!("Starting round {} with {} TPS", tps_round, tps);
        info!(
            "Run bench-tps for {} minutes",
            round_duration.as_secs() / 60
        );
        for client_id in 0..NUM_BENCH_CLIENTS {
            Command::new("bash")
                .args(&[
                    "wrapper-bench-tps.sh",
                    &net_dir,
                    &client_id.to_string(),
                    &tps.to_string(),
                ])
                .spawn()
                .unwrap();
        }
        sleep(round_duration);

        tps_round += 1;

        let epoch_info = rpc_client.get_epoch_info().unwrap();
        info!("Current epoch info: {:?}", &epoch_info);
        let current_epoch = epoch_info.epoch;
        stake::wait_for_activation(
            current_epoch + 2,
            epoch_info,
            &rpc_client,
            &stake_config,
            &genesis_block,
        );
    }
}
