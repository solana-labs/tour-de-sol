use crate::{notifier, utils};
use log::*;
use solana_client::{rpc_client::RpcClient, rpc_request::RpcEpochInfo};
use solana_sdk::{
    genesis_config::GenesisConfig,
    sysvar::{
        stake_history::{self, StakeHistory, StakeHistoryEntry},
        Sysvar,
    },
};
use solana_stake_program::config::Config as StakeConfig;
use std::{thread::sleep, time::Duration};

fn calculate_stake_warmup(mut stake_entry: StakeHistoryEntry, stake_config: &StakeConfig) -> u64 {
    let mut epochs = 0;
    loop {
        let percent_warming_up =
            stake_entry.activating as f64 / stake_entry.effective.max(1) as f64;
        let percent_cooling_down =
            stake_entry.deactivating as f64 / stake_entry.effective.max(1) as f64;
        debug!(
            "epoch +{}: stake warming up {:.1}%, cooling down {:.1}% ",
            epochs,
            percent_warming_up * 100.,
            percent_cooling_down * 100.
        );

        if (percent_warming_up < 0.05) && (percent_cooling_down < 0.05) {
            break;
        }
        let warmup_cooldown_rate = stake_config.warmup_cooldown_rate;
        let max_warmup_stake = (stake_entry.effective as f64 * warmup_cooldown_rate) as u64;
        let warmup_stake = stake_entry.activating.min(max_warmup_stake);
        stake_entry.effective += warmup_stake;
        stake_entry.activating -= warmup_stake;

        let max_cooldown_stake = (stake_entry.effective as f64 * warmup_cooldown_rate) as u64;
        let cooldown_stake = stake_entry.deactivating.min(max_cooldown_stake);
        stake_entry.effective -= cooldown_stake;
        stake_entry.deactivating -= cooldown_stake;
        debug!(
            "epoch +{}: stake warming up {}, cooling down {}",
            epochs, warmup_stake, cooldown_stake
        );

        epochs += 1;
    }
    info!("95% stake warmup will take {} epochs", epochs);
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
    genesis_config: &GenesisConfig,
    notifier: &notifier::Notifier,
) {
    // Sleep until activation_epoch has finished
    let mut current_epoch = epoch_info.epoch;
    let sleep_epochs = (activation_epoch + 1).saturating_sub(current_epoch);
    let slots_per_epoch = genesis_config.epoch_schedule.slots_per_epoch;
    if sleep_epochs > 0 {
        let sleep_slots = sleep_epochs * slots_per_epoch - epoch_info.slot_index;
        notifier.notify(&format!(
            "Waiting until epoch {} is finished...",
            activation_epoch
        ));
        utils::sleep_n_slots(sleep_slots, genesis_config);
    }

    loop {
        epoch_info = rpc_client.get_epoch_info().unwrap_or_else(|err| {
            utils::bail(
                notifier,
                &format!("Error: get_epoch_info RPC call failed: {}", err),
            );
        });
        let slot = rpc_client.get_slot().unwrap_or_else(|err| {
            utils::bail(
                notifier,
                &format!("Error: get_slot RPC call 3 failed: {}", err),
            );
        });
        info!("Current slot is {}", slot);

        current_epoch = epoch_info.epoch;
        let latest_epoch = current_epoch - 1;
        debug!(
            "Fetching stake history entry for epoch: {}...",
            latest_epoch
        );

        if let Some(stake_entry) = stake_history_entry(latest_epoch, &rpc_client) {
            debug!("Stake history entry: {:?}", &stake_entry);
            let warm_up_epochs = calculate_stake_warmup(stake_entry, stake_config);
            let stake_warmed_up_epoch = latest_epoch + warm_up_epochs;
            if stake_warmed_up_epoch > current_epoch {
                notifier.notify(&format!(
                    "Waiting until epoch {} for stake to warmup (current epoch is {})...",
                    stake_warmed_up_epoch, current_epoch
                ));
                let sleep_slots = epoch_info.slots_in_epoch - epoch_info.slot_index;
                utils::sleep_n_slots(sleep_slots, genesis_config);
            } else {
                break;
            }
        } else {
            warn!(
                "Failed to fetch stake history entry for epoch: {}",
                latest_epoch
            );
            sleep(Duration::from_secs(5));
        }

        let latest_slot = rpc_client.get_slot().unwrap_or_else(|err| {
            utils::bail(
                notifier,
                &format!("Error: get_slot RPC call 4 failed: {}", err),
            );
        });
        if slot == latest_slot {
            utils::bail(notifier, &format!("Slot did not advance from {}", slot));
        }
    }
}
