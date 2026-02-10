pub fn current_slot() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Safe unwrap: system time should always be after UNIX_EPOCH
    // If this fails, return 0 as a fallback (indicates beginning of time)
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 15)
        .unwrap_or_else(|e| {
            log::warn!("Failed to get current slot time: {}", e);
            0
        })
}
