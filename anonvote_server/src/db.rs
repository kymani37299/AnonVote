use std::collections::{HashSet, HashMap};
use std::sync::Mutex;

use num_bigint::BigUint;

use crate::model::user_data::UserData;

pub struct ChallengeData {
    pub user_hash : u64,
    pub challenge : BigUint,
    pub ka : BigUint,
    pub kb : BigUint,
}

#[derive(Default)]
pub struct AnonVoteDB {
    active_registration_codes : Mutex<HashSet<String>>,
    registered_users : Mutex<HashMap<u64, UserData>>,
    registered_ids : Mutex<HashSet<String>>,
    pending_votes : Mutex<HashMap<u64, u32>>,
    votes : Mutex<HashMap<u64, u32>>, // TODO: Maybe we don't want to link pub key with vote ?
    challenge_map : Mutex<HashMap<String, ChallengeData>>,
}

impl AnonVoteDB {
    pub fn connect() -> AnonVoteDB {
        AnonVoteDB::default()
    }

    pub fn add_registered_id(&self, id : String) -> bool {
        let reg_id_set =  &mut self.registered_ids.lock().unwrap();
        reg_id_set.insert(id)
    }

    pub fn add_registration_code(&self, code : String) -> bool {
        // If everything is ok add code
        let reg_code_set = &mut self.active_registration_codes.lock().unwrap();
        reg_code_set.insert(code)
    }

    pub fn use_registration_code(&self, code : &String) -> bool {
        let reg_code_set = &mut self.active_registration_codes.lock().unwrap();
        reg_code_set.remove(code)
    }

    pub fn try_register_user(&self, user : UserData) -> bool {
        let reg_users_map =  &mut self.registered_users.lock().unwrap();
        if reg_users_map.contains_key(&user.get_hash()) {
            return false;
        }
        reg_users_map.insert(user.get_hash(), user);
        return true;
    }

    pub fn user_registered(&self, user_hash : u64) -> bool {
        let reg_users_map =  &mut self.registered_users.lock().unwrap();
        reg_users_map.contains_key(&user_hash)
    }

    pub fn add_pending_vote(&self, user_hash : u64, vote : u32) -> bool {
        let pending_votes_map = &mut self.pending_votes.lock().unwrap();
        if pending_votes_map.contains_key(&user_hash) {
            return false;
        }
        pending_votes_map.insert(user_hash, vote);
        return true;
    }

    pub fn add_vote(&self, user_hash : u64, vote : u32) -> bool {
        let votes_map = &mut self.votes.lock().unwrap();
        if votes_map.contains_key(&user_hash) {
            return false;
        }
        votes_map.insert(user_hash, vote);
        return true;
    }

    pub fn add_challenge(&self, session_id : String, challenge_data : ChallengeData) -> bool {
        let challenges = &mut self.challenge_map.lock().unwrap();
        if challenges.contains_key(&session_id) {
            return false;
        }
        challenges.insert(session_id, challenge_data);
        return true;
    }

    pub fn user_voted(&self, user_hash : u64) -> bool {
        let votes_map = &mut self.votes.lock().unwrap();
        votes_map.contains_key(&user_hash)
    }
}