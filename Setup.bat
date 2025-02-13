cargo install wasm-pack
cargo build
cd anonvote_wasm
wasm-pack build --target web
cd ../anonvote_client_web
npm install --save-dev shx
cd ..
