mod availability;
mod prize;
mod rewards_earned;

use clap::{crate_description, crate_name, crate_version, value_t, value_t_or_exit, App, Arg};
use solana_core::blocktree::Blocktree;
use solana_core::blocktree_processor::process_blocktree;
use solana_sdk::genesis_block::GenesisBlock;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::pubkey::Pubkey;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

fn main() {
    solana_logger::setup();

    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("ledger")
                .short("l")
                .long("ledger")
                .value_name("DIR")
                .takes_value(true)
                .required(true)
                .help("Use directory for ledger location"),
        )
        .arg(
            Arg::with_name("starting_balance")
                .long("starting_balance")
                .value_name("SOL")
                .takes_value(true)
                .default_value("1000")
                .help("Starting balance of validators at the beginning of TdS"),
        )
        .arg(
            Arg::with_name("solana_baseline")
                .long("solana_baseline")
                .value_name("PUBKEY")
                .takes_value(true)
                .required(true)
                .help("Public key of the Solana baseline validator"),
        )
        .arg(
            Arg::with_name("final_slot")
                .long("final_slot")
                .value_name("SLOT")
                .takes_value(true)
                .help("Final slot of TdS ledger"),
        )
        .get_matches();

    let ledger_path = PathBuf::from(value_t_or_exit!(matches, "ledger", String));
    let starting_balance_sol = value_t_or_exit!(matches, "starting_balance", f64);
    let baseline_id_string = value_t_or_exit!(matches, "solana_baseline", String);
    let final_slot = value_t!(matches, "final_slot", u64).ok();

    let baseline_id = match Pubkey::from_str(&baseline_id_string) {
        Ok(baseline_id) => baseline_id,
        Err(err) => {
            eprintln!(
                "Failed to create a valid pubkey from {}: {}",
                baseline_id_string, err
            );
            exit(1);
        }
    };

    let genesis_block = GenesisBlock::load(&ledger_path).unwrap_or_else(|err| {
        eprintln!(
            "Failed to open ledger genesis_block at {:?}: {}",
            ledger_path, err
        );
        exit(1);
    });

    let blocktree = match Blocktree::open(&ledger_path) {
        Ok(blocktree) => blocktree,
        Err(err) => {
            eprintln!("Failed to open ledger at {:?}: {}", ledger_path, err);
            exit(1);
        }
    };

    println!("Processing ledger...");
    match process_blocktree(&genesis_block, &blocktree, None, false, final_slot) {
        Ok((bank_forks, _bank_forks_info, _leader_schedule_cache)) => {
            let bank = bank_forks.working_bank();
            let starting_balance = sol_to_lamports(starting_balance_sol);
            let rewards_earned_winners = rewards_earned::compute_winners(&bank, starting_balance);
            let availability_winners = availability::compute_winners(&bank, &baseline_id);
            println!("{:#?}\n{:#?}", rewards_earned_winners, availability_winners);
        }
        Err(err) => {
            eprintln!("Failed to process ledger: {:?}", err);
            exit(1);
        }
    }
}
