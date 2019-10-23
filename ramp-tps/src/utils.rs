use bzip2::bufread::BzDecoder;
use log::info;
use solana_netutil::parse_host;
use solana_sdk::genesis_block::GenesisBlock;
use solana_sdk::timing::duration_as_ms;
use std::fs::File;
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use tar::Archive;

const GENESIS_ARCHIVE_NAME: &str = "genesis.tar.bz2";

/// Inspired by solana_local_cluster::cluster_tests
fn slots_to_secs(num_slots: u64, genesis_block: &GenesisBlock) -> u64 {
    let poh_config = &genesis_block.poh_config;
    let ticks_per_slot = genesis_block.ticks_per_slot;
    let num_ticks_to_sleep = num_slots as f64 * ticks_per_slot as f64;
    let num_ticks_per_second = (1000 / duration_as_ms(&poh_config.target_tick_duration)) as f64;
    ((num_ticks_to_sleep + num_ticks_per_second - 1.0) / num_ticks_per_second) as u64
}

pub fn sleep_n_slots(num_slots: u64, genesis_block: &GenesisBlock) {
    let secs = slots_to_secs(num_slots, genesis_block);
    let mins = secs / 60;
    let hours = mins / 60;
    if hours >= 5 {
        info!("Sleeping for {} slots ({} hours)", num_slots, hours);
    } else if mins >= 5 {
        info!("Sleeping for {} slots ({} minutes)", num_slots, mins);
    } else if secs > 0 {
        info!("Sleeping for {} slots ({} seconds)", num_slots, secs);
    }
    sleep(Duration::from_secs(secs));
}

pub fn is_host(string: String) -> Result<(), String> {
    parse_host(&string)?;
    Ok(())
}

/// Inspired by solana_validator::download_tar_bz2
pub fn download_genesis(rpc_addr: &SocketAddr, download_path: &Path) -> Result<(), String> {
    let archive_name = GENESIS_ARCHIVE_NAME;
    let archive_path = download_path.join(archive_name);
    let url = format!("http://{}/{}", rpc_addr, archive_name);
    let download_start = Instant::now();
    info!("Downloading genesis ({})...", url);

    let client = reqwest::Client::new();
    let mut response = client
        .get(url.as_str())
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| format!("Unable to get: {:?}", err))?;
    let download_size = {
        response
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|content_length| content_length.to_str().ok())
            .and_then(|content_length| content_length.parse().ok())
            .unwrap_or(0)
    };

    let mut file = File::create(&archive_path)
        .map_err(|err| format!("Unable to create {:?}: {:?}", archive_path, err))?;
    io::copy(&mut response, &mut file)
        .map_err(|err| format!("Unable to write {:?}: {:?}", archive_path, err))?;

    info!(
        "Downloaded genesis ({} bytes) in {:?}",
        download_size,
        Instant::now().duration_since(download_start),
    );

    info!("Extracting genesis ({:?})...", archive_name);
    let extract_start = Instant::now();
    let tar_bz2 = File::open(&archive_path)
        .map_err(|err| format!("Unable to open {}: {:?}", archive_name, err))?;
    let tar = BzDecoder::new(io::BufReader::new(tar_bz2));
    let mut archive = Archive::new(tar);
    archive
        .unpack(download_path)
        .map_err(|err| format!("Unable to unpack {}: {:?}", archive_name, err))?;
    info!(
        "Extracted {} in {:?}",
        archive_name,
        Instant::now().duration_since(extract_start)
    );

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slots_to_secs() {
        let mut genesis_block = GenesisBlock::default();
        genesis_block.poh_config.target_tick_duration = Duration::from_millis(500);

        genesis_block.ticks_per_slot = 10;
        assert_eq!(slots_to_secs(2, &genesis_block), 10);

        genesis_block.ticks_per_slot = 1;
        assert_eq!(slots_to_secs(1, &genesis_block), 1);

        genesis_block.ticks_per_slot = 0;
        assert_eq!(slots_to_secs(10, &genesis_block), 0);
    }
}
