use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("SystemTime")
        .as_nanos()
}
