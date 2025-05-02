use crate::aggregator::{CommittedAggregator, CommittedAggregators};
use crate::common::*;
use crate::crypto::{AggregateSignature, PublicKey, Signature};
use crate::p2p_network::P2pNetwork;
use fastcrypto::SignatureService;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::{sync::watch, task::JoinHandle};

pub struct Node {
    name: i32,
    id: PublicKey,
    peers: HashMap<PublicKey, Sender<Message>>,
    f: usize, // 容错阈值
    round: u64,
    message_handler: Receiver<Message>,
    signature_service: SignatureService<Signature>,
    committed_aggregators: CommittedAggregators,
    finish: HashSet<(u64, PublicKey)>,
}

impl Node {
    pub fn spawn(
        name: i32,
        id: PublicKey,
        peers: HashMap<PublicKey, Sender<Message>>,
        f: usize,
        round: u64,
        message_handler: Receiver<Message>,
        signature_service: SignatureService<Signature>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                name,
                id,
                peers,
                f,
                round,
                message_handler,
                signature_service,
                committed_aggregators: CommittedAggregators::new(f),
                finish: HashSet::new(),
            }
            .run()
            .await;
        })
    }

    // 可靠广播
    pub async fn r_bcast(&mut self, data: Vec<u8>) {
        let propose = Propose::new(data, self.id.clone(), self.round).await;
        P2pNetwork::broadcast(self.peers.clone(), Message::Propose(propose.clone())).await;
        self.process_propose(propose.clone()).await;
    }

    pub async fn r_deliver(&self, data: Vec<u8>) {
        if let Ok(string_data) = String::from_utf8(data) {
            println!(
                "{} r_deliver the data: {}",
                &self.id.clone().to_string()[..6],
                string_data
            );
        }
    }

    pub async fn process_propose(&mut self, propose: Propose) {
        let round = propose.round;
        if round < self.round {
            println!(
                "propose from {} is out of date",
                &propose.sender.to_string()[..6]
            );
            return;
        }
        let commit = Committed::new(
            propose.data,
            propose.sender.clone(),
            self.id.clone(),
            round,
            &mut self.signature_service,
        )
        .await;
        self.process_committed(commit.clone()).await;
        P2pNetwork::broadcast(self.peers.clone(), Message::Committed(commit)).await;
    }

    pub async fn process_committed(&mut self, committed: Committed) {
        if self
            .finish
            .contains(&(committed.round, committed.author.clone()))
        {
            return;
        }

        if (committed.sender != self.id) {
            if !committed.verify(self.id.clone()) {
                println!(
                    "committed from {} is not valid",
                    &committed.sender.to_string()[..6]
                );
                return;
            }
        }
        if let Some(certificate) = self.committed_aggregators.append(committed) {
            self.process_certificate(certificate).await;
        }
    }
    pub async fn process_certificate(&mut self, certificate: Certificate) {
        let r = certificate.round;
        let publicKey = certificate.author.clone();
        if !self.finish.insert((r, publicKey)) {
            return;
        }
        self.r_deliver(certificate.data.clone()).await;
        P2pNetwork::broadcast(
            self.peers.clone(),
            Message::Certificate(certificate.clone()),
        )
        .await;
    }

    // 处理接收到的消息
    pub async fn run(&mut self) {
        while let Some(message) = self.message_handler.recv().await {
            // 处理消息
            match message {
                Message::Data(data) => {
                    println!(
                        "{} r_bcast data {}",
                        &self.id.to_string()[..6],
                        String::from_utf8(data.clone()).unwrap()
                    );
                    self.r_bcast(data).await
                }
                Message::Propose(propose) => {
                    self.process_propose(propose).await;
                }
                Message::Committed(committed) => {
                    self.process_committed(committed).await;
                }
                Message::Certificate(certificate) => {
                    self.process_certificate(certificate).await;
                }
                _ => panic!("Unexpected core message"),
            }
        }
    }
}
