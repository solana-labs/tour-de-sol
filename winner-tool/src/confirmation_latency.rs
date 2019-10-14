//! Calculates the winners of the "Confirmation Latency" category in Tour de SOL by tracking the
//! order of incoming validator votes. Validators earn one point for votes received before the
//! average and lose one point for votes received later than the average.

use crate::utils;
use crate::winner::{self, Winner, Winners};
use solana_runtime::bank::Bank;
use solana_sdk::account::Account;
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

#[derive(Clone, Default, Debug, PartialEq)]
pub struct VoterEntry {
    latency_score: i64, // +1 for low latency, -1 for high latency
    last_slot: Slot,
    last_hash: Hash,
}

// Checks `bank` voter state against the latest tracked `voter_record`. If voter hash has updated,
// check if the voter has new votes to record. Returns a record of votes seen in this checkpoint
// grouped by slot.
fn voter_checkpoint(
    slot: Slot,
    vote_accounts: HashMap<Pubkey, (u64, Account)>,
    voter_record: &mut VoterRecord,
) -> HashMap<Slot, HashSet<Pubkey>> {
    let mut slot_voters: HashMap<Slot, HashSet<Pubkey>> = HashMap::new();
    for (voter_key, (_stake, account)) in vote_accounts {
        let mut voter_entry = voter_record
            .entry(voter_key)
            .or_insert_with(VoterEntry::default);
        if voter_entry.last_hash != account.hash {
            voter_entry.last_hash = account.hash;
            let vote_state = VoteState::from(&account).unwrap();
            for lockout in vote_state.votes.iter().rev() {
                if lockout.slot <= voter_entry.last_slot {
                    break;
                } else if lockout.slot < slot.saturating_sub(MAX_VOTE_DELAY) {
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
        let is_low_latency = voters_seen < max(1, total_voters / 2);
        let score_differential = if is_low_latency { 1 } else { -1 };
        for voter in voter_set {
            let voter_entry = voter_record.get_mut(&voter).unwrap();
            voter_entry.latency_score += score_differential;
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
    bank_slot: Slot,
    vote_accounts: HashMap<Pubkey, (u64, Account)>,
    voter_record: &mut VoterRecord,
    slot_voter_segments: &mut SlotVoterSegments,
) {
    let mut slot_voters = voter_checkpoint(bank_slot, vote_accounts, voter_record);
    for (slot, voters) in slot_voters.drain() {
        let slot_entry = slot_voter_segments.entry(slot).or_insert_with(Vec::new);
        slot_entry.push(voters);
    }

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

fn validator_results(
    baseline_id: &Pubkey,
    excluded_set: &HashSet<Pubkey>,
    vote_accounts: HashMap<Pubkey, (u64, Account)>,
    voter_record: &mut VoterRecord,
) -> (Vec<(Pubkey, f64)>, f64) {
    let mut validator_latency: HashMap<Pubkey, i64> = HashMap::new();
    for (voter_key, (_stake, account)) in vote_accounts {
        let vote_state = VoteState::from(&account).unwrap();
        let voter_entry = voter_record.remove(&voter_key).unwrap();
        // It's possible that there are multiple vote accounts attributed to a validator
        //   so use the max score when duplicates are found
        let entry = validator_latency
            .entry(vote_state.node_pubkey)
            .or_insert(std::i64::MIN);
        *entry = max(*entry, voter_entry.latency_score);
    }

    let baseline = validator_latency.remove(baseline_id).unwrap() as f64;
    let mut results: Vec<(Pubkey, f64)> = validator_latency
        .iter()
        .filter(|(key, _)| !excluded_set.contains(key))
        .map(|(key, latency)| (*key, *latency as f64))
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    (results, baseline)
}

pub fn compute_winners(
    bank: &Bank,
    baseline_id: &Pubkey,
    excluded_set: &HashSet<Pubkey>,
    voter_record: &mut VoterRecord,
    slot_voter_segments: &mut SlotVoterSegments,
) -> Winners {
    // Score the remaining segments leftover from entry processing
    for (_, voter_segments) in slot_voter_segments.iter() {
        score_voters(voter_segments, voter_record);
    }

    let vote_accounts = bank.vote_accounts();
    let (results, baseline) =
        validator_results(baseline_id, excluded_set, vote_accounts, voter_record);
    let num_validators = results.len();
    let num_winners = min(num_validators, 3);

    Winners {
        category: winner::Category::ConfirmationLatency(format!("Baseline Score: {}", baseline)),
        top_winners: normalize_winners(&results[..num_winners]),
        bucket_winners: utils::bucket_winners(&results, baseline as f64, normalize_winners),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::hash::hash;
    use solana_vote_api::vote_state::{Lockout, VoteInit};
    use std::ops::Range;

    #[test]
    fn test_voter_checkpoint() {
        let current_slot = 100;
        let new_vote_account = |vote_range: Range<Slot>| -> Account {
            let mut state = VoteState::default();
            for vote_slot in vote_range {
                state.votes.push_back(Lockout::new(vote_slot));
            }
            let owner = Pubkey::new_rand();
            let mut account = Account::new_data(1, &state, &owner).unwrap();
            account.hash = hash(owner.as_ref());
            account
        };

        let voter1 = Pubkey::new_rand();
        let voter2 = Pubkey::new_rand();
        let voter3 = Pubkey::new_rand();

        let mut vote_accounts = HashMap::new();
        let mut voter_record = HashMap::new();

        // Discard too old votes and add to voter_record
        let too_old_slot = current_slot - MAX_VOTE_DELAY - 1;
        let voter1_account = new_vote_account(too_old_slot..current_slot + 1);
        let voter1_hash = voter1_account.hash;
        vote_accounts.insert(voter1.clone(), (0, voter1_account));

        // Up until last_slot
        let voter2_account = new_vote_account(0..current_slot + 1);
        let voter2_hash = voter2_account.hash;
        vote_accounts.insert(voter2.clone(), (0, voter2_account));
        voter_record.insert(
            voter2,
            VoterEntry {
                last_slot: current_slot - 1,
                ..VoterEntry::default()
            },
        );

        // Ignore same hash
        let voter3_account = new_vote_account(current_slot..current_slot + 1);
        let voter3_hash = voter3_account.hash;
        let voter3_entry = VoterEntry {
            last_hash: voter3_hash,
            ..VoterEntry::default()
        };
        vote_accounts.insert(voter3.clone(), (0, voter3_account));
        voter_record.insert(voter3, voter3_entry.clone());

        let checkpoint = voter_checkpoint(current_slot, vote_accounts, &mut voter_record);
        assert_eq!(checkpoint.len(), (MAX_VOTE_DELAY + 1) as usize);
        let mut expected_voters_set = HashSet::new();
        expected_voters_set.insert(voter1.clone());
        for (slot, voters) in checkpoint {
            // Expected voter 1 and voter 2 for the current_slot
            if slot == current_slot {
                let mut expected_voters_set = expected_voters_set.clone();
                expected_voters_set.insert(voter2.clone());
                assert_eq!(voters, expected_voters_set);
            // Expected only voter 1 for the other slots
            } else {
                assert_eq!(voters, expected_voters_set);
            }
        }

        // Voter 1 should be added to the voter record
        assert_eq!(
            voter_record.get(&voter1).unwrap(),
            &VoterEntry {
                last_slot: current_slot,
                last_hash: voter1_hash,
                ..VoterEntry::default()
            }
        );

        // Voter 2 should be updated
        assert_eq!(
            voter_record.get(&voter2).unwrap(),
            &VoterEntry {
                last_slot: current_slot,
                last_hash: voter2_hash,
                ..VoterEntry::default()
            }
        );

        // Voter 3 should not be updated
        assert_eq!(voter_record.get(&voter3).unwrap(), &voter3_entry);
    }

    #[test]
    fn test_score_voters() {
        let voters = vec![
            Pubkey::new_rand(),
            Pubkey::new_rand(),
            Pubkey::new_rand(),
            Pubkey::new_rand(),
        ];

        let low_latency_set = {
            let mut set = HashSet::new();
            for voter in &voters[..voters.len() - 1] {
                set.insert(voter.clone());
            }
            set
        };

        let high_latency_set = {
            let mut set = HashSet::new();
            set.insert(voters[voters.len() - 1]);
            set
        };

        let voter_sets = vec![low_latency_set, high_latency_set];

        let mut voter_record = {
            let mut map = HashMap::new();
            for voter in voters.iter() {
                map.insert(voter.clone(), VoterEntry::default());
            }
            map
        };

        score_voters(&voter_sets, &mut voter_record);

        for voter in &voters[..voters.len() - 1] {
            assert_eq!(voter_record.get(voter).unwrap().latency_score, 1);
        }
        let last_voter = voters[voters.len() - 1];
        assert_eq!(voter_record.get(&last_voter).unwrap().latency_score, -1);
    }

    #[test]
    fn test_on_entry() {
        let current_slot = 100;
        let recent_slot = 99;
        let old_slot = current_slot - MAX_VOTE_DELAY - 1;
        let new_vote_account = |vote_range: Range<Slot>| -> Account {
            let mut state = VoteState::default();
            for vote_slot in vote_range {
                state.votes.push_back(Lockout::new(vote_slot));
            }
            let owner = Pubkey::new_rand();
            let mut account = Account::new_data(1, &state, &owner).unwrap();
            account.hash = hash(owner.as_ref());
            account
        };

        let voter1 = Pubkey::new_rand();
        let voter2 = Pubkey::new_rand();
        let voter3 = Pubkey::new_rand();

        let mut vote_accounts = HashMap::new();
        vote_accounts.insert(
            voter1.clone(),
            (0, new_vote_account(current_slot..current_slot + 1)),
        );
        vote_accounts.insert(
            voter2.clone(),
            (0, new_vote_account(current_slot..current_slot + 1)),
        );
        vote_accounts.insert(
            voter3.clone(),
            (0, new_vote_account(recent_slot..current_slot + 1)),
        );

        let recent_slot_first_voter_set = {
            let mut set = HashSet::new();
            set.insert(voter1.clone());
            set.insert(voter2.clone());
            set
        };

        let expected_recent_slot_second_voter_set = {
            let mut set = HashSet::new();
            set.insert(voter3.clone());
            set
        };

        let old_slot_voter_set = {
            let mut set = HashSet::new();
            set.insert(voter1.clone());
            set
        };

        let expected_current_slot_voter_set = {
            let mut set = HashSet::new();
            set.insert(voter1.clone());
            set.insert(voter2.clone());
            set.insert(voter3.clone());
            set
        };

        let mut slot_voter_segments = BTreeMap::default();
        slot_voter_segments.insert(old_slot, vec![old_slot_voter_set]);
        slot_voter_segments.insert(recent_slot, vec![recent_slot_first_voter_set.clone()]);

        let mut voter_record = HashMap::new();
        on_entry(
            current_slot,
            vote_accounts,
            &mut voter_record,
            &mut slot_voter_segments,
        );
        assert_eq!(slot_voter_segments.len(), 2);

        // Should periodically purge and score slot_voter_segments
        assert!(slot_voter_segments.get(&old_slot).is_none());
        assert_eq!(voter_record[&voter1].latency_score, 1);

        // Should push back a new voter set after latest checkpoint
        assert_eq!(
            slot_voter_segments.get(&recent_slot).unwrap(),
            &vec![
                recent_slot_first_voter_set,
                expected_recent_slot_second_voter_set
            ]
        );

        // Should create new voter segment for new slot
        assert_eq!(
            slot_voter_segments.get(&current_slot).unwrap(),
            &vec![expected_current_slot_voter_set]
        );
    }

    #[test]
    fn test_validator_results() {
        let new_vote_account = |validator_id: &Pubkey| -> Account {
            let state = VoteState::new(&VoteInit {
                node_pubkey: validator_id.clone(),
                ..VoteInit::default()
            });
            Account::new_data(1, &state, &Pubkey::new_rand()).unwrap()
        };

        let validator1 = Pubkey::new_rand();
        let validator2 = Pubkey::new_rand();
        let bootstrap_leader = Pubkey::new_rand();
        let baseline_validator = Pubkey::new_rand();

        let voter1 = Pubkey::new_rand();
        let voter2 = Pubkey::new_rand();
        let voter3 = Pubkey::new_rand();

        let mut vote_accounts = HashMap::new();
        vote_accounts.insert(voter1.clone(), (0, new_vote_account(&validator1)));
        vote_accounts.insert(voter2.clone(), (0, new_vote_account(&validator2)));
        vote_accounts.insert(voter3.clone(), (0, new_vote_account(&baseline_validator)));

        let mut voter_record = HashMap::new();
        voter_record.insert(
            voter1,
            VoterEntry {
                latency_score: 100,
                ..VoterEntry::default()
            },
        );
        voter_record.insert(
            voter2,
            VoterEntry {
                latency_score: 200,
                ..VoterEntry::default()
            },
        );
        voter_record.insert(
            voter3,
            VoterEntry {
                latency_score: 300,
                ..VoterEntry::default()
            },
        );

        let excluded_set = {
            let mut set = HashSet::new();
            set.insert(bootstrap_leader);
            set.insert(baseline_validator);
            set
        };

        let (results, baseline) = validator_results(
            &baseline_validator,
            &excluded_set,
            vote_accounts,
            &mut voter_record,
        );
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (validator2, 200f64));
        assert_eq!(results[1], (validator1, 100f64));
        assert_eq!(baseline, 300f64);
    }
}
