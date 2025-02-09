use wasm_bindgen::prelude::*;
use zkp_protocol::*;
use num_bigint::BigUint;

// wasm-pack build --target web

#[wasm_bindgen]
pub struct PublicKeyWasm {
    a : Vec<u8>,
    b : Vec<u8>,
    alpha : Vec<u8>,
    beta : Vec<u8>,
}

#[wasm_bindgen]
pub struct SecretKeyWasm {
    secret : Vec<u8>,
}

#[wasm_bindgen]
pub struct ChallengeRequestWasm {
    k : Vec<u8>,
    ka : Vec<u8>,
    kb : Vec<u8>
}

#[wasm_bindgen]
impl SecretKeyWasm {
    pub fn new(secret : Vec<u8>) -> SecretKeyWasm {
        SecretKeyWasm {
            secret
        }
    }
    
    pub fn secret(&self) -> Vec<u8> {
        self.secret.clone()
    }

    pub fn solve(&self, k : Vec<u8>, challenge : Vec<u8>) -> Vec<u8> {
        let k = BigUint::from_bytes_be(&k);
        let challenge = BigUint::from_bytes_be(&challenge);
        let secret = self.parse();
        secret.solve(&k, &challenge).to_bytes_be()
    }
}

#[wasm_bindgen]
impl PublicKeyWasm {
    pub fn new(a : Vec<u8>, b : Vec<u8>, alpha : Vec<u8>, beta : Vec<u8>) -> PublicKeyWasm {
        PublicKeyWasm {
            a,b,alpha,beta
        }
    }

    pub fn generate_challenge_request(&self) -> ChallengeRequestWasm {
        let public_key = self.parse();
        let challenge_request = public_key.generate_challenge_request();
        ChallengeRequestWasm{
            k : challenge_request.0.to_bytes_be(),
            ka : challenge_request.1.to_bytes_be(),
            kb : challenge_request.2.to_bytes_be()
        }
    }

    pub fn a(&self) -> Vec<u8> {
        self.a.clone()
    }
    pub fn b(&self) -> Vec<u8> {
        self.b.clone()
    }
    pub fn alpha(&self) -> Vec<u8> {
        self.alpha.clone()
    }
    pub fn beta(&self) -> Vec<u8> {
        self.beta.clone()
    }
}

#[wasm_bindgen]
impl ChallengeRequestWasm {
    pub fn k(&self) -> Vec<u8> {
        self.k.clone()
    }
    pub fn ka(&self) -> Vec<u8> {
        self.ka.clone()
    }
    pub fn kb(&self) -> Vec<u8> {
        self.kb.clone()
    }
}

impl SecretKeyWasm {
    pub fn parse(&self) -> SecretKey {
        SecretKey::from_bytes_be(&self.secret)
    }
}

impl PublicKeyWasm {
    pub fn parse(&self) -> PublicKey {
        PublicKey::from_bytes_be(&self.a, &self.b, &self.alpha, &self.beta)
    }
}

#[wasm_bindgen]
pub fn generate_secret_key() -> SecretKeyWasm {
    let secret = SecretKey::generate();
    SecretKeyWasm {
        secret : secret.secret().to_bytes_be()
    }
}

#[wasm_bindgen]
pub fn generate_public_key(secret : &SecretKeyWasm) -> PublicKeyWasm {
    let secret = secret.parse();
    let public = secret.generate_public_key();
    PublicKeyWasm {
        a : public.a().to_bytes_be(),
        b : public.b().to_bytes_be(),
        alpha : public.alpha().to_bytes_be(),
        beta : public.beta().to_bytes_be(),
    }
}