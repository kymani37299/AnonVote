use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Hash)]
pub struct UserData {
    pub a : Vec<u8>,
    pub b : Vec<u8>,
    pub alpha : Vec<u8>,
    pub beta : Vec<u8>,
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
    use super::*;

    #[test]
    fn test_hash() {
        let data1 = UserData{
            a : vec!(1,2,3),
            b: vec!(4,5),
            alpha: vec!(6),
            beta: vec!(7,8),
        };

        let data2 = UserData{
            a : vec!(1,2,3),
            b: vec!(4,5),
            alpha: vec!(6),
            beta: vec!(7,8),
        };

        let hash1 = data1.get_hash();
        let hash2 = data2.get_hash();

        assert_eq!(hash1, hash2, "Hashes should be the same for equal UserData instances.");
    }
}