use crate::winner::Winner;
use solana_core::blocktree::Blocktree;
use solana_sdk::clock::Slot;
use solana_sdk::pubkey::Pubkey;

/// Returns an ordered list of slots for the blockchain ending with `last_block` and starting with
/// `first_block`
pub fn block_chain(first_block: Slot, last_block: Slot, blocktree: &Blocktree) -> Vec<Slot> {
    let mut block_chain = Vec::new();
    let mut block_slot = last_block;
    loop {
        block_chain.push(block_slot);
        if block_slot == first_block {
            break;
        }
        block_slot = blocktree.meta(block_slot).unwrap().unwrap().parent_slot;
    }
    block_chain.into_iter().rev().collect()
}

/// Transforms a validator score into a formatted score string for display purposes
pub type WinnerTransform = fn(&[(Pubkey, f64)]) -> Vec<Winner>;

const HIGH_BUCKET: &str = "Greater than 95% of the baseline";
const MID_BUCKET: &str = "95 - 75% of the baseline";
const LOW_BUCKET: &str = "75 - 50% of the baseline";

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

    let hi_bucket_index = find_bucket_index(0.95 * baseline);
    let hi = &results[..hi_bucket_index];
    bucket_winners.push((HIGH_BUCKET.to_string(), winner_transform(hi)));

    let md_bucket_index = find_bucket_index(0.75 * baseline);
    let md = &results[hi_bucket_index..md_bucket_index];
    bucket_winners.push((MID_BUCKET.to_string(), winner_transform(md)));

    let lo_bucket_index = find_bucket_index(0.5 * baseline);
    let lo = &results[md_bucket_index..lo_bucket_index];
    bucket_winners.push((LOW_BUCKET.to_string(), winner_transform(lo)));

    bucket_winners
}

/// Return an error if any Pubkey cannot be parsed.
pub fn is_pubkey_list(string: String) -> Result<(), String> {
    let first_err = string
        .split(' ')
        .filter_map(|string| match string.parse::<Pubkey>() {
            Ok(_) => None,
            Err(_) => Some(format!("\"{}\" is not a valid public key", string)),
        })
        .nth(0);

    match first_err {
        Some(err) => Err(err),
        None => Ok(()),
    }
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

    #[test]
    fn test_is_pubkey_list() {
        let pubkey1 = Pubkey::new_rand();
        let pubkey2 = Pubkey::new_rand();

        assert!(is_pubkey_list(format!("{}", pubkey1)).is_ok());
        assert!(is_pubkey_list(format!("{} {}", pubkey1, pubkey2)).is_ok());
        assert!(is_pubkey_list(format!("invalid")).is_err());
        assert!(is_pubkey_list(format!("{} invalid", pubkey1)).is_err());
        assert!(is_pubkey_list(format!("invalid {}", pubkey1)).is_err());
    }
}
