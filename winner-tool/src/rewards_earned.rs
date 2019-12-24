//! Calculates the winners of the "Most Rewards Earned" category in Tour de SOL by summing the
//! balances of all stake and vote accounts attributed to a particular validator.
//!
//! The top 3 validators will receive the top prizes and validators will be awarded additional
//! prizes if they place into the following buckets:
//!
//! `high` - Top 25%
//! `medium` - Top 25-50%
//! `low` - Top 50-90%
//! `bottom` - Bottom 10%

use crate::winner::{self, Winner, Winners};
use solana_runtime::bank::Bank;
use solana_sdk::{account::Account, native_token::lamports_to_sol, pubkey::Pubkey};
use solana_stake_program::stake_state::Delegation;
use solana_vote_program::vote_state::VoteState;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};

const HIGH_BUCKET: &str = "Top 25%";
const MEDUIM_BUCKET: &str = "25% to 50%";
const LOW_BUCKET: &str = "50% to 90%";
const BOTTOM_BUCKET: &str = "Bottom 10%";

fn voter_stake_rewards(stake_delegations: HashMap<Pubkey, Delegation>) -> HashMap<Pubkey, u64> {
    let mut voter_stake_sum: HashMap<Pubkey, u64> = HashMap::new();
    for (_key, delegation) in stake_delegations {
        voter_stake_sum
            .entry(delegation.voter_pubkey)
            .and_modify(|stake_sum| *stake_sum += delegation.stake)
            .or_insert(delegation.stake);
    }
    voter_stake_sum
}

fn validator_results(
    validator_reward_map: HashMap<Pubkey, u64>,
    excluded_set: &HashSet<Pubkey>,
    starting_balance: u64,
) -> Vec<(Pubkey, i64)> {
    let mut validator_rewards: Vec<(Pubkey, u64)> = validator_reward_map
        .iter()
        .filter(|(key, _)| !excluded_set.contains(key))
        .map(|(key, balance)| (*key, *balance))
        .collect();

    // Sort descending and calculate results
    validator_rewards.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    validator_rewards
        .into_iter()
        .map(|(key, earned)| (key, (earned as i64) - (starting_balance as i64)))
        .collect()
}

fn validator_rewards(
    mut voter_stake_rewards: HashMap<Pubkey, u64>,
    vote_accounts: HashMap<Pubkey, (u64, Account)>,
) -> HashMap<Pubkey, u64> {
    // Sum validator earned reward totals (stake rewards + commission)
    let mut validator_reward_map: HashMap<Pubkey, u64> = HashMap::new();
    for (voter_key, (_stake, account)) in vote_accounts {
        if let Some(vote_state) = VoteState::from(&account) {
            let voter_commission = account.lamports;
            let voter_stake_reward = voter_stake_rewards.remove(&voter_key).unwrap_or_default();

            let validator_id = vote_state.node_pubkey;
            validator_reward_map
                .entry(validator_id)
                .and_modify(|validator_reward| {
                    // If multiple vote accounts are detected, take the max
                    *validator_reward =
                        max(*validator_reward, voter_commission + voter_stake_reward);
                })
                .or_insert(voter_commission + voter_stake_reward);
        }
    }

    validator_reward_map
}

