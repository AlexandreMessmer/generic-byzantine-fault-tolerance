use std::time::Duration;

use crate::crypto::identity_table::IdentityTable;
use crate::{system::peer::Peer, talk::command::Command, talk::message::Message};
use talk::{crypto::Identity, sync::fuse::Fuse};
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::time::sleep;
use uuid::Uuid;

use super::peer::PeerId;
use super::peer_runner::PeerRunner;

/// A client is a peer that has a defined behavior in the system
/// Formally, it is a client runner. To make it easier, we will
/// define the client as the same entity as its runner

pub struct Client {
    runner: PeerRunner,
    identity_table: IdentityTable,
}

impl Client {
    pub fn new(runner: PeerRunner, identity_table: &IdentityTable) -> Self {
        Client {
            runner,
            identity_table: identity_table.clone(),
        }
    }

    pub fn id(&self) -> PeerId {
        self.runner.peer.id
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, _) = self.runner.peer.receiver.receive() => {
                    println!("Received something");
                    self.handle_message(id, message).await;
                }

                Some(command) = self.runner.outlet.recv() => {
                    self.handle_command(command).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Execute(message) => {
                let replicas = self.identity_table.replicas().iter();
                for replica in replicas {
                    self.runner.peer.sender.spawn_send(
                        replica.clone(),
                        message.clone(),
                        &self.runner.fuse,
                    );
                }
                //todo!("Implement the wait until, and return a res");
            }
            _ => {}
        }
    }

    async fn wait_for_replicas() -> u64 {
        sleep(Duration::from_secs(10)).await;
        2
    }

    async fn handle_message(&mut self, _id: Identity, message: Message) {
        match message {
            _ => {}
        }
    }
}
