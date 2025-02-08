import init, {generate_secret_key, generate_public_key} from "./anonvote_wasm/pkg/anonvote_wasm.js";

let wasmInitialized = false; 

init().then(wasm => {
    wasmInitialized = true;
});

export function generate_key_pair() {
    if (!wasmInitialized) {
        return undefined;
    }

    let secret_key = generate_secret_key();
    let public_key = generate_public_key(secret_key);
    return {
        secret_key : secret_key,
        public_key : public_key
    };
}