// Bucket validators for reward distribution
fn bucket_winners(results: &[(Pubkey, i64)]) -> Vec<(String, Vec<Winner>)> {
    let num_validators = results.len();
    let mut bucket_winners = Vec::new();

    // Tied winners should not end up in different buckets
    let handle_ties = |mut index: usize| -> usize {
        while (index + 1 < num_validators) && (results[index].1 == results[index + 1].1) {
            index += 1;
        }
        index
    };

    // Top 25% of validators
    let high_bucket_index = handle_ties(max(1, num_validators / 4) - 1);
    let high = &results[..=high_bucket_index];
    bucket_winners.push((HIGH_BUCKET.to_string(), normalize_winners(high)));

    // Top 25-50% of validators
    let medium_bucket_index = handle_ties(max(1, num_validators / 2) - 1);
    let medium = &results[(high_bucket_index + 1)..=medium_bucket_index];
    bucket_winners.push((MEDUIM_BUCKET.to_string(), normalize_winners(medium)));

    // Top 50-90% of validators
    let low_bucket_index = handle_ties(max(1, 9 * num_validators / 10) - 1);
    let low = &results[(medium_bucket_index + 1)..=low_bucket_index];
    bucket_winners.push((LOW_BUCKET.to_string(), normalize_winners(low)));

    // Bottom 10% of validators
    let bottom_bucket_index = handle_ties(max(1, num_validators) - 1);
    let bottom = &results[(low_bucket_index + 1)..=bottom_bucket_index];
    bucket_winners.push((BOTTOM_BUCKET.to_string(), normalize_winners(bottom)));

    bucket_winners
}

fn normalize_winners(winners: &[(Pubkey, i64)]) -> Vec<Winner> {
    winners
        .iter()
        .map(|(key, earned)| {
            let mut earned = *earned;
            let mut sign = "";
            if earned < 0 {
                sign = "-";
                earned = -earned;
            }
            (
                *key,
                format!(
                    "Earned {}{:.5} SOL ({}{} lamports) in stake rewards and commission",
                    sign,
                    lamports_to_sol(earned as u64),
                    sign,
                    earned
                ),
            )
        })
        .collect()
}

