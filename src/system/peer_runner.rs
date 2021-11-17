use std::time::Duration;

use rand::thread_rng;
use talk::crypto::Identity;
use talk::unicast::{Acknowledgement, PushSettings};

use talk::sync::fuse::Fuse;

use tokio::sync::mpsc::Receiver as MPSCReceiver;

use crate::system::peer::Peer;
use crate::system::Command;
use crate::system::Message::Plaintext;
use rand::Rng;

use super::Message;

pub struct PeerRunner {
    peer: Peer,
    runner_outlet: MPSCReceiver<Command>,
    keys_table: Vec<Identity>,

    fuse: Fuse,
}

impl PeerRunner {
    pub fn new(
        peer: Peer,
        runner_outlet: MPSCReceiver<Command>,
        keys_table: Vec<Identity>,
    ) -> Self {
        PeerRunner {
            peer,
            runner_outlet,
            keys_table,

            fuse: Fuse::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, _) = self.peer.receiver.receive() => {
                    println!("Received something");
                    self.handle_message(id, message).await;
                }

                Some(command) = self.runner_outlet.recv() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Send(id, message) => {
                println!("Peer #{} forwarded: {:?}", self.peer.id, message);
                self.simulate_busy().await;
                if id < self.keys_table.len() {
                    println!("HERE X");
                    let id = self.keys_table.get(id).unwrap().clone();
                    let _ = self
                        .peer
                        .sender
                        .spawn_push(
                            id,
                            message.clone(),
                            PushSettings {
                                stop_condition: Acknowledgement::Weak,
                                ..Default::default()
                            },
                            &self.fuse,
                        );

                    println!("HERE Y");
                }
            }
        }
    }

    async fn simulate_busy(&self) {
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    async fn handle_message(&mut self, _id: Identity, message: Message) {
        match message {
            Plaintext(str) => println!("Peer #{} got: {}", self.peer.id, str),
        }
    }
}
