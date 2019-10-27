use crate::{slack, utils};
use log::*;
use solana_client::{rpc_client::RpcClient, rpc_request::RpcEpochInfo};
use solana_sdk::{
    genesis_block::GenesisBlock,
    sysvar::stake_history::{self, StakeHistory, StakeHistoryEntry},
};
use solana_stake_api::config::Config as StakeConfig;
use std::{thread::sleep, time::Duration};

fn calculate_stake_warmup(mut stake_entry: StakeHistoryEntry, stake_config: &StakeConfig) -> u64 {
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

fn stake_history_entry(epoch: u64, rpc_client: &RpcClient) -> Option<StakeHistoryEntry> {
    let stake_history_account = rpc_client.get_account(&stake_history::id()).ok()?;
    let stake_history = StakeHistory::from_account(&stake_history_account)?;
    stake_history.get(&epoch).cloned()
}

pub fn wait_for_activation(
    activation_epoch: u64,
    mut epoch_info: RpcEpochInfo,
    rpc_client: &RpcClient,
    stake_config: &StakeConfig,
    genesis_block: &GenesisBlock,
    slack_logger: &slack::Logger,
) {
    // Sleep until activation_epoch has finished
    let mut current_epoch = epoch_info.epoch;
    let sleep_epochs = (activation_epoch + 1).saturating_sub(current_epoch);
    let slots_per_epoch = genesis_block.epoch_schedule.slots_per_epoch;
    if sleep_epochs > 0 {
        let sleep_slots = sleep_epochs * slots_per_epoch - epoch_info.slot_index;
        slack_logger.info(&format!(
            "Waiting until activation epoch ({}) is finished...",
            activation_epoch
        ));
        utils::sleep_n_slots(sleep_slots, genesis_block);
    }

    loop {
        epoch_info = rpc_client.get_epoch_info().unwrap_or_else(|err| {
            utils::bail(
                slack_logger,
                &format!("Error: get_epoch_info RPC call failed: {}", err),
            );
        });
        let slot = rpc_client.get_slot().unwrap_or_else(|err| {
            utils::bail(
                slack_logger,
                &format!("Error: get_slot RPC call 3 failed: {}", err),
            );
        });
        info!("Current slot is {}", slot);

        current_epoch = epoch_info.epoch - 1;
        debug!(
            "Fetching stake history entry for epoch: {}...",
            current_epoch
        );

        if let Some(stake_entry) = stake_history_entry(current_epoch, &rpc_client) {
            debug!("Stake history entry: {:?}", &stake_entry);
            let warm_up_epochs = calculate_stake_warmup(stake_entry, stake_config);
            if warm_up_epochs > 0 {
                slack_logger.info(&format!(
                    "Waiting until epoch {} for stake to warmup (current epoch is {})...",
                    current_epoch + warm_up_epochs,
                    current_epoch
                ));
                let sleep_slots = epoch_info.slots_in_epoch - epoch_info.slot_index;
                utils::sleep_n_slots(sleep_slots, genesis_block);
            } else {
                break;
            }
        } else {
            warn!(
                "Failed to fetch stake history entry for epoch: {}",
                current_epoch
            );
            sleep(Duration::from_secs(5));
        }

        let latest_slot = rpc_client.get_slot().unwrap_or_else(|err| {
            utils::bail(
                slack_logger,
                &format!("Error: get_slot RPC call 3 failed: {}", err),
            );
        });
        if slot == latest_slot {
            warn!("Slot {} did not advance, cluster may be stuck", slot);
            break;
        }
    }
}
