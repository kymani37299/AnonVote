use crate::db::AnonVoteDB;
use crate::model::user_data::UserData;

use anonvote_proto::proto::anonvote::anon_vote_server::AnonVote;
use anonvote_proto::proto::anonvote::{ValidateIdReq, ValidateIdRes, RegisterReq, RegisterRes, VoteReq, VoteRes, ValidateVoteReq, ValidateVoteRes};

use tonic::{Request, Response, Status, Code };
use rand::distr::{Alphanumeric, SampleString};

const REGISTRATION_KEY_LEN : usize = 16;

pub struct AnonVoteImpl {
    db : AnonVoteDB, 
}

impl AnonVoteImpl {
    pub fn new(db : AnonVoteDB) -> AnonVoteImpl {
        AnonVoteImpl {
            db
        }
    }
}

impl AnonVoteImpl {
    fn generate_random_string(len : usize) -> String {
        let rnd = &mut rand::rng();
        Alphanumeric.sample_string(rnd, len)
    }
    
    fn validate_id(_id : &String) -> bool {
        // This is where the we will call goverment apis to validate identity of the person
        // Since this is simulation we will assume rule:
        // 1. Valid id is exactly 5 chars in length
        _id.len() == 5
    }

    fn validate_user_data(user : &UserData) -> bool {
        // TODO: Make more detailed validation of public key
        user.a.len() > 0 && user.b.len() > 0 && user.alpha.len() > 0 &&user.beta.len() > 0
    }
}

#[tonic::async_trait]
impl AnonVote for AnonVoteImpl {
    async fn validate_id(&self, req : Request<ValidateIdReq>) -> Result<Response<ValidateIdRes>, Status> {
        let req = req.into_inner();

        // Check id validity
        let valid_id = AnonVoteImpl::validate_id(&req.id);
        if !valid_id {
            return Err(Status::new(Code::InvalidArgument, "User identification failed!"));
        }

        // Add ID to the registered list, while also checking if the id is already registered
        let added = self.db.add_registered_id(req.id);
        if !added {
            return Err(Status::new(Code::AlreadyExists, "This ID already generated code!"));
        }

        // Generate registration key
        let mut registration_key : String;
        loop { // We are looping just in case that generated registration_key already exists
            registration_key = AnonVoteImpl::generate_random_string(REGISTRATION_KEY_LEN);
            let added = self.db.add_registration_code(registration_key.clone());
            if added {
                break;
            }
        }
        Ok(Response::new(ValidateIdRes{registration_key}))
    }

    async fn register(&self, req : Request<RegisterReq>) -> Result<Response<RegisterRes>, Status> {
        // TODO: Sort the registration code multiple locks, maybe there is some vurneability
        // Since we are removing and re-adding the code in some cases

        let req = req.into_inner();
        let user_data = UserData {
            a : req.a,
            b : req.b,
            alpha : req.alpha,
            beta : req.beta
        };
        let user_data_valid = AnonVoteImpl::validate_user_data(&user_data);
        if !user_data_valid {
            return Err(Status::new(Code::InvalidArgument, "Invalid user data!"));
        }

        let valid_key = self.db.use_registration_code(&req.registration_key);
        if !valid_key {
            return Err(Status::new(Code::InvalidArgument, "Invalid registration key!"));
        }

        let succ = self.db.try_register_user(user_data);
        if !succ {
            self.db.add_registration_code(req.registration_key); // Add back registartion code since it was not used
            return Err(Status::new(Code::AlreadyExists, "User with this public key already exists! Please try again."));
        }
        Ok(Response::new(RegisterRes{}))
    }

    async fn vote(&self, _req : Request<VoteReq>) -> Result<Response<VoteRes>, Status> {
        todo!()
    }

    async fn validate_vote(&self, _req : Request<ValidateVoteReq>) -> Result<Response<ValidateVoteRes>, Status> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use zkp_protocol::SecretKey;
    use num_bigint::BigUint;

