use std::collections::{HashSet, HashMap};
use std::sync::Mutex;

use crate::model::user_data::UserData;

#[derive(Default)]
pub struct AnonVoteDB {
    active_registration_codes : Mutex<HashSet<String>>,
    registered_users : Mutex<HashMap<u64, UserData>>,
    registered_ids : Mutex<HashSet<String>>
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
}