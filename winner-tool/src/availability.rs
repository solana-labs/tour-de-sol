//! Calculates the winners of the "Highest Availability" category in Tour de SOL by determining the
//! voting effeciency of each validator and incurring a weighted penalty for each leader slot they
//! missed.
//!
//! The top 3 validators will receive the top prizes and validators will be awarded additional
//! prizes if they perform well enough against the Solana team's validator as a baseline.

use crate::utils;
use crate::winner::{self, Winner, Winners};
use solana_ledger::blocktree::Blocktree;
use solana_ledger::leader_schedule_cache::LeaderScheduleCache;
use solana_runtime::bank::Bank;
use solana_sdk::account::Account;
use solana_sdk::clock::Slot;
use solana_sdk::pubkey::Pubkey;
use solana_vote_api::vote_state::{VoteState, MAX_LOCKOUT_HISTORY};
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

// Missed leader slots are weighted heavier than missing a vote
const MISSED_LEADER_SLOT_WEIGHT: u64 = 10;

fn normalize_winners(winners: &[(Pubkey, f64)]) -> Vec<Winner> {
    winners
        .iter()
        .map(|(key, availability)| (*key, format_availability(*availability)))
        .collect()
}

fn format_availability(availability: f64) -> String {
    format!("{:.*}% availability", 3, availability * 100f64)
}

fn validator_credits(vote_accounts: HashMap<Pubkey, (u64, Account)>) -> HashMap<Pubkey, u64> {
    let mut validator_credits = HashMap::new();
    for (_voter_key, (_stake, account)) in vote_accounts {
        if let Some(vote_state) = VoteState::from(&account) {
            validator_credits
                .entry(vote_state.node_pubkey)
                .and_modify(|credits| *credits = max(*credits, vote_state.credits()))
                .or_insert_with(|| vote_state.credits());
        }
    }
    validator_credits
}

fn validator_results(
    validator_credits: HashMap<Pubkey, u64>,
    excluded_set: &HashSet<Pubkey>,
    total_credits: u64,
    validator_leader_stats: HashMap<Pubkey, LeaderStat>,
) -> Vec<(Pubkey, f64)> {
    let mut results: Vec<(Pubkey, f64)> = validator_credits
        .iter()
        .filter(|(key, _)| !excluded_set.contains(key))
        .map(|(key, credits)| {
            let missed_slots = validator_leader_stats
                .get(key)
                .map(|stat| stat.missed_slots)
                .unwrap_or_default();
            (
                *key,
                weighted_availability(*credits, missed_slots, total_credits),
            )
        })
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results
}

/// A validator's availability is calculated from the combination of their voting effeciency and
/// a weighted penalty for missing their leader slot.
fn weighted_availability(credits: u64, missed_slots: u64, total_credits: u64) -> f64 {
    credits as f64 / (MISSED_LEADER_SLOT_WEIGHT * missed_slots + total_credits) as f64
}

#[derive(Debug)]
struct LeaderStat {
    missed_slots: u64,
    total_slots: u64,
}

impl LeaderStat {
    fn new(missed: bool) -> Self {
        LeaderStat {
            missed_slots: if missed { 1 } else { 0 },
            total_slots: 1,
        }
    }
}

fn validator_leader_stats(
    bank: &Bank,
    block_chain: Vec<Slot>,
    leader_schedule_cache: &LeaderScheduleCache,
) -> HashMap<Pubkey, LeaderStat> {
    let mut validator_leader_stats: HashMap<Pubkey, LeaderStat> = HashMap::new();
    let mut inc_leader_stat = |slot: u64, missed: bool| {
        let leader = leader_schedule_cache
            .slot_leader_at(slot, Some(bank))
            .unwrap();

        validator_leader_stats
            .entry(leader)
            .and_modify(|leader_stat| {
                leader_stat.total_slots += 1;
                if missed {
                    leader_stat.missed_slots += 1;
                }
            })
            .or_insert_with(|| LeaderStat::new(missed));
    };

    let mut last_slot = bank.slot();
    for parent_slot in block_chain.into_iter().rev() {
        if parent_slot > 0 {
            inc_leader_stat(parent_slot, false);
        }
        for missed_slot in (parent_slot + 1..last_slot).rev() {
            inc_leader_stat(missed_slot, true);
        }
        last_slot = parent_slot;
    }
    validator_leader_stats
}

