use std::time::Duration;

use crate::crypto::identity_table::IdentityTable;
use super::{command::Command, message::Message, peer::Peer};
use talk::{crypto::Identity, sync::fuse::Fuse};
use tokio::sync::mpsc::{Receiver as MPSCReceiver};
/// A client is a peer that has a defined behavior in the system
/// Formally, it is a client runner. To make it easier, we will
/// define the client as the same entity as its runner

pub struct Client {
    client: Peer,
    outlet: MPSCReceiver<Command>,
    identity_table: IdentityTable,
    fuse: Fuse,
}

impl Client {
    pub fn new(
        client: Peer,
        outlet: MPSCReceiver<Command>,
        identity_table: &IdentityTable,
    ) -> Self {
        Client {
            client,
            outlet,
            identity_table: identity_table.clone(),
            fuse: Fuse::new(),
        }
    }

    pub async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, _) = self.client.receiver.receive() => {
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
                    //
                },

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