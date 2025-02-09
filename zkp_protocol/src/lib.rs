use num_bigint::BigUint;

pub mod zkp_util {
    use num_bigint::{ BigUint, RandBigInt};

    pub fn generate_random_below(bound: &BigUint) -> BigUint {
        let mut rng = rand::thread_rng();
        rng.gen_biguint_below(bound)
    }

    pub fn p() -> BigUint {
        let p = hex::decode("B10B8F96A080E01DDE92DE5EAE5D54EC52C99FBCFB06A3C69A6A9DCA52D23B616073E28675A23D189838EF1E2EE652C013ECB4AEA906112324975C3CD49B83BFACCBDD7D90C4BD7098488E9C219A73724EFFD6FAE5644738FAA31A4FF55BCCC0A151AF5F0DC8B4BD45BF37DF365C1A65E68CFDA76D4DA708DF1FB2BC2E4A4371").expect("Invalid p hex data!");
        let p = BigUint::from_bytes_be(&p);
        p
    }

    pub fn q() -> BigUint {
        let q = hex::decode("F518AA8781A8DF278ABA4E7D64B7CB9D49462353").expect("Invalid q hex data");
        let q = BigUint::from_bytes_be(&q);
        q
    }

    pub fn alpha() -> BigUint {
        let alpha = hex::decode("A4D1CBD5C3FD34126765A442EFB99905F8104DD258AC507FD6406CFF14266D31266FEA1E5C41564B777E690F5504F213160217B4B01B886A5E91547F9E2749F4D7FBD7D3B9A92EE1909D0D2263F80A76A6A24C087A091F531DBF0A0169B6A28AD662A4D18E73AFA32D779D5918D08BC8858F4DCEF97C2A24855E6EEB22B3B2E5").expect("Invalid alpha hex value");
        let alpha = BigUint::from_bytes_be(&alpha);
        alpha
    }

    pub fn generate_challenge() -> BigUint {
        generate_random_below(&q())
    }
}

pub struct SecretKey {
    secret : BigUint,
}

#[derive(Hash, Clone)]
pub struct PublicKey {
    a : BigUint,
    b : BigUint,
    alpha : BigUint,
    beta : BigUint,
}

impl PublicKey {
    pub fn new(a : BigUint, b : BigUint, alpha : BigUint, beta : BigUint) -> PublicKey {
        PublicKey {
            a,b,alpha,beta
        }
    }

    pub fn from(a : Vec<u8>, b : Vec<u8>, alpha : Vec<u8>, beta : Vec<u8>) -> PublicKey {
        PublicKey {
            a : BigUint::from_bytes_be(&a),
            b : BigUint::from_bytes_be(&b),
            alpha : BigUint::from_bytes_be(&alpha),
            beta : BigUint::from_bytes_be(&beta),
        }
    }

    pub fn from_bytes_be(a : &Vec<u8>, b : &Vec<u8>, alpha : &Vec<u8>, beta : &Vec<u8>) -> PublicKey {
        PublicKey {
            a : BigUint::from_bytes_be(a),
            b : BigUint::from_bytes_be(b),
            alpha : BigUint::from_bytes_be(alpha),
            beta : BigUint::from_bytes_be(beta),
        }
    }

    pub fn generate_challenge_request(&self) -> (BigUint, BigUint, BigUint) {
        let k = zkp_util::generate_random_below(&zkp_util::q());
        let ka = self.alpha.modpow(&k, &zkp_util::p());
        let kb = self.beta.modpow(&k, &zkp_util::p());
        (k ,ka, kb)
    }

    pub fn verify(&self, ka : &BigUint, kb : &BigUint, challenge : &BigUint, solution : &BigUint) -> bool {
        let cond1 = *ka == (self.alpha.modpow(solution, &zkp_util::p()) * self.a.modpow(challenge, &zkp_util::p())).modpow(&BigUint::from(1u32), &zkp_util::p());
        let cond2 = *kb == (self.beta.modpow(solution, &zkp_util::p()) * self.b.modpow(challenge, &zkp_util::p())).modpow(&BigUint::from(1u32), &zkp_util::p());
        cond1 && cond2
    }

    pub fn a(&self) -> &BigUint {
        &self.a
    }

    pub fn b(&self) -> &BigUint {
        &self.b
    }

    pub fn alpha(&self) -> &BigUint {
        &self.alpha
    }

    pub fn beta(&self) -> &BigUint {
        &self.beta
    }
}

impl SecretKey {
    pub fn new(secret : BigUint) -> SecretKey {
        SecretKey {
            secret
        }
    }

    pub fn from_bytes_be(bytes : &Vec<u8>) -> SecretKey {
        SecretKey {
            secret : BigUint::from_bytes_be(&bytes)
        }
    }

    pub fn generate() -> SecretKey {
        let secret = zkp_util::generate_random_below(&zkp_util::q());
        SecretKey {
            secret
        }
    }

    pub fn generate_public_key(&self) -> PublicKey {
        let beta = zkp_util::alpha().modpow(&zkp_util::generate_random_below(&zkp_util::q()), &zkp_util::p());
        let alpha = zkp_util::alpha();
        let a = alpha.modpow(&self.secret, &zkp_util::p());
        let b = beta.modpow(&self.secret, &zkp_util::p());
        PublicKey::new(a,b,alpha,beta)
    }

    pub fn solve(&self, k : &BigUint, challenge : &BigUint) -> BigUint {
        if *k >= challenge * &self.secret {
            return (k - challenge * &self.secret).modpow(&BigUint::from(1u32), &zkp_util::q());
        }
        return &zkp_util::q() - (challenge * &self.secret - k).modpow(&BigUint::from(1u32), &zkp_util::q());
    }

    pub fn secret(&self) -> &BigUint {
        &self.secret
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_random() {
        for _ in 0..10 {
            let secret = zkp_util::generate_random_below(&zkp_util::p());
            let secret_key = SecretKey::new(secret);
            let public_key = secret_key.generate_public_key();
    
            let (k,ka,kb) = public_key.generate_challenge_request();
    
            let challenge = zkp_util::generate_challenge();
    
            let solution = secret_key.solve(&k, &challenge);
    
            let result = public_key.verify(&ka, &kb, &challenge, &solution);
            assert!(result);
        }
    }
}