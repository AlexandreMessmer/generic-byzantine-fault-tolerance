use std::time::Duration;

use talk::{crypto::Identity, sync::fuse::Fuse};

use super::*;

use crate::talk::{Command, Message};

impl PeerRunner {
    pub(in crate::system) fn new(
        peer: Peer,
        outlet: InstructionReceiver,
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

    async fn handle_command(&mut self, instruction: Instruction) {
        match instruction {
            (Command::Send(id, message), _) => {
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
            Message::Plaintext(str) => println!("Peer #{} got: {}", self.peer.id, str),
            _ => {}
        }
    }

    pub fn compose_runners(
        nbr: usize,
        keys: Vec<Identity>,
        senders: Vec<Sender>,
        receivers: Vec<Receiver>,
        outlets: Vec<InstructionReceiver>,
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

    pub fn peer_mut(&mut self) -> &mut Peer {
        &mut self.peer
    }

    pub fn peer(&self) -> &Peer {
        &self.peer
    }

    pub fn outlet(&mut self) -> &InstructionReceiver {
        &self.outlet
    }
}

#[async_trait::async_trait]
impl Runner for PeerRunner {
    async fn run(&mut self) {
        loop {
            tokio::select! {
                (id, message, _) = self.peer.receive_message() => {
                    println!("Received something");
                    self.handle_message(id, message).await;
                }

                Some(instruction) = self.outlet.recv() => {
                    self.handle_command(instruction).await;
                }
            }
        }
    }

    fn fuse(&self) -> &Fuse {
        &self.fuse
    }
}

impl Identifiable for PeerRunner {
    fn id(&self) -> PeerId {
        self.peer.id()
    }
}
