use crate::common::Message;
use crate::crypto::PublicKey;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender;

pub struct P2pNetwork {}

impl P2pNetwork {
    pub async fn broadcast(receivers: HashMap<PublicKey, Sender<Message>>, message: Message) {
        for (_, sender) in receivers.into_iter() {
            let msg = message.clone();
            let _ = sender.clone().send(msg).await;
        }
    }
}
