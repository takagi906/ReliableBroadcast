mod aggregator;
mod common;
mod crypto;
mod node;
mod p2p_network;

use crate::common::Message;
use crate::crypto::KeyPair;
use crate::node::Node;
use fastcrypto::{
    traits::{KeyPair as _, Signer as _},
    Hash as _, SignatureService,
};
use rand::{rngs::OsRng, Rng};
use std::collections::HashMap;
use tokio::sync::mpsc;
#[tokio::main] // ← 关键修复
async fn main() {
    let f = 3;
    let keypair1 = KeyPair::generate(&mut OsRng);
    let keypair2 = KeyPair::generate(&mut OsRng);
    let keypair3 = KeyPair::generate(&mut OsRng);
    let keypair4 = KeyPair::generate(&mut OsRng);

    let (tx1, rx1) = mpsc::channel::<Message>(32);
    let (tx2, rx2) = mpsc::channel::<Message>(32);
    let (tx3, rx3) = mpsc::channel::<Message>(32);
    let (tx4, rx4) = mpsc::channel::<Message>(32);

    let mut peers1 = HashMap::new();
    let mut peers2 = HashMap::new();
    let mut peers3 = HashMap::new();
    let mut peers4 = HashMap::new();

    peers1.insert(keypair2.public().clone(), tx2.clone());
    peers1.insert(keypair3.public().clone(), tx3.clone());

    peers2.insert(keypair1.public().clone(), tx1.clone());
    peers2.insert(keypair4.public().clone(), tx4.clone());

    peers3.insert(keypair1.public().clone(), tx1.clone());
    peers3.insert(keypair4.public().clone(), tx4.clone());

    peers4.insert(keypair2.public().clone(), tx2.clone());
    peers4.insert(keypair3.public().clone(), tx3.clone());

    let handle1 = tokio::spawn(Node::spawn(
        0,
        keypair1.public().clone(),
        peers1.clone(),
        f,
        0,
        rx1,
        SignatureService::new(keypair1.copy()),
    ));
    let handle2 = tokio::spawn(Node::spawn(
        1,
        keypair2.public().clone(),
        peers2.clone(),
        f,
        0,
        rx2,
        SignatureService::new(keypair2.copy()),
    ));
    let handle3 = tokio::spawn(Node::spawn(
        2,
        keypair3.public().clone(),
        peers3.clone(),
        f,
        0,
        rx3,
        SignatureService::new(keypair3.copy()),
    ));
    let handle4 = tokio::spawn(Node::spawn(
        3,
        keypair4.public().clone(),
        peers4.clone(),
        f,
        0,
        rx4,
        SignatureService::new(keypair4.copy()),
    ));
    let s1 = String::from("Hello, World!");
    let s2 = String::from("Hello, Rust!");
    let s3 = String::from("Hello, Cpp!");
    let s4 = String::from("Hello, Golang!");
    tx1.send(Message::Data(s1.into_bytes())).await;
    tx2.send(Message::Data(s2.into_bytes())).await;
    tx3.send(Message::Data(s3.into_bytes())).await;
    tx4.send(Message::Data(s4.into_bytes())).await;
    let _ = tokio::join!(handle1, handle2, handle3, handle4);
}
