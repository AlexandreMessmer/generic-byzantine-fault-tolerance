use std::time::Duration;

use crate::crypto::identity_table::IdentityTable;
use crate::{talk::command::Command, talk::message::Message};
use talk::{crypto::Identity, sync::fuse::Fuse};
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use super::*;
use super::runner::Runner;

/// A replica is a peer that has a defined behavior in the system
/// Formally, it is a replica runner. To make it easier, we will
/// define the replica as the same entity as its runner

pub struct Replica {
    runner: PeerRunner,
    identity_table: IdentityTable,
}

impl Replica {
    pub fn new(runner: PeerRunner, identity_table: &IdentityTable) -> Self {
        Replica {
            runner,
            identity_table: identity_table.clone(),
        }
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
            Command::Testing(sender) => {
                println!("Replica #{} starts testing...", self.id());
                    for client in self.identity_table.clients().iter() {
                        self.send(client, Message::Testing);
                    }
                    for replica in self.identity_table.replicas() {
                        self.send(replica, Message::Testing);
                    }
                if let Some(rx) = sender {
                    let _ = rx.send(Command::Answer);
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
            _ => {}
        }
    }
}

impl Runner for Replica {
    fn send(&self, target: &Identity, message: Message){
        self.runner.peer.sender.spawn_send(target.clone(), message, &self.runner.fuse);
    }

    fn id(&self) -> PeerId{
        self.runner.peer.id
    }

}
