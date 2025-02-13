mod server_impl;
mod db;
mod model {
    pub mod user_data;
    pub mod challenge_data;
}

use anonvote_proto::proto::anonvote::anon_vote_server::AnonVoteServer;
use tonic::transport::Server;
use server_impl::AnonVoteImpl;
use db::AnonVoteDB;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:50051".to_string();
    let db = AnonVoteDB::connect();
    let vote_option_count = db.get_vote_options_count() as u32;
    let anonvote_impl = AnonVoteImpl::new(db, vote_option_count);

    println!("Starting server...");

    let server = Server::builder()
        .add_service(AnonVoteServer::new(anonvote_impl))
        .serve(addr.parse().expect("Could not convert address"));

    println!("Server started on {}", addr);

    server.await.unwrap();
}