    #[tokio::test]
    async fn test_validate_id() {
        let db = AnonVoteDB::connect();
        let server_impl = AnonVoteImpl::new(db);

        let validate_req = Request::new(ValidateIdReq {
            id : String::from("123456")
        });
        let validate_res = server_impl.validate_id(validate_req).await;
        assert!(validate_res.is_err(), "Did not throw error while passing invalid ID");

        let validate_req = Request::new(ValidateIdReq {
            id : String::from("12345")
        });
        let validate_res = server_impl.validate_id(validate_req).await;
        assert!(validate_res.is_ok(), "Did throw error while passing valid ID");

        let validate_req = Request::new(ValidateIdReq {
            id : String::from("12345")
        });
        let validate_res = server_impl.validate_id(validate_req).await;
        assert!(validate_res.is_err(), "Did not throw error while passing same ID twice");
    }

    #[tokio::test]
    async fn test_register() {
        let db = AnonVoteDB::connect();
        let server_impl = AnonVoteImpl::new(db);

        let validate_req = Request::new(ValidateIdReq {
            id : String::from("12345")
        });
        let validate_res = server_impl.validate_id(validate_req).await;
        assert!(validate_res.is_ok(), "Did throw error while passing valid ID");

        let registration_key = validate_res.unwrap().into_inner().registration_key;

        let secret_key = SecretKey::new(BigUint::from(123456789u32));
        let public_key = secret_key.generate_public_key();

        let secret_key2 = SecretKey::new(BigUint::from(742524531u32));
        let public_key2 = secret_key2.generate_public_key();

        let register_req = Request::new(RegisterReq {
            registration_key : registration_key.clone(),
            a : public_key.a().to_bytes_be(),
            b : public_key.b().to_bytes_be(),
            alpha : public_key.alpha().to_bytes_be(),
            beta : vec!()
        });

        let register_res = server_impl.register(register_req).await;
        assert!(register_res.is_err(), "Did not throw error while passing invalid public key");

        let register_req = Request::new(RegisterReq {
            registration_key : String::from("asdsafsafad"),
            a : public_key.a().to_bytes_be(),
            b : public_key.b().to_bytes_be(),
            alpha : public_key.alpha().to_bytes_be(),
            beta : public_key.beta().to_bytes_be(),
        });

        let register_res = server_impl.register(register_req).await;
        assert!(register_res.is_err(), "Did not throw error while passing invalid registration key");

        let register_req = Request::new(RegisterReq {
            registration_key : registration_key.clone(),
            a : public_key.a().to_bytes_be(),
            b : public_key.b().to_bytes_be(),
            alpha : public_key.alpha().to_bytes_be(),
            beta : public_key.beta().to_bytes_be(),
        });

        let register_res = server_impl.register(register_req).await;
        assert!(register_res.is_ok(), "Did throw error while passing valid registration");

        let register_req = Request::new(RegisterReq {
            registration_key : registration_key.clone(),
            a : public_key2.a().to_bytes_be(),
            b : public_key2.b().to_bytes_be(),
            alpha : public_key2.alpha().to_bytes_be(),
            beta : public_key2.beta().to_bytes_be(),
        });

        let register_res = server_impl.register(register_req).await;
        assert!(register_res.is_err(), "Did not throw error while trying to register twice");

        let validate_req = Request::new(ValidateIdReq {
            id : String::from("64532")
        });
        let validate_res = server_impl.validate_id(validate_req).await;
        assert!(validate_res.is_ok(), "Did throw error while passing valid ID");

        let registration_key = validate_res.unwrap().into_inner().registration_key;

        let register_req = Request::new(RegisterReq {
            registration_key : registration_key.clone(),
            a : public_key.a().to_bytes_be(),
            b : public_key.b().to_bytes_be(),
            alpha : public_key.alpha().to_bytes_be(),
            beta : public_key.beta().to_bytes_be(),
        });

        let register_res = server_impl.register(register_req).await;
        assert!(register_res.is_err(), "Did not throw error while trying to register with same public key twice");
    }
}