//! Calculates the winners of the "Confirmation Latency" category in Tour de SOL by tracking the
//! order of incoming validator votes. Validators earn one point for votes received before the
//! average and lose one point for votes received later than the average.

use crate::prize::{self, Winner, Winners};
use crate::utils;
use solana_core::blocktree::Blocktree;
use solana_core::blocktree_processor::{process_blocktree, ProcessOptions};
use solana_core::leader_schedule_cache::LeaderScheduleCache;
use solana_runtime::bank::Bank;
use solana_sdk::clock::{Slot, MAX_RECENT_BLOCKHASHES};
use solana_sdk::genesis_block::GenesisBlock;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_vote_api::vote_state::VoteState;
use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::process::exit;
use std::sync::Arc;
use std::time::{Duration, Instant};

// Votes received `MAX_VOTE_DELAY` slots after the current slot will not be counted towards a
// validator's latency score because this delay implies an availability issue rather than a latency
// issue.
const MAX_VOTE_DELAY: u64 = 10;

#[derive(Default, Debug)]
struct VoterEntry {
    latency_score: i64, // +1 for low latency, -1 for high latency
    last_slot: Slot,
    last_hash: Hash,
}

// Checks `bank` voter state against the latest tracked `voter_record`. If voter hash has updated,
// check if the voter has new votes to record. Returns a record of votes seen in this checkpoint
// grouped by slot.
fn voter_checkpoint(
    bank: &Bank,
    voter_record: &mut HashMap<Pubkey, VoterEntry>,
) -> HashMap<Slot, HashSet<Pubkey>> {
    let slot = bank.slot();
    let mut slot_voters: HashMap<Slot, HashSet<Pubkey>> = HashMap::new();
    for (voter_key, (_stake, account)) in bank.vote_accounts() {
        let mut voter_entry = voter_record
            .entry(voter_key)
            .or_insert_with(VoterEntry::default);
        if voter_entry.last_hash != account.hash {
            voter_entry.last_hash = account.hash;
            let vote_state = VoteState::from(&account).unwrap();
            for lockout in vote_state.votes.iter().rev() {
                if lockout.slot == voter_entry.last_slot {
                    break;
                }
                if lockout.slot < slot.saturating_sub(MAX_VOTE_DELAY) {
                    // vote was very late, don't track latency
                } else {
                    let voters = slot_voters.entry(lockout.slot).or_insert_with(HashSet::new);
                    voters.insert(voter_key);
                }
            }
            voter_entry.last_slot = vote_state.votes.back().unwrap().slot;
        }
    }
    slot_voters
}

fn score_voters(voters: Vec<HashSet<Pubkey>>, voter_record: &mut HashMap<Pubkey, VoterEntry>) {
    let total_voters: usize = voters.iter().map(|set| set.len()).sum();
    let mut voters_seen = 0;
    for voter_set in voters {
        let is_low_latency = voters_seen < total_voters / 2;
        let score_differential = if is_low_latency { 1 } else { -1 };
        for voter in &voter_set {
            if let Some(voter_entry) = voter_record.get_mut(&voter) {
                voter_entry.latency_score += score_differential;
            }
        }
        voters_seen += voter_set.len();
    }
}

fn normalize_winners(winners: &[(Pubkey, f64)]) -> Vec<Winner> {
    winners
        .iter()
        .map(|(key, latency_score)| (*key, format!("Latency score: {:.*}", 0, latency_score)))
        .collect()
}

