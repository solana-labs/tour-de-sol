use crate::winner::Winner;
use solana_ledger::blockstore::Blockstore;
use solana_sdk::clock::Slot;
use solana_sdk::pubkey::Pubkey;

/// Returns an ordered list of slots for the blockchain ending with `last_block` and starting with
/// `first_block`
pub fn block_chain(first_block: Slot, last_block: Slot, blockstore: &Blockstore) -> Vec<Slot> {
    let mut block_chain = Vec::new();
    let mut block_slot = last_block;
    loop {
        block_chain.push(block_slot);
        if block_slot == first_block {
            break;
        }
        block_slot = blockstore.meta(block_slot).unwrap().unwrap().parent_slot;
    }
    block_chain.into_iter().rev().collect()
}

/// Transforms a validator score into a formatted score string for display purposes
pub type WinnerTransform = fn(&[(Pubkey, f64)]) -> Vec<Winner>;

const HIGH_BUCKET: &str = "Greater than 95% of the baseline";
const MEDIUM_BUCKET: &str = "95% to 75% of the baseline";
const LOW_BUCKET: &str = "75% to 50% of the baseline";
const BOTTOM_BUCKET: &str = "Under 50% of the baseline";

/// Bucket winners relative to the Solana validator baseline.
pub fn bucket_winners(
    results: &[(Pubkey, f64)],
    baseline: f64,
    winner_transform: WinnerTransform,
) -> Vec<(String, Vec<Winner>)> {
    let find_bucket_index = |value: f64| -> usize {
        results
            .iter()
            .rposition(|&result| result.1 > value)
            .map(|position| position + 1)
            .unwrap_or(0)
    };

    let mut bucket_winners = Vec::new();

    let high_bucket_index = find_bucket_index(0.95 * baseline);
    let high = &results[..high_bucket_index];
    bucket_winners.push((HIGH_BUCKET.to_string(), winner_transform(high)));

    let medium_bucket_index = find_bucket_index(0.75 * baseline);
    let medium = &results[high_bucket_index..medium_bucket_index];
    bucket_winners.push((MEDIUM_BUCKET.to_string(), winner_transform(medium)));

    let low_bucket_index = find_bucket_index(0.5 * baseline);
    let low = &results[medium_bucket_index..low_bucket_index];
    bucket_winners.push((LOW_BUCKET.to_string(), winner_transform(low)));

    let bottom_bucket_index = find_bucket_index(-1.);
    let bottom = &results[low_bucket_index..bottom_bucket_index];
    bucket_winners.push((BOTTOM_BUCKET.to_string(), winner_transform(bottom)));

    bucket_winners
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normalize_winners(winners: &[(Pubkey, f64)]) -> Vec<Winner> {
        winners
            .iter()
            .map(|(key, score)| (*key, score.to_string()))
            .collect()
    }

    #[test]
    fn test_bucket_winners() {
        let mut results = Vec::new();

        let expected_hi_bucket = vec![(Pubkey::new_rand(), 1.00), (Pubkey::new_rand(), 0.96)];

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
        let bucket_winners = bucket_winners(&results, baseline, normalize_winners);

        assert_eq!(bucket_winners[0].1, normalize_winners(&expected_hi_bucket));
        assert_eq!(bucket_winners[1].1, normalize_winners(&expected_md_bucket));
        assert_eq!(bucket_winners[2].1, normalize_winners(&expected_lo_bucket));
    }
}
