use fastcrypto::traits::VerifyingKey;
use fastcrypto::{traits::AggregateAuthenticator, Digest, Hash, SignatureService, DIGEST_LEN};
use mysten_util_mem::MallocSizeOf;
use std::fmt;
// 消息类型
use crate::crypto::{AggregateSignature, PublicKey, Signature};
use base64ct::Encoding;
use blake2::{digest::Update, VarBlake2b};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Propose {
    pub sender: PublicKey,
    pub data: Vec<u8>,
    pub round: u64,
}

impl Hash for Propose {
    type TypedDigest = ProposeDigest;
    fn digest(&self) -> ProposeDigest {
        let hasher_update = |hasher: &mut VarBlake2b| {
            hasher.update(&self.sender);
            hasher.update(self.round.to_le_bytes());
            hasher.update(&self.data);
        };
        ProposeDigest(fastcrypto::blake2b_256(hasher_update))
    }
}

impl Propose {
    pub async fn new(data: Vec<u8>, sender: PublicKey, round: u64) -> Self {
        let propose = Self {
            sender,
            data,
            round,
        };
        propose
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Committed {
    pub sender: PublicKey,
    pub data: Vec<u8>,
    pub author: PublicKey,
    pub round: u64,
    pub signature: <PublicKey as VerifyingKey>::Sig,
}

impl Hash for Committed {
    type TypedDigest = CommittedDigest;
    fn digest(&self) -> CommittedDigest {
        let hasher_update = |hasher: &mut VarBlake2b| {
            hasher.update(&self.sender);
            hasher.update(self.round.to_le_bytes());
            hasher.update(&self.data);
        };
        CommittedDigest(fastcrypto::blake2b_256(hasher_update))
    }
}

impl Committed {
    pub async fn new(
        data: Vec<u8>,
        author: PublicKey,
        sender: PublicKey,
        round: u64,
        signature_service: &mut SignatureService<Signature>,
    ) -> Self {
        let committed = Self {
            sender,
            data,
            author,
            round,
            signature: Signature::default(),
        };
        let signature = signature_service
            .request_signature(committed.digest().into())
            .await;
        Self {
            signature,
            ..committed
        }
    }
    pub fn verify(&self, public_key: PublicKey) -> bool {
        true
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Certificate {
    pub sender: PublicKey,
    pub data: Vec<u8>,
    pub author: PublicKey,
    pub round: u64,
    pub aggregated_signature: AggregateSignature,
}

impl Certificate {
    pub fn new(
        data: Vec<u8>,
        sender: PublicKey,
        author: PublicKey,
        round: u64,
        votes: Vec<(PublicKey, Signature)>,
    ) -> Self {
        let aggregated_signature = if votes.is_empty() {
            AggregateSignature::default()
        } else {
            AggregateSignature::aggregate(votes.clone().into_iter().map(|(_, sig)| sig).collect())
                .map_err(|e| {
                    eprintln!("Failed to aggregate signatures: {:?}", e);
                    e
                })
                .unwrap()
        };
        Self {
            sender,
            author,
            data,
            round,
            aggregated_signature: aggregated_signature,
        }
    }
}
#[derive(Clone)]
pub enum Message {
    Data(Vec<u8>),
    Propose(Propose),
    Committed(Committed),
    Certificate(Certificate),
}

#[derive(
    Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash, PartialOrd, Ord, MallocSizeOf,
)]
pub struct ProposeDigest([u8; DIGEST_LEN]);
impl From<ProposeDigest> for Digest {
    fn from(hd: ProposeDigest) -> Self {
        Digest::new(hd.0)
    }
}

impl fmt::Debug for ProposeDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", base64::encode(self.0))
    }
}

impl fmt::Display for ProposeDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", base64::encode(self.0).get(0..16).unwrap())
    }
}

#[derive(
    Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash, PartialOrd, Ord, MallocSizeOf,
)]
pub struct CommittedDigest([u8; DIGEST_LEN]);
impl From<CommittedDigest> for Digest {
    fn from(hd: CommittedDigest) -> Self {
        Digest::new(hd.0)
    }
}

impl fmt::Debug for CommittedDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", base64::encode(self.0))
    }
}

impl fmt::Display for CommittedDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", base64::encode(self.0).get(0..16).unwrap())
    }
}
