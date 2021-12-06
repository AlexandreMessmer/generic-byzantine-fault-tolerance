use std::time::Duration;

use talk::crypto::Identity;

use talk::sync::fuse::Fuse;
use tokio::sync::mpsc::Receiver as MPSCReceiver;

use super::*;
use crate::system::Peer;
use crate::talk::Command;
use crate::talk::Message::{self, Plaintext};

impl PeerRunner {
    pub(in crate::system) fn new(
        peer: Peer,
        outlet: MPSCReceiver<Command>,
        keys_table: Vec<Identity>,
        settings: RunnerSettings,
    ) -> Self {
        PeerRunner {
            peer,
            outlet,
            keys_table,
            fuse: Fuse::new(),
            settings,
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

    pub fn compose_runners(
        nbr: usize,
        keys: Vec<Identity>,
        senders: Vec<Sender>,
        receivers: Vec<Receiver>,
        outlets: Vec<MPSCReceiver<Command>>,
        keys_table: Vec<Identity>,
        settings: RunnerSettings,
    ) -> Vec<PeerRunner> {
        let (keys, senders, receivers) =
            (keys.into_iter(), senders.into_iter(), receivers.into_iter());
        let ids = (0..nbr).into_iter();
        ids.zip(keys)
            .zip(senders)
            .zip(receivers)
            .zip(outlets)
            .map(|((((id, key), sender), receiver), outlet)| {
                PeerRunner::new(
                    Peer::new(id, key, sender, receiver),
                    outlet,
                    keys_table.clone(),
                    settings.clone(),
                )
            })
            .collect::<Vec<_>>()
    }

    pub async fn simulate_delay(&self) {
        tokio::time::sleep(self.settings.transmission_delay()).await;
    }
}
