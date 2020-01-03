use crate::notifier::Notifier;
use log::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::InstructionError;
use solana_sdk::{
    account_utils::State,
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, KeypairUtil},
    transaction::Transaction,
};
use solana_stake_program::stake_state::{Lockup, StakeState};
use solana_stake_program::{stake_instruction, stake_state::Authorized as StakeAuthorized};
use std::{str::FromStr, thread::sleep, time::Duration};

pub fn fetch_remaining_voters(rpc_client: &RpcClient) -> Vec<(Pubkey, Pubkey)> {
    match rpc_client.get_vote_accounts() {
        Err(err) => {
            warn!("Failed to get_vote_accounts(): {}", err);
            vec![]
        }
        Ok(vote_accounts) => vote_accounts
            .current
            .into_iter()
            .filter_map(|info| {
                if let (Ok(node_pubkey), Ok(vote_pubkey)) = (
                    Pubkey::from_str(&info.node_pubkey),
                    Pubkey::from_str(&info.vote_pubkey),
                ) {
                    Some((node_pubkey, vote_pubkey))
                } else {
                    None
                }
            })
            .collect(),
    }
}

/// Endlessly retry stake delegation until success
fn delegate_stake(
    rpc_client: &RpcClient,
    faucet_keypair: &Keypair,
    vote_account_pubkey: Pubkey,
    sol_gift: u64,
) {
    let stake_account_keypair = Keypair::new();
    let mut retry_count = 0;
    loop {
        let recent_blockhash = loop {
            match rpc_client.get_recent_blockhash() {
                Ok(response) => break response.0,
                Err(err) => {
                    error!("Failed to get recent blockhash: {}", err);
                    sleep(Duration::from_secs(5));
                }
            }
        };

        let mut transaction = Transaction::new_signed_instructions(
            &[faucet_keypair, &stake_account_keypair],
            stake_instruction::create_account_and_delegate_stake(
                &faucet_keypair.pubkey(),
                &stake_account_keypair.pubkey(),
                &vote_account_pubkey,
                &StakeAuthorized::auto(&faucet_keypair.pubkey()),
                &Lockup::default(),
                sol_to_lamports(sol_gift as f64),
            ),
            recent_blockhash,
        );

        // Check if stake was delegated but just failed to confirm on an earlier attempt
        if retry_count > 0 {
            if let Ok(account) = rpc_client.get_account(&stake_account_keypair.pubkey()) {
                let result: Result<StakeState, InstructionError> = account.state();
                if result.is_ok() {
                    break;
                }
            }
        }

        if let Err(err) = rpc_client.send_and_confirm_transaction(
            &mut transaction,
            &[faucet_keypair, &stake_account_keypair],
        ) {
            error!(
                "Failed to delegate stake (retries: {}): {}",
                retry_count, err
            );
            retry_count += 1;
            sleep(Duration::from_secs(5));
        } else {
            break;
        }
    }
}

/// Award stake to the surviving validators by delegating stake to their vote account
pub fn award_stake(
    rpc_client: &RpcClient,
    faucet_keypair: &Keypair,
    voters: Vec<(String, Pubkey)>,
    sol_gift: u64,
    notifier: &mut Notifier,
) {
    for (node_pubkey, vote_account_pubkey) in voters {
        info!("Delegate {} SOL to {}", sol_gift, node_pubkey);
        delegate_stake(rpc_client, faucet_keypair, vote_account_pubkey, sol_gift);
        notifier.buffer(format!("Delegated {} SOL to {}", sol_gift, node_pubkey));
    }
    notifier.flush();
}
