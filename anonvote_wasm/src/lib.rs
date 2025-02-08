use wasm_bindgen::prelude::*;
use zkp_protocol::*;

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
impl SecretKeyWasm {
    pub fn secret(&self) -> Vec<u8> {
        self.secret.clone()
    }
}

#[wasm_bindgen]
impl PublicKeyWasm {
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

impl SecretKeyWasm {
    pub fn parse(&self) -> SecretKey {
        SecretKey::from_bytes_be(&self.secret)
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