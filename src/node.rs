use crate::common::*;
use crate::crypto::KeyPair;
use crate::crypto::{AggregateSignature, PublicKey, Signature};
use crate::p2p_network::P2pNetwork;
use fastcrypto::traits::Signer;
use std::collections::HashMap;
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
    secret_for_test1: Sender<Message>,
    secret_for_test2: Sender<Message>,
}

impl Node {
    pub fn spawn(
        name: i32,
        id: PublicKey,
        peers: HashMap<PublicKey, Sender<Message>>,
        f: usize,
        round: u64,
        message_handler: Receiver<Message>,
        secret_for_test1: Sender<Message>,
        secret_for_test2: Sender<Message>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            Self {
                name,
                id,
                peers,
                f,
                round,
                message_handler,
                secret_for_test1,
                secret_for_test2,
            }
            .run()
            .await;
        })
    }

    pub async fn r_bcast(&self, data: Vec<u8>) {
        let propose = Propose::new(data, self.id.clone(), self.round).await;
        // self.secret_for_test1.send(Message::Propose(propose)).await;
        P2pNetwork::broadcast(
            self.peers.clone(),
            Message::Propose(propose),
        )
        .await;
    }

    // 处理接收到的消息
    pub async fn run(&mut self) {
        while let Some(message) = self.message_handler.recv().await {
            // 处理消息
            match message {
                Message::Data(data) => {
                    println!("receive a log");
                    self.r_bcast(data).await
                }
                Message::Propose(propose) => {
                    println!("receive a Propose");
                }
                Message::Committed(committed) => {
                    println!("receive a log");
                }
                Message::Certificate(ertificate) => {}
                _ => panic!("Unexpected core message"),
            }
        }
    }
}
