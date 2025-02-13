use std::collections::{HashSet, HashMap};
use std::sync::Mutex;
use std::vec::Vec;

use crate::model::user_data::UserData;
use crate::model::challenge_data::ChallengeData;

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

    pub fn get_user(&self, user_hash : u64) -> Option<UserData> {
        let reg_users_map =  &mut self.registered_users.lock().unwrap();
        reg_users_map.get(&user_hash).cloned()
    }

    pub fn add_pending_vote(&self, user_hash : u64, vote : u32) -> bool {
        let pending_votes_map = &mut self.pending_votes.lock().unwrap();
        if pending_votes_map.contains_key(&user_hash) {
            return false;
        }
        pending_votes_map.insert(user_hash, vote);
        return true;
    }

    pub fn get_pending_vote(&self, user_hash : u64) -> Option<u32> {
        let pending_votes_map = &mut self.pending_votes.lock().unwrap();
        pending_votes_map.get(&user_hash).cloned()
    }

    pub fn get_and_remove_pending_vote(&self, user_hash : u64) -> Option<u32> {
        let pending_votes_map = &mut self.pending_votes.lock().unwrap();
        pending_votes_map.remove(&user_hash)
    }

    pub fn add_vote(&self, user_hash : u64, vote : u32) -> bool {
        let votes_map = &mut self.votes.lock().unwrap();
        if votes_map.contains_key(&user_hash) {
            return false;
        }
        votes_map.insert(user_hash, vote);
        return true;
    }

    pub fn add_challenge(&self, session_id : &String, challenge_data : ChallengeData) -> bool {
        let challenges = &mut self.challenge_map.lock().unwrap();
        if challenges.contains_key(session_id) {
            return false;
        }
        challenges.insert(session_id.clone(), challenge_data);
        return true;
    }

    pub fn get_challenge(&self, session_id : &String) -> Option<ChallengeData> {
        let challenges = &mut self.challenge_map.lock().unwrap();
        challenges.get(session_id).cloned()
    }

    pub fn remove_challenge(&self, session_id : &String) -> bool {
        let challenges = &mut self.challenge_map.lock().unwrap();
        challenges.remove(session_id).is_some()
    }

    pub fn user_voted(&self, user_hash : u64) -> bool {
        let votes_map = &mut self.votes.lock().unwrap();
        votes_map.contains_key(&user_hash)
    }

    pub fn get_vote_options(&self) -> Vec<String> {
        // Placeholder values
        let vote_options = vec![
        "First vote - Always the first!".to_string(), 
        "Mr. Placeholder".to_string(), 
        "Final Choice – The last name you'll pick!".to_string()
        ];
        vote_options
    }

    pub fn get_vote_options_count(&self) -> usize {
        self.get_vote_options().len()
    }

    pub fn get_vote_results(&self) -> Vec<u32> {
        let votes_map = &mut self.votes.lock().unwrap();
        let vote_option_count = self.get_vote_options_count();
        let mut votes = vec![0u32; vote_option_count];
        for vote in votes_map.values() {
            let vote = *vote as usize;
            if vote >= vote_option_count {
                continue;
            }
            votes[vote] += 1;
        }
        votes
    }
}