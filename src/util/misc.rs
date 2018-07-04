use std::time::{SystemTime, UNIX_EPOCH};

/// Convert byte slice to hex string
pub fn bytes_as_string(bytes: &[u8]) -> String {
    let string_reps: Vec<String> = bytes.into_iter().map(|b| format!("{:02X}", b)).collect();
    string_reps.join("")
}

/// Get
pub fn epoch_time() -> u64 {
    let start = SystemTime::now();
    let epoch_duration = start.duration_since(UNIX_EPOCH).expect("Negative time delta");
    let epoch_ms = epoch_duration.as_secs() * 1000 +
        epoch_duration.subsec_nanos() as u64 / 1_000_000;

    epoch_ms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_bytes_as_string() {
        assert_eq!("000102030405060708090A0B0C0D0E0F",
            bytes_as_string(&[
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f
            ]));
        assert_eq!("00", bytes_as_string(&[ 0x00 ]));
    }

    #[test]
    pub fn test_epoch_time() {
        use std::time::Duration;
        use std::thread;

        let epoch_timestamp = epoch_time();
        assert!(epoch_timestamp > 0);
        thread::sleep(Duration::from_millis(1));
        let next_epoch_timestamp = epoch_time();
        assert!((next_epoch_timestamp - epoch_timestamp) >= 1);
    }
}