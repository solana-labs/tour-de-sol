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
use solana_clap_utils::{
    input_parsers::pubkey_of,
    input_validators::{is_pubkey, is_pubkey_or_keypair},
};
use solana_ledger::{
    blocktree::Blocktree,
    blocktree_processor::{process_blocktree, ProcessOptions},
};
use solana_runtime::bank::Bank;
use solana_sdk::{genesis_config::GenesisConfig, native_token::sol_to_lamports, pubkey::Pubkey};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    process::exit,
    sync::{Arc, RwLock},
};

const PUBKEY_MAP_FILE: &str = "validators/all-username.yml";

fn main() {
    solana_logger::setup_with_filter("solana=info");

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
                .default_value("2")
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
            Arg::with_name("exclude_pubkey")
                .long("exclude-pubkey")
                .value_name("PUBKEY")
                .multiple(true)
                .takes_value(true)
                .validator(is_pubkey)
                .help("Exclude this public keys from the rewards calculation"),
        )
        .arg(
            Arg::with_name("final_slot")
                .long("final-slot")
                .value_name("SLOT")
                .takes_value(true)
                .help("Final slot of TdS ledger"),
        )
        .arg(
            Arg::with_name("pubkey_map_file")
                .long("pubkey-map-file")
                .value_name("FILE")
                .default_value(PUBKEY_MAP_FILE)
                .takes_value(true)
                .help("YAML file that maps validator identity pubkeys to keybase user id"),
        )
        .get_matches();

    let ledger_path = PathBuf::from(value_t_or_exit!(matches, "ledger", String));
    let starting_balance_sol = value_t_or_exit!(matches, "starting_balance", f64);
    let baseline_validator = pubkey_of(&matches, "baseline_validator").unwrap();
    let excluded_set: HashSet<Pubkey> = if matches.is_present("exclude_pubkey") {
        let exclude_pubkeys = values_t_or_exit!(matches, "exclude_pubkey", Pubkey);
        exclude_pubkeys.into_iter().collect()
    } else {
        HashSet::new()
    };
    let final_slot = value_t!(matches, "final_slot", u64).ok();

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

    let genesis_config = GenesisConfig::load(&ledger_path).unwrap_or_else(|err| {
        eprintln!(
            "Failed to open ledger genesis_config at {:?}: {}",
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
        poh_verify: false,
        dev_halt_at_slot: final_slot,
        full_leader_cache: true,
        entry_callback: Some(entry_callback),
        override_num_threads: Some(1),
    };

    let print_winners = |winners: winner::Winners| {
        println!("\n{:?}:", winners.category);
        if !winners.top_winners.is_empty() {
            println!("  Top Three:");
            for (index, winner) in winners.top_winners.iter().enumerate() {
                println!(
                    "    {}. {:<44}: {}",
                    index + 1,
                    pubkey_to_keybase(&winner.0),
                    winner.1
                );
            }
        }
        for (bucket_name, winners) in winners.bucket_winners.iter() {
            println!("  {}:", bucket_name);
            if winners.is_empty() {
                println!("    None");
            } else {
                for winner in winners {
                    println!("    - {:<44}: {}", pubkey_to_keybase(&winner.0), winner.1);
                }
            }
        }
    };

    println!("Processing ledger...");
    match process_blocktree(&genesis_config, &blocktree, vec![], opts) {
        Ok((bank_forks, _bank_forks_info, leader_schedule_cache)) => {
            let bank = bank_forks.working_bank();
            let starting_balance = sol_to_lamports(starting_balance_sol);

            let rewards_earned_winners =
                rewards_earned::compute_winners(&bank, &excluded_set, starting_balance);
            print_winners(rewards_earned_winners);

            let availability_winners = availability::compute_winners(
                &bank,
                &blocktree,
                &baseline_validator,
                &excluded_set,
                &leader_schedule_cache,
            );
            print_winners(availability_winners);

            let latency_winners = confirmation_latency::compute_winners(
                &bank,
                &baseline_validator,
                &excluded_set,
                &mut voter_record.write().unwrap(),
                &mut slot_voter_segments.write().unwrap(),
            );
            print_winners(latency_winners);
        }
        Err(err) => {
            eprintln!("Failed to process ledger: {:?}", err);
            exit(1);
        }
    }
}
