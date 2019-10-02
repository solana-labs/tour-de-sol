//! Calculates the winners of the "Highest Availability" category in Tour de Sol by dividing the
//! credits of validator vote accounts by the total number of blocks in the chain.
//!
//! The top 3 validators will receive the top prizes and validators will be awarded additional
//! prizes if they perform well enough against the Solana team's validator as a baseline.

use crate::prize::{self, Winner, Winners};
use solana_runtime::bank::Bank;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;
use solana_vote_api::vote_state::{VoteState, MAX_LOCKOUT_HISTORY};
use std::cmp::{max, min};
use std::collections::HashMap;

const HIGH_BUCKET: &str = "Greater than 95% of the baseline";
const MID_BUCKET: &str = "95 - 75% of the baseline";
const LOW_BUCKET: &str = "75 - 50% of the baseline";

fn bucket_winners(results: &[(Pubkey, f64)], baseline: f64) -> Vec<(String, Vec<Winner>)> {
    let find_bucket_index = |value: f64| -> usize {
        let mut index = 0;
        while index < results.len() && results[index].1 > value {
            index = index + 1;
        }
        index
    };

    let mut bucket_winners = Vec::new();

    let hi_bucket_index = find_bucket_index(0.95 * baseline);
    let hi = &results[..hi_bucket_index];
    bucket_winners.push((HIGH_BUCKET.to_string(), normalize_winners(hi)));

    let md_bucket_index = find_bucket_index(0.75 * baseline);
    let md = &results[hi_bucket_index..md_bucket_index];
    bucket_winners.push((MID_BUCKET.to_string(), normalize_winners(md)));

    let lo_bucket_index = find_bucket_index(0.5 * baseline);
    let lo = &results[md_bucket_index..lo_bucket_index];
    bucket_winners.push((LOW_BUCKET.to_string(), normalize_winners(lo)));

    bucket_winners
}

fn normalize_winners(winners: &[(Pubkey, f64)]) -> Vec<(Pubkey, String)> {
    winners
        .iter()
        .map(|(key, availability)| {
            (
                *key,
                format!("{:.*}% availability", 3, availability * 100f64),
            )
        })
        .collect()
}

fn validator_credits(vote_accounts: HashMap<Pubkey, (u64, Account)>) -> HashMap<Pubkey, u64> {
    let mut validator_credits = HashMap::new();
    for (_voter_key, (_stake, account)) in vote_accounts {
        if let Some(vote_state) = VoteState::from(&account) {
            if let Some(credits) = validator_credits.get_mut(&vote_state.node_pubkey) {
                *credits = max(*credits, vote_state.credits());
            } else {
                validator_credits.insert(vote_state.node_pubkey, vote_state.credits());
            }
        }
    }
    validator_credits
}

fn validator_results(
    validator_credits: HashMap<Pubkey, u64>,
    total_credits: u64,
) -> Vec<(Pubkey, f64)> {
    let mut results: Vec<(Pubkey, f64)> = validator_credits
        .iter()
        .map(|(key, credits)| (*key, *credits as f64 / total_credits as f64))
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results
}

pub fn compute_winners(bank: &Bank, baseline_id: &Pubkey) -> Winners {
    let vote_accounts = bank.vote_accounts();
    let mut validator_credits = validator_credits(vote_accounts);
    let baseline_credits = validator_credits
        .remove(baseline_id)
        .expect("Solana baseline validator not found");

    let total_blocks = bank.block_height();
    let total_credits = total_blocks.saturating_sub(MAX_LOCKOUT_HISTORY as u64);
    let results = validator_results(validator_credits, total_credits);

    let num_validators = results.len();
    let num_winners = min(num_validators, 3);
    let baseline = baseline_credits as f64 / total_credits as f64;

    Winners {
        category: prize::Category::Availability,
        top_winners: normalize_winners(&results[..num_winners]),
        bucket_winners: bucket_winners(&results, baseline),
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
        credits_map.insert(top_validator, 1000);
        credits_map.insert(bottom_validator, 100);

        let results = validator_results(credits_map, 1000);
        assert_eq!(results[0], (top_validator, 1.0));
        assert_eq!(results[1], (bottom_validator, 0.1));
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
            Account::new_data(
                1,
                &state,
                &Pubkey::new_rand(),
            )
            .unwrap()
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

        assert_eq!(
            expected_credits,
            validator_credits(vote_accounts)
        );
    }

    #[test]
    fn test_bucket_winners() {
        let mut results = Vec::new();

        let expected_hi_bucket = vec![
            (Pubkey::new_rand(), 1.00),
            (Pubkey::new_rand(), 0.96),
        ];

        let expected_md_bucket = vec![];

        let expected_lo_bucket = vec![
            (Pubkey::new_rand(), 0.75),
            (Pubkey::new_rand(), 0.75),
            (Pubkey::new_rand(), 0.51),
        ];

        results.extend(expected_hi_bucket.iter());
        results.extend(expected_md_bucket.iter());
        results.extend(expected_lo_bucket.iter());
        results.push((Pubkey::new_rand(), 0.50));

        let baseline = 1.0;
        let bucket_winners = bucket_winners(&results, baseline);

        assert_eq!(bucket_winners[0].1, normalize_winners(&expected_hi_bucket));
        assert_eq!(bucket_winners[1].1, normalize_winners(&expected_md_bucket));
        assert_eq!(bucket_winners[2].1, normalize_winners(&expected_lo_bucket));
    }
}
