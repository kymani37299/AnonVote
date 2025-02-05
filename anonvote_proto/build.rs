fn main() {
    tonic_build::configure()
        .build_server(true)
        .out_dir("src/proto/") // you can change the generated code's location
        .compile(
            &["proto/anonvote.proto"],
            &["proto/"], // specify the root location to search proto dependencies
        )
        .unwrap();
}