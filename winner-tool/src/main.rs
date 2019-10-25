//! This tool calculates the quantitative category winners for Tour de SOL.
//!
//! NOTE: Ledger processing uses native programs, so this tool must be invoked with `cargo run`.
//! If installed with `cargo install` the native programs may not be linked properly.

mod availability;
mod confirmation_latency;
mod rewards_earned;
mod utils;
mod winner;

use clap::{
    crate_description, crate_name, crate_version, value_t, value_t_or_exit, values_t_or_exit, App,
    Arg,
};
use confirmation_latency::{SlotVoterSegments, VoterRecord};
use solana_cli::{
    input_parsers::pubkey_of,
    input_validators::{is_pubkey, is_pubkey_or_keypair},
};
use solana_ledger::{
    blocktree::Blocktree,
    blocktree_processor::{process_blocktree, ProcessOptions},
};
use solana_runtime::bank::Bank;
use solana_sdk::{genesis_block::GenesisBlock, native_token::sol_to_lamports, pubkey::Pubkey};
use std::{
    collections::HashSet,
    path::PathBuf,
    process::exit,
    sync::{Arc, RwLock},
};

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
                .long("starting-balance")
                .value_name("SOL")
                .takes_value(true)
                .default_value("1000")
                .help("Starting balance of validators at the beginning of TdS"),
        )
        .arg(
            Arg::with_name("baseline_validator")
                .long("baseline-validator")
                .value_name("PUBKEY")
                .takes_value(true)
                .required(true)
                .validator(is_pubkey_or_keypair)
                .help("Public key of the baseline validator"),
        )
        .arg(
            Arg::with_name("exclude_pubkeys")
                .long("exclude-pubkeys")
                .value_name("PUBKEY")
                .multiple(true)
                .takes_value(true)
                .validator(is_pubkey)
                .help("List of excluded public keys"),
        )
        .arg(
            Arg::with_name("final_slot")
                .long("final-slot")
                .value_name("SLOT")
                .takes_value(true)
                .help("Final slot of TdS ledger"),
        )
        .get_matches();

    let ledger_path = PathBuf::from(value_t_or_exit!(matches, "ledger", String));
    let starting_balance_sol = value_t_or_exit!(matches, "starting_balance", f64);
    let baseline_validator = pubkey_of(&matches, "baseline_validator").unwrap();
    let excluded_set: HashSet<Pubkey> = if matches.is_present("exclude_pubkeys") {
        let exclude_pubkeys = values_t_or_exit!(matches, "exclude_pubkeys", Pubkey);
        exclude_pubkeys.into_iter().collect()
    } else {
        HashSet::new()
    };
    let final_slot = value_t!(matches, "final_slot", u64).ok();

    let genesis_block = GenesisBlock::load(&ledger_path).unwrap_or_else(|err| {
        eprintln!(
            "Failed to open ledger genesis_block at {:?}: {}",
            ledger_path, err
        );
        exit(1);
    });

    let blocktree = Blocktree::open(&ledger_path).unwrap_or_else(|err| {
        eprintln!("Failed to open ledger at {:?}: {:?}", ledger_path, err);
        exit(1);
    });

    // Track voter record after each entry
    let voter_record: Arc<RwLock<VoterRecord>> = Arc::default();
    let slot_voter_segments: Arc<RwLock<SlotVoterSegments>> = Arc::default();
    let entry_callback = {
        let voter_record = voter_record.clone();
        let slot_voter_segments = slot_voter_segments.clone();
        Arc::new(move |bank: &Bank| {
            confirmation_latency::on_entry(
                bank.slot(),
                bank.vote_accounts(),
                &mut voter_record.write().unwrap(),
                &mut slot_voter_segments.write().unwrap(),
            );
        })
    };

    let opts = ProcessOptions {
        verify_ledger: false,
        dev_halt_at_slot: final_slot,
        full_leader_cache: true,
        entry_callback: Some(entry_callback),
        override_num_threads: Some(1),
    };

    println!("Processing ledger...");
    match process_blocktree(&genesis_block, &blocktree, None, opts) {
        Ok((bank_forks, _bank_forks_info, leader_schedule_cache)) => {
            let bank = bank_forks.working_bank();
            let starting_balance = sol_to_lamports(starting_balance_sol);

            let rewards_earned_winners =
                rewards_earned::compute_winners(&bank, &excluded_set, starting_balance);
            println!("{:#?}", rewards_earned_winners);

            let availability_winners = availability::compute_winners(
                &bank,
                &blocktree,
                &baseline_validator,
                &excluded_set,
                &leader_schedule_cache,
            );
            println!("{:#?}", availability_winners);

            let latency_winners = confirmation_latency::compute_winners(
                &bank,
                &baseline_validator,
                &excluded_set,
                &mut voter_record.write().unwrap(),
                &mut slot_voter_segments.write().unwrap(),
            );
            println!("{:#?}", latency_winners);
        }
        Err(err) => {
            eprintln!("Failed to process ledger: {:?}", err);
            exit(1);
        }
    }
}
