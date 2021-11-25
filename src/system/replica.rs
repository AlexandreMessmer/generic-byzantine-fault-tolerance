use std::time::Duration;

use crate::crypto::identity_table::IdentityTable;
use crate::{talk::command::Command, talk::message::Message, system::peer::Peer};
use talk::{crypto::Identity, sync::fuse::Fuse};
use tokio::sync::mpsc::{Receiver as MPSCReceiver};

/// A replica is a peer that has a defined behavior in the system
/// Formally, it is a replica runner. To make it easier, we will
/// define the replica as the same entity as its runner

pub struct Replica {
    replica: Peer,
    outlet: MPSCReceiver<Command>,
    identity_table: IdentityTable,
    pub fuse: Fuse,
}

impl Replica {
    pub fn new(
        replica: Peer,
        outlet: MPSCReceiver<Command>,
        identity_table: &IdentityTable,
    ) -> Self {
        Replica {
            replica,
            outlet,
            identity_table: identity_table.clone(),
            fuse: Fuse::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, _) = self.replica.receiver.receive() => {
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