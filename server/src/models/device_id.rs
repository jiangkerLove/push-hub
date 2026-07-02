use sha2::{Digest, Sha256};

/// 由平台 + push_token 确定性生成设备 ID，同一组合始终得到相同 ID。
pub fn assign_device_id(platform: &str, push_token: &str) -> String {
    let seed = format!("{}:{}", platform.trim().to_lowercase(), push_token.trim());
    uuid_from_sha256(&seed)
}

fn uuid_from_sha256(seed: &str) -> String {
    let digest = Sha256::digest(seed.as_bytes());
    let mut bytes = digest[..16].to_vec();
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let msb = i64::from_be_bytes(bytes[0..8].try_into().unwrap());
    let lsb = i64::from_be_bytes(bytes[8..16].try_into().unwrap());
    uuid::Uuid::from_u64_pair(msb as u64, lsb as u64).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_platform_token_same_id() {
        let a = assign_device_id("xiaomi", "regid-123");
        let b = assign_device_id("xiaomi", "regid-123");
        assert_eq!(a, b);
    }

    #[test]
    fn different_platform_different_id() {
        let a = assign_device_id("xiaomi", "token-1");
        let b = assign_device_id("online", "token-1");
        assert_ne!(a, b);
    }
}
