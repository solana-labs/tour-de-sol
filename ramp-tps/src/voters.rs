use solana_client::rpc_client::RpcClient;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, KeypairUtil};
use solana_sdk::transaction::Transaction;
use solana_stake_api::stake_instruction;
use solana_stake_api::stake_state::Authorized as StakeAuthorized;
use std::str::FromStr;
use log::*;

pub fn fetch_remaining_voters(rpc_client: &RpcClient) -> Vec<Pubkey> {
    rpc_client
        .get_vote_accounts()
        .unwrap()
        .current
        .into_iter()
        .filter_map(|info| Pubkey::from_str(&info.vote_pubkey).ok())
        .collect()
}

pub fn award_stake(
    rpc_client: &RpcClient,
    mint_keypair: &Keypair,
    voters: Vec<Pubkey>,
    sol_gift: u64,
) {
    let recent_blockhash = rpc_client.get_recent_blockhash().unwrap().0;
    for vote_account_pubkey in voters {
        let stake_account_keypair = Keypair::new();
        let mut transaction = Transaction::new_signed_instructions(
            &[mint_keypair, &stake_account_keypair],
            stake_instruction::create_stake_account_and_delegate_stake(
                &mint_keypair.pubkey(),
                &stake_account_keypair.pubkey(),
                &vote_account_pubkey,
                &StakeAuthorized::auto(&stake_account_keypair.pubkey()),
                sol_to_lamports(sol_gift as f64),
            ),
            recent_blockhash,
        );

        info!("Delegating {} SOL to {}", sol_gift, vote_account_pubkey);
        if let Err(err) = rpc_client
            .send_and_confirm_transaction(&mut transaction, &[mint_keypair, &stake_account_keypair]) {
                error!("Failed to delgate {} SOL to {}: {}", sol_gift, vote_account_pubkey, err);
        }
    }
}
