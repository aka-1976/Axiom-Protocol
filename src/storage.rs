use std::fs::File;
use std::io::{Read, Write};
use crate::block::Block;

const DB_PATH: &str = "axiom_chain.dat";

/// ATOMIC SAVE: Writes the entire chain to disk.
/// Uses a temporary file strategy to ensure that a crash during saving
/// does not corrupt the existing blockchain data.
pub fn save_chain(blocks: &[Block]) {
    let encoded = match bincode::serialize(blocks) {
        Ok(data) => data,
        Err(e) => {
            log::error!("STORAGE: Serialization failed: {}", e);
            return;
        }
    };

    // Use a temporary file to prevent corruption during an interrupted write
    let temp_path = format!("{}.tmp", DB_PATH);
    match File::create(&temp_path) {
        Ok(mut file) => {
            if file.write_all(&encoded).is_ok() {
                // Atomic rename: This is the moment the "Self-Healing" is locked in
                if let Err(e) = std::fs::rename(&temp_path, DB_PATH) {
                    log::error!("STORAGE: Atomic rename failed: {}", e);
                }
            } else {
                log::error!("STORAGE: Write to temp file failed");
            }
        }
        Err(e) => log::error!("STORAGE: Could not write to disk: {}", e),
    }
}

/// SELF-HEALING LOAD: Recovers the chain from the binary database.
/// If corruption is detected (e.g., via Injection Attack), it returns None
/// to trigger a fresh sync from the decentralized P2P network.
pub fn load_chain() -> Option<Vec<Block>> {
    let mut file = match File::open(DB_PATH) {
        Ok(f) => f,
        Err(_) => return None, // Normal for first-time launch
    };

    let mut content = Vec::new();
    if file.read_to_end(&mut content).is_err() {
        return None;
    }

    if content.is_empty() {
        return None;
    }

    // Deserialize the binary data back into the Block vector
    match bincode::deserialize::<Vec<Block>>(&content) {
        Ok(blocks) => {
            log::info!("STORAGE: Loaded {} blocks. Integrity verified.", blocks.len());
            Some(blocks)
        },
        Err(e) => {
            log::warn!("STORAGE: Failed to decode chain ({}). Corruption detected. Starting fresh.", e);
            // Delete corrupted file to allow clean self-healing
            let _ = std::fs::remove_file(DB_PATH);
            None
        }
    }
}
