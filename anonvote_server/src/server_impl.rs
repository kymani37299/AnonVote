use crate::db::AnonVoteDB;
use crate::model::user_data::UserData;
use crate::model::challenge_data::ChallengeData;

use anonvote_proto::proto::anonvote::anon_vote_server::AnonVote;
use anonvote_proto::proto::anonvote::{ValidateIdReq, ValidateIdRes, RegisterReq, RegisterRes, VoteReq, VoteRes, ValidateVoteReq, ValidateVoteRes};

use num_bigint::BigUint;
use tonic::{Request, Response, Status, Code };
use rand::distr::{Alphanumeric, SampleString};
use zkp_protocol::{zkp_util, PublicKey};

const REGISTRATION_KEY_LEN : usize = 16;
const AUTH_KEY_LEN : usize = 16;

pub struct AnonVoteImpl {
    db : AnonVoteDB, 
    vote_option_count : u32 // Valid votes are {1, 2, ... , vote_option_count}
}

impl AnonVoteImpl {
    pub fn new(db : AnonVoteDB, vote_option_count : u32) -> AnonVoteImpl {
        AnonVoteImpl {
            db,
            vote_option_count
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
        *user.key.a() > BigUint::ZERO && *user.key.b() > BigUint::ZERO && *user.key.beta() > BigUint::ZERO   
    }

    fn vote_valid(&self, vote : &u32) -> bool {
        *vote > 0 && *vote <= self.vote_option_count
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
        let public_key = PublicKey::from_bytes_be(&req.a, &req.b, &req.alpha, &req.beta);
        let user_data = UserData { key : public_key };
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

    async fn vote(&self, req : Request<VoteReq>) -> Result<Response<VoteRes>, Status> {
        let req= req.into_inner();

        // Get user hash
        let public_key = PublicKey::from_bytes_be(&req.a, &req.b, &req.alpha, &req.beta);
        let user_data = UserData { key : public_key };
        let user_hash = user_data.get_hash();

        // Check if user data is valid
        if !AnonVoteImpl::validate_user_data(&user_data) {
            return Err(Status::new(Code::InvalidArgument, "Invalid user data!"));
        }

        // Check if vote is valid
        if !self.vote_valid(&req.vote) {
            return Err(Status::new(Code::InvalidArgument, "Invalid vote!"));
        }
        
        // Check if user is registered
        if !self.db.user_registered(user_hash) {
            return Err(Status::new(Code::InvalidArgument, "User not registered!"));
        }

        // Check if user has already voted
        if self.db.user_voted(user_hash) {
            return Err(Status::new(Code::AlreadyExists, "User already voted!"));
        }

        // Try to ddd vote to pending votes
        let vote_added = self.db.add_pending_vote(user_hash, req.vote);
        if !vote_added {
            return Err(Status::new(Code::AlreadyExists, "This user aleady has pending vote!"));
        }

        // Generate challenge
        let c = zkp_util::generate_challenge();
        let c_bytes = c.to_bytes_be();
        let challenge = ChallengeData {
            user_hash,
            ka : BigUint::from_bytes_be(&req.ka),
            kb : BigUint::from_bytes_be(&req.kb),
            challenge : c
        };

        // Generate session_id
        let session_id = AnonVoteImpl::generate_random_string(AUTH_KEY_LEN);
        let added = self.db.add_challenge(&session_id.clone(), challenge);

        // Edge case - if there is already same session id in the db give internal error to try again
        // TODO: Handle case where we generated same session id 
        if !added {
            return Err(Status::new(Code::Internal, "Internal error, please try again!"));
        }

        Ok(Response::new(VoteRes{
            auth_session_id : session_id,
            challenge : c_bytes
        }))
    }

    async fn validate_vote(&self, req : Request<ValidateVoteReq>) -> Result<Response<ValidateVoteRes>, Status> {
        let req = req.into_inner();
        let challenge_data = self.db.get_challenge(&req.auth_session_id);
        let challenge_data = challenge_data.ok_or(Status::new(Code::InvalidArgument, "Invalid session id!"))?;
        
        let pending_vote = self.db.get_pending_vote(challenge_data.user_hash);
        let pending_vote = pending_vote.ok_or(Status::new(Code::InvalidArgument, "The pending vote linked with this session no longer exists!"))?; 
        
        // TODO: Check if the vote in request is even needed, maybe we want to hide the initial vote from the validation part
        if pending_vote != req.vote {
            return Err(Status::new(Code::InvalidArgument, "The pending vote does not match the vote provided!"));
        }

        // TODO: Delete the challenge from db if user_data doesn't exist
        let user_data = self.db.get_user(challenge_data.user_hash);
        let user_data = user_data.ok_or(Status::new(Code::InvalidArgument, "The user linked with this session no longer exists."))?;
        let solution = BigUint::from_bytes_be(&req.solution);

        let verified = user_data.key.verify(&challenge_data.ka, &challenge_data.kb, &challenge_data.challenge, &solution);
        if !verified {
            return Err(Status::new(Code::InvalidArgument, "The solution provided is not verified!"));
        }

        let removed = self.db.remove_challenge(&req.auth_session_id);
        if !removed {
            // We did find the challenge at the beggining, someone else also tried to verify the challenge
            // at the same time. We will mark this as internal error for now.
            return Err(Status::new(Code::Internal, "Internal error E0001!"));
        }

        let pending_vote = self.db.get_and_remove_pending_vote(challenge_data.user_hash);

        // Similar error as E0001
        let pending_vote = pending_vote.ok_or(Status::new(Code::Internal, "Internal error E0002!"))?;

        let added = self.db.add_vote(challenge_data.user_hash, pending_vote);
        if !added {
            // Similar error as E0001
            return Err(Status::new(Code::Internal, "Internal error E0003!"));
        }

        Ok(Response::new(ValidateVoteRes { }))
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
        let server_impl = AnonVoteImpl::new(db, 3);

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
        let server_impl = AnonVoteImpl::new(db, 3);

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

    struct TestUserData(String, SecretKey, PublicKey);

    async fn generate_user(id : &str, secret_key : u32) -> TestUserData {
        let sk = SecretKey::new(BigUint::from(secret_key));
        let pk = sk.generate_public_key();
        TestUserData(String::from(id), sk, pk)
    }

    async fn register_user(server_impl : &AnonVoteImpl, user: &TestUserData) {
        let validate_req = Request::new(ValidateIdReq {
            id : user.0.clone()
        });
        let validate_res = server_impl.validate_id(validate_req).await;
        assert!(validate_res.is_ok(), "Did throw error while passing valid ID");

        let registration_key = validate_res.unwrap().into_inner().registration_key;

        let register_req = Request::new(RegisterReq {
            registration_key : registration_key,
            a : user.2.a().to_bytes_be(),
            b : user.2.b().to_bytes_be(),
            alpha : user.2.alpha().to_bytes_be(),
            beta : user.2.beta().to_bytes_be(),
        });

        let register_res = server_impl.register(register_req).await;
        assert!(register_res.is_ok(), "Did throw error while passing valid registration");
    }

    #[tokio::test]
    async fn test_vote() {
        let db = AnonVoteDB::connect();
        let server_impl = AnonVoteImpl::new(db, 3);

        let user1 = generate_user("12345", 123456789u32).await;
        register_user(&server_impl, &user1).await;

        let (_, ka,kb) = user1.2.generate_challenge_request();
        let vote_req = Request::new(VoteReq {
            vote : 1,
            alpha : user1.2.alpha().to_bytes_be(),
            beta : user1.2.beta().to_bytes_be(),
            a : user1.2.a().to_bytes_be(),
            b : user1.2.b().to_bytes_be(),
            ka : ka.to_bytes_be(),
            kb : kb.to_bytes_be()
        });

        let vote_res = server_impl.vote(vote_req).await;
        assert!(vote_res.is_ok(), "Did throw error while voting correctly. Error: {:?}",vote_res);

        let (_, ka,kb) = user1.2.generate_challenge_request();
        let vote_req = Request::new(VoteReq {
            vote : 0,
            alpha : user1.2.alpha().to_bytes_be(),
            beta : user1.2.beta().to_bytes_be(),
            a : user1.2.a().to_bytes_be(),
            b : user1.2.b().to_bytes_be(),
            ka : ka.to_bytes_be(),
            kb : kb.to_bytes_be()
        });

        let vote_res = server_impl.vote(vote_req).await;
        assert!(vote_res.is_err(), "Did not throw error while voting again");

        let user2 = generate_user("54321", 4135164u32).await;
        let other_secret_key = SecretKey::new(BigUint::from(5315314u32));
        let other_public_key = other_secret_key.generate_public_key();

        register_user(&server_impl, &user2).await;

        let (_, ka,kb) = user2.2.generate_challenge_request();
        let vote_req = Request::new(VoteReq {
            vote : 2,
            alpha : vec!(),
            beta : user2.2.beta().to_bytes_be(),
            a : user2.2.a().to_bytes_be(),
            b : user2.2.b().to_bytes_be(),
            ka : ka.to_bytes_be(),
            kb : kb.to_bytes_be()
        });

        let vote_res = server_impl.vote(vote_req).await;
        assert!(vote_res.is_err(), "Did not throw error while passing invalid user data");

        let (_, ka,kb) = user2.2.generate_challenge_request();
        let vote_req = Request::new(VoteReq {
            vote : 10,
            alpha : user2.2.alpha().to_bytes_be(),
            beta : user2.2.beta().to_bytes_be(),
            a : user2.2.a().to_bytes_be(),
            b : user2.2.b().to_bytes_be(),
            ka : ka.to_bytes_be(),
            kb : kb.to_bytes_be()
        });

        let vote_res = server_impl.vote(vote_req).await;
        assert!(vote_res.is_err(), "Did not throw error while passing invalid vote");

        let (_, ka,kb) = other_public_key.generate_challenge_request();
        let vote_req = Request::new(VoteReq {
            vote : 2,
            alpha : other_public_key.alpha().to_bytes_be(),
            beta : other_public_key.beta().to_bytes_be(),
            a : other_public_key.a().to_bytes_be(),
            b : other_public_key.b().to_bytes_be(),
            ka : ka.to_bytes_be(),
            kb : kb.to_bytes_be()
        });

        let vote_res = server_impl.vote(vote_req).await;
        assert!(vote_res.is_err(), "Did not throw error while passing unregistered user");
    }

    async fn vote(server_impl : &AnonVoteImpl, user : &TestUserData, vote : u32) -> (String, BigUint, BigUint) {
        register_user(&server_impl, user).await;

        let (k, ka,kb) = user.2.generate_challenge_request();
        let vote_req = Request::new(VoteReq {
            vote,
            alpha : user.2.alpha().to_bytes_be(),
            beta : user.2.beta().to_bytes_be(),
            a : user.2.a().to_bytes_be(),
            b : user.2.b().to_bytes_be(),
            ka : ka.to_bytes_be(),
            kb : kb.to_bytes_be()
        });

        let vote_res = server_impl.vote(vote_req).await;
        assert!(vote_res.is_ok(), "Did throw error while voting correctly. Error: {:?}",vote_res);

        let vote_res = vote_res.unwrap().into_inner();
        (vote_res.auth_session_id, k, BigUint::from_bytes_be(&vote_res.challenge))
    }

    #[tokio::test]
    async fn test_verify() {
        let db = AnonVoteDB::connect();
        let server_impl = AnonVoteImpl::new(db, 3);

        let user = generate_user("12345", 12341u32).await;
        let (auth_session_id, k, c) = vote(&server_impl, &user, 1).await;

        let solution = user.1.solve(&k, &c);

        let validate_req = Request::new(ValidateVoteReq {
            auth_session_id : auth_session_id.clone(),
            solution : solution.to_bytes_be(),
            vote : 1
        });

        let validate_res = server_impl.validate_vote(validate_req).await;
        assert!(validate_res.is_ok(), "Did throw error while voting correctly. Error: {:?}",validate_res);

        let validate_req = Request::new(ValidateVoteReq {
            auth_session_id : auth_session_id.clone(),
            solution : solution.to_bytes_be(),
            vote : 1
        });

        let validate_res = server_impl.validate_vote(validate_req).await;
        assert!(validate_res.is_err(), "Did not throw error while trying to validate correctly twice");

        let user = generate_user("13345", 423211u32).await;
        let (auth_session_id, k, c) = vote(&server_impl, &user, 1).await;

        let solution = user.1.solve(&k, &c) - BigUint::from(1u32);

        let validate_req = Request::new(ValidateVoteReq {
            auth_session_id : auth_session_id.clone(),
            solution : solution.to_bytes_be(),
            vote : 1
        });

        let validate_res = server_impl.validate_vote(validate_req).await;
        assert!(validate_res.is_err(), "Did not throw error while not giving correct solution.");

        let validate_req = Request::new(ValidateVoteReq {
            auth_session_id : auth_session_id.clone(),
            solution : solution.to_bytes_be(),
            vote : 2
        });

        let validate_res = server_impl.validate_vote(validate_req).await;
        assert!(validate_res.is_err(), "Did not throw error while not giving valid vote.");

        let validate_req = Request::new(ValidateVoteReq {
            auth_session_id : "dsadasadas".to_string(),
            solution : solution.to_bytes_be(),
            vote : 1
        });

        let validate_res = server_impl.validate_vote(validate_req).await;
        assert!(validate_res.is_err(), "Did not throw error while not giving valid auth session id.");
        
    }
}