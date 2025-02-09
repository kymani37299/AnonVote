import init, {generate_secret_key, generate_public_key, PublicKeyWasm, SecretKeyWasm} from "./anonvote_wasm/pkg/anonvote_wasm.js";

let wasmInitialized = false; 

init().then(wasm => {
    wasmInitialized = true;
});

const convertToUint8Array = (obj) => {
    return new Uint8Array(Object.values(obj));
  };

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

export function key_pair_to_json(key_pair) {
    const jsObject = {
        secret : key_pair.secret_key.secret(),
        a : key_pair.public_key.a(),
        b : key_pair.public_key.b(),
        alpha : key_pair.public_key.alpha(),
        beta : key_pair.public_key.beta()
    };
    return JSON.stringify(jsObject);
}

export function json_to_key_pair(jsObject) {
    if(!jsObject)
        return undefined;

    const {a,b,alpha,beta,secret} = jsObject;

    // Ensure all byte arrays are Uint8Arrays
    const aBytes = a ? convertToUint8Array(a) : null;
    const bBytes = b ? convertToUint8Array(b) : null;
    const alphaBytes = alpha ? convertToUint8Array(alpha) : null;
    const betaBytes = beta ? convertToUint8Array(beta) : null;
    const secretBytes = secret ? convertToUint8Array(secret) : null;

    let public_key = PublicKeyWasm.new(
        aBytes, 
        bBytes, 
        alphaBytes, 
        betaBytes);
    let private_key = SecretKeyWasm.new(secretBytes);

    return {
        public_key : public_key,
        private_key : private_key
    };
}