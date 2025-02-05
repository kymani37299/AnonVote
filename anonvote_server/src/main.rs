use anonvote_proto::proto::anonvote::anon_vote_server::{AnonVote, AnonVoteServer};
use anonvote_proto::proto::anonvote::{ValidateIdReq, ValidateIdRes, RegisterReq, RegisterRes, VoteReq, VoteRes, ValidateVoteReq, ValidateVoteRes};

use tonic::{Request, Response, Status, Code, transport::Server};

#[derive(Default)]
struct AnonVoteImpl {
}

#[tonic::async_trait]
impl AnonVote for AnonVoteImpl {
    async fn validate_id(&self, req : Request<ValidateIdReq>) -> Result<Response<ValidateIdRes>, Status> {
        todo!()
    }

    async fn register(&self, req : Request<RegisterReq>) -> Result<Response<RegisterRes>, Status> {
        todo!()
    }

    async fn vote(&self, req : Request<VoteReq>) -> Result<Response<VoteRes>, Status> {
        todo!()
    }

    async fn validate_vote(&self, req : Request<ValidateVoteReq>) -> Result<Response<ValidateVoteRes>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:50051".to_string();
    let anonvote_impl = AnonVoteImpl::default();

    println!("Starting server...");

    let server = Server::builder()
        .add_service(AnonVoteServer::new(anonvote_impl))
        .serve(addr.parse().expect("Could not convert address"));

    println!("Server started on {}", addr);

    server.await.unwrap();
}
