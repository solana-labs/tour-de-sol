use log::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, KeypairUtil};
use solana_sdk::transaction::Transaction;
use solana_stake_api::stake_instruction;
use solana_stake_api::stake_state::Authorized as StakeAuthorized;
use std::str::FromStr;

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

pub fn award_stake(
    rpc_client: &RpcClient,
    mint_keypair: &Keypair,
    voters: Vec<(String, Pubkey)>,
    sol_gift: u64,
    slack_logger: &crate::slack::Logger,
) {
    let recent_blockhash = rpc_client.get_recent_blockhash().unwrap().0;
    for (node_pubkey, vote_account_pubkey) in voters {
        let stake_account_keypair = Keypair::new();
        let mut transaction = Transaction::new_signed_instructions(
            &[mint_keypair, &mint_keypair],
            stake_instruction::create_stake_account_and_delegate_stake(
                &mint_keypair.pubkey(),
                &stake_account_keypair.pubkey(),
                &vote_account_pubkey,
                &StakeAuthorized::auto(&mint_keypair.pubkey()),
                sol_to_lamports(sol_gift as f64),
            ),
            recent_blockhash,
        );

        if let Err(err) = rpc_client
            .send_and_confirm_transaction(&mut transaction, &[mint_keypair, &stake_account_keypair])
        {
            slack_logger.info(&format!(
                "Failed to delegate {} SOL to {}: {}",
                sol_gift, node_pubkey, err
            ));
        } else {
            slack_logger.info(&format!("Delegated {} SOL to {}", sol_gift, node_pubkey));
        }
    }
}
