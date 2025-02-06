use num_bigint::BigUint;

#[derive(Clone)]
pub struct ChallengeData {
    pub user_hash : u64,
    pub challenge : BigUint,
    pub ka : BigUint,
    pub kb : BigUint,
}