pub fn compute_winners(
    bank: &Bank,
    blocktree: &Blocktree,
    baseline_id: &Pubkey,
    excluded_set: &HashSet<Pubkey>,
    leader_schedule_cache: &LeaderScheduleCache,
) -> Winners {
    let block_chain = utils::block_chain(0, bank.slot(), blocktree);
    let mut validator_credits = validator_credits(bank.vote_accounts());
    let baseline_credits = validator_credits.remove(baseline_id).unwrap_or_else(|| {
        panic!(
            "Solana baseline validator {} not found in validator_credits",
            baseline_id
        )
    });

    let mut validator_leader_stats =
        validator_leader_stats(bank, block_chain, &leader_schedule_cache);
    let baseline_leader_stat = validator_leader_stats
        .remove(baseline_id)
        .unwrap_or_else(|| {
            panic!(
                "Solana baseline validator {} not found in validator_leader_stats",
                baseline_id
            )
        });

    let total_blocks = bank.block_height();
    let total_credits = total_blocks.saturating_sub(MAX_LOCKOUT_HISTORY as u64);
    let results = validator_results(
        validator_credits,
        excluded_set,
        total_credits,
        validator_leader_stats,
    );

    let num_validators = results.len();
    let num_winners = min(num_validators, 3);
    let baseline = weighted_availability(
        baseline_credits,
        baseline_leader_stat.missed_slots,
        total_credits,
    );

    Winners {
        category: winner::Category::Availability(format!(
            "Baseline: {}",
            format_availability(baseline)
        )),
        top_winners: normalize_winners(&results[..num_winners]),
        bucket_winners: utils::bucket_winners(&results, baseline, normalize_winners),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_vote_api::vote_state::VoteInit;

    #[test]
    fn test_validator_results() {
        let mut credits_map = HashMap::new();
        let top_validator = Pubkey::new_rand();
        let bottom_validator = Pubkey::new_rand();
        let excluded_validator = Pubkey::new_rand();
        credits_map.insert(top_validator, 1000);
        credits_map.insert(bottom_validator, 100);
        credits_map.insert(excluded_validator, 10);
        let total_credits = 1000;

        let excluded_set = {
            let mut set = HashSet::new();
            set.insert(excluded_validator);
            set
        };

        let mut validator_leader_stats = HashMap::new();
        validator_leader_stats.insert(
            top_validator,
            LeaderStat {
                missed_slots: 0,
                total_slots: 1000,
            },
        );
        validator_leader_stats.insert(
            bottom_validator,
            LeaderStat {
                missed_slots: 100,
                total_slots: 1000,
            },
        );

        let results = validator_results(
            credits_map,
            &excluded_set,
            total_credits,
            validator_leader_stats,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (top_validator, 1.0));
        assert_eq!(results[1], (bottom_validator, 0.05));
    }

    #[test]
    fn test_validator_credits() {
        let new_vote_account = |credits: u64, validator_id: &Pubkey| -> Account {
            let mut state = VoteState::new(&VoteInit {
                node_pubkey: validator_id.clone(),
                ..VoteInit::default()
            });
            (0..credits).for_each(|_| {
                state.increment_credits(0);
            });
            Account::new_data(1, &state, &Pubkey::new_rand()).unwrap()
        };

        let validator1 = Pubkey::new_rand();
        let validator2 = Pubkey::new_rand();

        let mut vote_accounts = HashMap::new();
        let voter1 = Pubkey::new_rand();
        vote_accounts.insert(voter1.clone(), (0, new_vote_account(25, &validator1)));
        vote_accounts.insert(Pubkey::new_rand(), (0, new_vote_account(10, &validator2)));
        vote_accounts.insert(Pubkey::new_rand(), (0, new_vote_account(15, &validator2)));

        let expected_credits = {
            let mut map = HashMap::new();
            map.insert(validator1, 25);
            map.insert(validator2, 15);
            map
        };

        assert_eq!(expected_credits, validator_credits(vote_accounts));
    }
}
