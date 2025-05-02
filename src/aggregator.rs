use crate::common::{Certificate, Committed};
use crate::crypto::{AggregateSignature, PublicKey, Signature};
use anyhow::ensure;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct CommittedAggregators {
    aggregators: HashMap<(u64, PublicKey), CommittedAggregator>,
    f: usize,
}
impl CommittedAggregators {
    pub fn new(f: usize) -> Self {
        Self {
            aggregators: HashMap::new(),
            f,
        }
    }
    pub fn append(&mut self, commit: Committed) -> Option<Certificate> {
        let r = commit.round;
        let public_key = commit.author.clone();
        let aggregator = self
            .aggregators
            .entry((r, public_key))
            .or_insert_with(|| CommittedAggregator::new(self.f)); // 假设 self.f 是容错阈值
        let c = aggregator.append(commit);
        c
    }
}

pub struct CommittedAggregator {
    weight: i32,
    committeds: Vec<(PublicKey, Signature)>,
    used: HashSet<PublicKey>,
    f: usize,
}

impl CommittedAggregator {
    pub fn new(f: usize) -> Self {
        Self {
            weight: 0,
            committeds: Vec::new(),
            used: HashSet::new(),
            f,
        }
    }
    pub fn append(&mut self, commit: Committed) -> Option<Certificate> {
        let author = commit.sender.clone();
        // Ensure it is the first time this authority votes.
        if !self.used.insert(author.clone()) {
            return None;
        }
        self.weight += 1;
        self.committeds
            .push((author.clone(), commit.signature.clone()));
        if self.weight >= self.f as i32 {
            return Some(Certificate::new(
                commit.data,
                commit.sender.clone(),
                commit.author.clone(),
                commit.round,
                self.committeds.clone(),
            ));
        }
        None
    }
}
