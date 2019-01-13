use std::time::{SystemTime, UNIX_EPOCH};

/// Convert u64 to hex string
pub fn u64_as_hex_string(val: u64) -> String {
    format!("{:016x}", val)
}

/// Convert u32 to hex string
pub fn u32_as_hex_string(val: u32) -> String {
    format!("{:08x}", val)
}

/// Get time since epoch
pub fn epoch_time() -> u64 {
    let start = SystemTime::now();
    let epoch_duration = start.duration_since(UNIX_EPOCH).expect("Negative time delta");
    epoch_duration.as_secs() * 1000 +
        u64::from(epoch_duration.subsec_nanos()) / 1_000_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_as_hex_string() {
        assert_eq!(u64_as_hex_string(0), "0000000000000000");
        assert_eq!(u64_as_hex_string(16), "0000000000000010");
        assert_eq!(u64_as_hex_string(6043537212972274484), "53def5133e111334");
        assert_eq!(u64_as_hex_string(18437817136469695293), "ffe048ff74e2db3d");
        assert_eq!(u64_as_hex_string(18446744073709551615), "ffffffffffffffff");
    }

    #[test]
    fn test_u32_as_hex_string() {
        assert_eq!(u32_as_hex_string(0), "00000000");
        assert_eq!(u32_as_hex_string(16), "00000010");
        assert_eq!(u32_as_hex_string(2568797931), "991cbeeb");
        assert_eq!(u32_as_hex_string(4294967295), "ffffffff");
    }

    #[test]
    fn test_epoch_time() {
        use std::time::Duration;
        use std::thread;

        let epoch_timestamp = epoch_time();
        assert!(epoch_timestamp > 0);
        thread::sleep(Duration::from_millis(1));
        let next_epoch_timestamp = epoch_time();
        assert!((next_epoch_timestamp - epoch_timestamp) >= 1);
    }
}