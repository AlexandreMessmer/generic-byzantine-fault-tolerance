use std::time::Duration;

use talk::crypto::Identity;

use talk::sync::fuse::Fuse;
use talk::unicast::Receiver;
use tokio::sync::mpsc::Receiver as MPSCReceiver;

use crate::system::peer::Peer;
use crate::talk::command::Command;
use crate::talk::message::Message::{self, Plaintext};

pub struct PeerRunner {
    pub peer: Peer,
    pub outlet: MPSCReceiver<Command>,
    pub keys_table: Vec<Identity>,
    pub fuse: Fuse,
}

impl PeerRunner {
    pub fn new(peer: Peer, outlet: MPSCReceiver<Command>, keys_table: Vec<Identity>) -> Self {
        PeerRunner {
            peer,
            outlet,
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

                Some(command) = self.outlet.recv() => {
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
                    let id = self.keys_table.get(id).unwrap().clone();
                    let _ = self.peer.sender.spawn_send(id, message.clone(), &self.fuse);
                }
            }

            _ => {}
        }
    }

    async fn simulate_busy(&self) {
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    async fn handle_message(&mut self, _id: Identity, message: Message) {
        match message {
            Plaintext(str) => println!("Peer #{} got: {}", self.peer.id, str),
            _ => {}
        }
    }
}
