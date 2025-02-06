use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use zkp_protocol::PublicKey;

#[derive(Hash, Clone)]
pub struct UserData {
    pub key : PublicKey,
}

impl UserData {
    pub fn get_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod test {
    use num_bigint::BigUint;

    use super::*;

    #[test]
    fn test_hash() {

        let data1 = PublicKey::new(BigUint::from(1u32), BigUint::from(2u32), BigUint::from(3u32),BigUint::from(4u32)); 
        let data2 = PublicKey::new(BigUint::from(1u32), BigUint::from(2u32), BigUint::from(3u32),BigUint::from(4u32)); 

        let data1 = UserData{ key: data1};
        let data2 = UserData{ key: data2};

        let hash1 = data1.get_hash();
        let hash2 = data2.get_hash();

        assert_eq!(hash1, hash2, "Hashes should be the same for equal UserData instances.");
    }
}