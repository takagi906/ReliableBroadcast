mod common;
mod crypto;
mod node;
mod p2p_network;
use crate::common::Message;
use crate::crypto::KeyPair;
use crate::node::Node;
use fastcrypto::{
    traits::{KeyPair as _, Signer as _},
    Hash as _,
};
use rand::{rngs::OsRng, Rng};
use std::collections::HashMap;
use tokio::sync::mpsc;
#[tokio::main] // ← 关键修复
async fn main() {
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
        0,
        0,
        rx1,
        tx2.clone(),
        tx3.clone(),
    ));
    let handle2 = tokio::spawn(Node::spawn(
        1,
        keypair2.public().clone(),
        peers2.clone(),
        0,
        0,
        rx2,
        tx1.clone(),
        tx4.clone(),
    ));
    let handle3 = tokio::spawn(Node::spawn(
        2,
        keypair3.public().clone(),
        peers3.clone(),
        0,
        0,
        rx3,
        tx1.clone(),
        tx4.clone(),
    ));
    let handle4 = tokio::spawn(Node::spawn(
        3,
        keypair4.public().clone(),
        peers4.clone(),
        0,
        0,
        rx4,
        tx2.clone(),
        tx3.clone(),
    ));
    tx1.send(Message::Data(vec![1, 2, 3])).await;
    let _ = tokio::join!(handle1, handle2, handle3, handle4);
}