pub fn compute_winners(
    bank: &Bank,
    excluded_set: &HashSet<Pubkey>,
    starting_balance: u64,
) -> Winners {
    let voter_stake_rewards = voter_stake_rewards(bank.stake_delegations());
    let validator_reward_map = validator_rewards(voter_stake_rewards, bank.vote_accounts());
    let results = validator_results(validator_reward_map, excluded_set, starting_balance);
    let num_validators = results.len();
    let num_winners = min(num_validators, 3);
    assert!(num_winners > 0);

    Winners {
        category: winner::Category::RewardsEarned,
        top_winners: normalize_winners(&results[..num_winners]),
        bucket_winners: bucket_winners(&results),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_vote_program::vote_state::VoteInit;

    #[test]
    fn test_validator_results() {
        let mut rewards_map = HashMap::new();
        let top_validator = Pubkey::new_rand();
        let bottom_validator = Pubkey::new_rand();
        let excluded_validator = Pubkey::new_rand();
        rewards_map.insert(top_validator, 1000);
        rewards_map.insert(bottom_validator, 10);
        rewards_map.insert(excluded_validator, 100);

        let excluded_set = {
            let mut set = HashSet::new();
            set.insert(excluded_validator);
            set
        };

        let results = validator_results(rewards_map, &excluded_set, 100);
        assert_eq!(results[0], (top_validator, 900));
        assert_eq!(results[1], (bottom_validator, -90));
    }

    #[test]
    fn test_validator_rewards() {
        let new_vote_account = |lamports: u64, validator_id: &Pubkey| -> Account {
            Account::new_data(
                lamports,
                &VoteState::new(&VoteInit {
                    node_pubkey: validator_id.clone(),
                    ..VoteInit::default()
                }),
                &Pubkey::new_rand(),
            )
            .unwrap()
        };

        let validator1 = Pubkey::new_rand();
        let validator2 = Pubkey::new_rand();

        let mut vote_accounts = HashMap::new();
        let voter1 = Pubkey::new_rand();
        vote_accounts.insert(voter1.clone(), (0, new_vote_account(100, &validator1)));
        vote_accounts.insert(Pubkey::new_rand(), (0, new_vote_account(100, &validator2)));
        vote_accounts.insert(Pubkey::new_rand(), (0, new_vote_account(200, &validator2)));

        let voter_stake_rewards = {
            let mut map = HashMap::new();
            map.insert(voter1, 1000);
            map
        };

        let expected_rewards = {
            let mut map = HashMap::new();
            map.insert(validator1, 1100);
            map.insert(validator2, 200);
            map
        };

        assert_eq!(
            expected_rewards,
            validator_rewards(voter_stake_rewards, vote_accounts)
        );
    }

    #[test]
    fn test_voter_stake_rewards() {
        let new_stake_delegation = |stake: u64, voter_pubkey: &Pubkey| -> Delegation {
            Delegation {
                voter_pubkey: voter_pubkey.clone(),
                stake,
                ..Delegation::default()
            }
        };

        let voter_pubkey1 = Pubkey::new_rand();
        let voter_pubkey2 = Pubkey::new_rand();

        let mut stake_accounts = HashMap::new();
        stake_accounts.insert(
            Pubkey::new_rand(),
            new_stake_delegation(100, &voter_pubkey1),
        );
        stake_accounts.insert(
            Pubkey::new_rand(),
            new_stake_delegation(100, &voter_pubkey2),
        );
        stake_accounts.insert(
            Pubkey::new_rand(),
            new_stake_delegation(100, &voter_pubkey2),
        );

        let expected = {
            let mut map = HashMap::new();
            map.insert(voter_pubkey1, 100);
            map.insert(voter_pubkey2, 200);
            map
        };

        assert_eq!(expected, voter_stake_rewards(stake_accounts));
    }

    #[test]
    fn test_bucket_winners() {
        let mut results = Vec::new();

        let expected_high_bucket = vec![(Pubkey::new_rand(), 8_000), (Pubkey::new_rand(), 7_000)];

        let expected_medium_bucket = vec![(Pubkey::new_rand(), 6_000), (Pubkey::new_rand(), 5_000)];

        let expected_low_bucket = vec![
            (Pubkey::new_rand(), 4_000),
            (Pubkey::new_rand(), 3_000),
            (Pubkey::new_rand(), 2_000),
        ];
        let expected_bottom_bucket = vec![(Pubkey::new_rand(), 1_000)];

        results.extend(expected_high_bucket.iter());
        results.extend(expected_medium_bucket.iter());
        results.extend(expected_low_bucket.iter());
        results.extend(expected_bottom_bucket.iter());

        let bucket_winners = bucket_winners(&results);

        assert_eq!(
            bucket_winners[0].1,
            normalize_winners(&expected_high_bucket)
        );
        assert_eq!(
            bucket_winners[1].1,
            normalize_winners(&expected_medium_bucket)
        );
        assert_eq!(bucket_winners[2].1, normalize_winners(&expected_low_bucket));
        assert_eq!(
            bucket_winners[3].1,
            normalize_winners(&expected_bottom_bucket)
        );
    }

    #[test]
    fn test_bucket_winners_with_ties() {
        let mut results = Vec::new();

        // Ties should all get bucketed together
        let expected_high_bucket = vec![
            (Pubkey::new_rand(), 8_000),
            (Pubkey::new_rand(), 7_000),
            (Pubkey::new_rand(), 7_000),
            (Pubkey::new_rand(), 7_000),
        ];

        let expected_medium_bucket = vec![];

        let expected_low_bucket = vec![
            (Pubkey::new_rand(), 4_000),
            (Pubkey::new_rand(), 3_000),
            (Pubkey::new_rand(), 2_000),
        ];

        let expected_bottom_bucket = vec![(Pubkey::new_rand(), 1_000)];

        results.extend(expected_high_bucket.iter());
        results.extend(expected_medium_bucket.iter());
        results.extend(expected_low_bucket.iter());
        results.extend(expected_bottom_bucket.iter());

        let bucket_winners = bucket_winners(&results);

        assert_eq!(
            bucket_winners[0].1,
            normalize_winners(&expected_high_bucket)
        );
        assert_eq!(
            bucket_winners[1].1,
            normalize_winners(&expected_medium_bucket)
        );
        assert_eq!(bucket_winners[2].1, normalize_winners(&expected_low_bucket));
        assert_eq!(
            bucket_winners[3].1,
            normalize_winners(&expected_bottom_bucket)
        );
    }
}
