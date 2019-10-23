use crate::utils::sleep_n_slots;
use log::info;
use solana_client::rpc_client::RpcClient;
use solana_sdk::genesis_block::GenesisBlock;
use solana_sdk::sysvar::stake_history::{self, StakeHistory, StakeHistoryEntry};
use solana_stake_api::config::Config as StakeConfig;
use std::thread::sleep;
use std::time::Duration;

fn epochs_until_activation(mut stake_entry: StakeHistoryEntry, stake_config: &StakeConfig) -> u64 {
    let mut epochs = 0;
    loop {
        if (stake_entry.activating as f64 / stake_entry.effective.max(1) as f64) < 0.05 {
            break;
        }
        let max_warmup_stake = (stake_entry.effective as f64 * stake_config.warmup_rate) as u64;
        let warmup_stake = stake_entry.activating.min(max_warmup_stake);
        stake_entry.effective += warmup_stake;
        stake_entry.activating -= warmup_stake;
        epochs += 1;
    }
    epochs
}

fn stake_activation_epoch_entry(
    activation_epoch: u64,
    rpc_client: &RpcClient,
) -> Option<StakeHistoryEntry> {
    let stake_history_account = rpc_client.get_account(&stake_history::id()).ok()?;
    let stake_history = StakeHistory::from_account(&stake_history_account)?;
    stake_history.get(&activation_epoch).cloned()
}

pub fn wait_for_activation(
    activation_epoch: u64,
    rpc_client: &RpcClient,
    stake_config: &StakeConfig,
    genesis_block: &GenesisBlock,
) {
    info!("Fetching current epoch info...");
    let epoch_info = rpc_client.get_epoch_info().unwrap();
    info!("Current epoch info: {:?}", &epoch_info);
    let current_epoch = epoch_info.epoch;

    // Sleep until activation_epoch has finished
    let sleep_epochs = (activation_epoch + 1).saturating_sub(current_epoch);
    let slots_per_epoch = genesis_block.epoch_schedule.slots_per_epoch;
    if sleep_epochs > 0 {
        let sleep_slots = sleep_epochs * slots_per_epoch - epoch_info.slot_index;
        info!(
            "Waiting until activation epoch ({}) is finished...",
            activation_epoch
        );
        sleep_n_slots(sleep_slots, genesis_block);
    }

    loop {
        info!(
            "Fetching stake history entry for activation epoch ({})...",
            activation_epoch
        );
        if let Some(stake_entry) = stake_activation_epoch_entry(activation_epoch, &rpc_client) {
            info!("Stake history entry: {:?}", &stake_entry);
            let num_epochs = epochs_until_activation(stake_entry, stake_config);
            let warmed_up_epoch = activation_epoch + num_epochs;
            if warmed_up_epoch > current_epoch {
                let epoch_info = rpc_client.get_epoch_info().unwrap();
                info!(
                    "Waiting until epoch {} for stake to warmup...",
                    warmed_up_epoch
                );
                let sleep_epochs = warmed_up_epoch - current_epoch;
                let sleep_slots = sleep_epochs * slots_per_epoch - epoch_info.slot_index;
                sleep_n_slots(sleep_slots, genesis_block);
            }
            break;
        } else {
            info!(
                "Failed to fetch stake history entry activation epoch: {}",
                activation_epoch
            );
            sleep(Duration::from_secs(5));
        }
    }
}
