use fastcrypto::bls12381;

pub type PublicKey = bls12381::BLS12381PublicKey;
pub type Signature = bls12381::BLS12381Signature;
pub type AggregateSignature = bls12381::BLS12381AggregateSignature;
pub type PrivateKey = bls12381::BLS12381PrivateKey;
pub type KeyPair = bls12381::BLS12381KeyPair;
