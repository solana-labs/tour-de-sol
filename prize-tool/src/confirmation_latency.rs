//! Calculates the winners of the "Confirmation Latency" category in Tour de SOL by tracking the
//! order of incoming validator votes. Validators earn one point for votes received before the
//! average and lose one point for votes received later than the average.

use crate::prize::{self, Winner, Winners};
use crate::utils;
use solana_runtime::bank::Bank;
use solana_sdk::clock::Slot;
use solana_sdk::hash::Hash;
use solana_sdk::pubkey::Pubkey;
use solana_vote_api::vote_state::VoteState;
use std::cmp::{max, min};
use std::collections::{BTreeMap, HashMap, HashSet};

// Votes received `MAX_VOTE_DELAY` slots after the current slot will not be counted towards a
// validator's latency score because this delay implies an availability issue rather than a latency
// issue.
const MAX_VOTE_DELAY: u64 = 10;

#[derive(Default, Debug)]
pub struct VoterEntry {
    latency_score: i64, // +1 for low latency, -1 for high latency
    last_slot: Slot,
    last_hash: Hash,
}

// Checks `bank` voter state against the latest tracked `voter_record`. If voter hash has updated,
// check if the voter has new votes to record. Returns a record of votes seen in this checkpoint
// grouped by slot.
fn voter_checkpoint(bank: &Bank, voter_record: &mut VoterRecord) -> HashMap<Slot, HashSet<Pubkey>> {
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

// Assign latency scores to voters depending on how early their vote was recorded.
fn score_voters(voters: &[HashSet<Pubkey>], voter_record: &mut HashMap<Pubkey, VoterEntry>) {
    let total_voters: usize = voters.iter().map(|set| set.len()).sum();
    let mut voters_seen = 0;
    for voter_set in voters {
        let is_low_latency = voters_seen < total_voters / 2;
        let score_differential = if is_low_latency { 1 } else { -1 };
        for voter in voter_set {
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

/// Snapshot of the voting record of a validator
pub type VoterRecord = HashMap<Pubkey, VoterEntry>;

/// Ordered record of votes for each slot
pub type SlotVoterSegments = BTreeMap<u64, Vec<HashSet<Pubkey>>>;

/// Track voter latency by checkpointing the voter record after each entry.
pub fn on_entry(
    bank: &Bank,
    voter_record: &mut HashMap<Pubkey, VoterEntry>,
    slot_voter_segments: &mut BTreeMap<u64, Vec<HashSet<Pubkey>>>,
) {
    let mut slot_voters = voter_checkpoint(&bank, voter_record);
    for (slot, voters) in slot_voters.drain() {
        let slot_entry = slot_voter_segments.entry(slot).or_insert_with(Vec::new);
        slot_entry.push(voters);
    }
    let bank_slot = bank.slot();

    // Clear `slot_voter_segments` map when slot votes are old enough
    let old_slots: Vec<_> = slot_voter_segments
        .iter()
        .map(|(slot, _)| *slot)
        .take_while(|slot| *slot < bank_slot.saturating_sub(MAX_VOTE_DELAY))
        .collect();
    for old_slot in old_slots {
        let voter_segments = slot_voter_segments.remove(&old_slot).unwrap();
        score_voters(&voter_segments, voter_record);
    }
}

pub fn compute_winners(
    bank: &Bank,
    baseline_id: &Pubkey,
    voter_record: &mut VoterRecord,
    slot_voter_segments: &mut SlotVoterSegments,
) -> Winners {
    // Score the remaining segments leftover from entry processing
    for (_, voter_segments) in slot_voter_segments.iter() {
        score_voters(voter_segments, voter_record);
    }

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