// Track voter latency by sequentially processing the blockchain specified by `block_slots` and
// checkpointing the voter record after each entry.
fn track_voter_latency(
    block_slots: Vec<Slot>,
    root_bank: Arc<Bank>,
    blocktree: &Blocktree,
    leader_schedule_cache: &LeaderScheduleCache,
) -> HashMap<Pubkey, VoterEntry> {
    let mut voter_record: HashMap<Pubkey, VoterEntry> = HashMap::new();
    let mut slot_voter_segments: BTreeMap<u64, Vec<HashSet<Pubkey>>> = BTreeMap::new();
    let mut bank = root_bank;
    let mut last_status_report = Instant::now();
    for block_slot in block_slots {
        if last_status_report.elapsed() > Duration::from_secs(2) {
            println!("processing ledger...block {}", block_slot);
            last_status_report = Instant::now();
        }

        let leader = leader_schedule_cache
            .slot_leader_at(block_slot, Some(&bank))
            .unwrap();
        bank = Arc::new(Bank::new_from_parent(&bank, &leader, block_slot));
        let entries = blocktree.get_slot_entries(block_slot, 0, None).unwrap();

        // Process the transactions for each entry batch and then check if voter state has changed
        for entry in entries {
            if entry.is_tick() {
                bank.register_tick(&entry.hash);
                continue;
            }
            let batch = bank.prepare_batch(&entry.transactions, None);
            if let Some(err) = bank
                .load_execute_and_commit_transactions(&batch, MAX_RECENT_BLOCKHASHES)
                .into_iter()
                .find_map(|result| result.err())
            {
                eprintln!("Failed to process entry: {:?}", err);
                exit(1);
            }

            // Process the votes from the last batch of transactions
            let mut slot_voters = voter_checkpoint(&bank, &mut voter_record);
            for (slot, voters) in slot_voters.drain() {
                let slot_entry = slot_voter_segments.entry(slot).or_insert_with(Vec::new);
                slot_entry.push(voters);
            }
        }
        bank.freeze();
        if blocktree.is_root(block_slot) {
            bank.squash();
        }

        // Clear `slot_voter_segments` map when slot votes are old enough
        let old_slots: Vec<_> = slot_voter_segments
            .iter()
            .map(|(slot, _)| *slot)
            .take_while(|slot| slot < &block_slot.saturating_sub(MAX_VOTE_DELAY))
            .collect();
        for old_slot in old_slots {
            let voter_segments = slot_voter_segments.remove(&old_slot).unwrap();
            score_voters(voter_segments, &mut voter_record);
        }
    }

    for (_, voter_segments) in slot_voter_segments {
        score_voters(voter_segments, &mut voter_record);
    }
    voter_record
}

fn root_bank(genesis_block: &GenesisBlock, blocktree: &Blocktree) -> Arc<Bank> {
    let opts = ProcessOptions {
        verify_ledger: false,
        dev_halt_at_slot: Some(1),
        full_leader_cache: false,
    };

    match process_blocktree(&genesis_block, &blocktree, None, opts) {
        Ok((bank_forks, _bank_forks_info, _leader_schedule_cache)) => {
            return bank_forks.working_bank();
        }
        Err(err) => {
            eprintln!("Failed to process ledger: {:?}", err);
            exit(1);
        }
    }
}

pub fn compute_winners(
    bank: &Bank,
    blocktree: &Blocktree,
    genesis_block: &GenesisBlock,
    leader_schedule_cache: &LeaderScheduleCache,
    baseline_id: &Pubkey,
) -> Winners {
    let root_bank = root_bank(genesis_block, blocktree);
    let block_slots = utils::block_slots(root_bank.slot() + 1, bank.slot(), blocktree);

    let mut voter_record =
        track_voter_latency(block_slots, root_bank, blocktree, leader_schedule_cache);
    let mut validator_latency: HashMap<Pubkey, i64> = HashMap::new();
    for (voter_key, (_stake, account)) in bank.vote_accounts() {
        let vote_state = VoteState::from(&account).unwrap();
        let voter_entry = voter_record.remove(&voter_key).unwrap();
        // It's possible that there are multiple vote accounts attributed to a validator
        //   so use the max score when duplicates are found
        let entry = validator_latency
            .entry(vote_state.node_pubkey)
            .or_insert(std::i64::MIN);
        *entry = max(*entry, voter_entry.latency_score);
    }

    let baseline = validator_latency.remove(baseline_id).unwrap();
    let mut results: Vec<(Pubkey, f64)> = validator_latency
        .iter()
        .map(|(key, latency)| (*key, *latency as f64))
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let num_validators = results.len();
    let num_winners = min(num_validators, 3);

    Winners {
        category: prize::Category::ConfirmationLatency(format!("Baseline Score: {}", baseline)),
        top_winners: normalize_winners(&results[..num_winners]),
        bucket_winners: utils::bucket_winners(&results, baseline as f64, normalize_winners),
    }
}
