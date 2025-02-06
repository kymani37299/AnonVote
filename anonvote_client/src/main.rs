use anonvote_proto::proto::anonvote::{anon_vote_client::AnonVoteClient, ValidateIdReq, RegisterReq, VoteReq, ValidateVoteReq};
use zkp_protocol::SecretKey;

use std::io;
use std::process;
use num_bigint::BigUint;

fn input_string(prompt: &str) -> String {
    let mut buf = String::new();
    println!("{}", prompt);
    io::stdin().read_line(&mut buf).unwrap();
    buf.trim().to_string() // Trim any trailing newlines or whitespace
}

fn handle_response<T>(res: Result<T, tonic::Status>) -> T {
    match res {
        Ok(response) => {
            return response;
        }
        Err(e) => {
            eprintln!("{} : {}", e.code() , e.message());
            process::exit(1); // Exit with a non-zero status code on error
        }
    }
}

#[tokio::main]
async fn main() {
    let mut client = AnonVoteClient::connect("http://127.0.0.1:50051")
        .await
        .expect("Could not connect to the server");

    let id = input_string("Please provide ID number");
    let res = client.validate_id(ValidateIdReq { id }).await;
    let res = handle_response(res).into_inner();

    let secret_key = SecretKey::generate();
    let public_key = secret_key.generate_public_key();

    let res = client.register(RegisterReq {
        registration_key : res.registration_key,
        a : public_key.a().to_bytes_be(),
        b : public_key.b().to_bytes_be(),
        alpha : public_key.alpha().to_bytes_be(),
        beta : public_key.beta().to_bytes_be()
    }).await;
    let res = handle_response(res).into_inner();

    let vote = input_string("Enter a vote").parse().unwrap();

    let (k, ka, kb) = public_key.generate_challenge_request();
    let res = client.vote(VoteReq {
        vote,
        a : public_key.a().to_bytes_be(),
        b : public_key.b().to_bytes_be(),
        alpha : public_key.alpha().to_bytes_be(),
        beta : public_key.beta().to_bytes_be(),
        ka : ka.to_bytes_be(),
        kb : kb.to_bytes_be()
    }).await;
    let res = handle_response(res).into_inner();

    let solution = secret_key.solve(&k, &BigUint::from_bytes_be(&res.challenge));

    let res = client.validate_vote(ValidateVoteReq{
        auth_session_id : res.auth_session_id,
        vote,
        solution : solution.to_bytes_be()
    }).await;
    let _res = handle_response(res).into_inner();

    println!("You've successfully voted!");
}
