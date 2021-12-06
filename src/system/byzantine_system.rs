
use crate::crypto::identity_table::IdentityTable;
use crate::system::{Peer, PeerId};
use crate::talk::{Command, Message, FeedbackReceiver, FeedbackChannel};
use talk::crypto::Identity;
use talk::sync::fuse::Fuse;
use talk::unicast::test::UnicastSystem;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::mpsc::Sender as MPSCSender;
use tokio::task::JoinHandle;

use super::client::Client;
use super::replica::Replica;
use super::*;
use super::{PeerIdentifier, PeerType};
use crate::settings::{RunnerSettings, SystemSettings as Settings};

/// Use a PeerSystem as model.
pub struct ByzantineSystem {
    settings: Settings,
    client_inlets: Vec<InstructionSender>,
    replica_inlets: Vec<InstructionSender>,
    fuse: Fuse,
}

impl ByzantineSystem {
    pub async fn setup(nbr_clients: usize, nbr_replicas: usize) -> Self {
        let (client_inlets, client_outlets) = ByzantineSystem::create_channels(nbr_clients);
        let (replica_inlets, replica_outlets) = ByzantineSystem::create_channels(nbr_replicas);

        let size = nbr_clients + nbr_replicas;
        let UnicastSystem {
            mut keys,
            mut senders,
            mut receivers,
        } = UnicastSystem::<Message>::setup(size).await.into();

        let mut replica_keys = Vec::<Identity>::new();
        let mut replica_senders = Vec::<Sender>::new();
        let mut replica_receivers = Vec::<Receiver>::new();
        for _ in 0..nbr_replicas {
            replica_keys.push(keys.pop().unwrap());
            replica_senders.push(senders.pop().unwrap());
            replica_receivers.push(receivers.pop().unwrap());
        }

        let identity_table = IdentityTable::new(keys.clone(), replica_keys.clone());
        let system_settings = Settings::default_settings(nbr_clients, nbr_replicas);
        let clients: Vec<Client> = PeerRunner::compose_runners(
            nbr_clients,
            keys,
            senders,
            receivers,
            client_outlets,
            Vec::new(),
            RunnerSettings::from_system(&system_settings),
        )
        .into_iter()
        .map(|runner| Client::new(runner, &identity_table))
        .collect::<Vec<_>>();

        let replicas: Vec<Replica> = PeerRunner::compose_runners(
            nbr_replicas,
            replica_keys,
            replica_senders,
            replica_receivers,
            replica_outlets,
            Vec::new(),
            RunnerSettings::from_system(&system_settings),
        )
        .into_iter()
        .map(|runner| Replica::new(runner, &identity_table))
        .collect::<Vec<_>>();

        let fuse = Fuse::new();
        {
            for mut client in clients {
                fuse.spawn(async move {
                    client.run().await;
                });
            }
        }

        {
            for mut replica in replicas {
                fuse.spawn(async move {
                    replica.run().await;
                });
            }
        }
        ByzantineSystem {
            settings: system_settings,
            client_inlets,
            replica_inlets,
            fuse,
        }
    }

    /// Abstract the creation of a feedback channel
    pub fn send_command(&self, command: Command, (kind, id): PeerIdentifier) -> Option<FeedbackReceiver> {
        let (tx, rx) = FeedbackChannel::channel();
        if let Some(_) = self.send_instruction((command, tx), (kind, id)) {
            return Some(rx);
        }
        None
    }

    fn send_instruction(&self, instruction: Instruction, (kind, id): PeerIdentifier) -> Option<JoinHandle<Option<()>>> {
        let inlet = match kind {
            PeerType::Client => self.client_inlet(id),
            PeerType::Replica => self.replica_inlet(id),
        };
        if let Some(inlet) = inlet {
            let res = self.fuse.spawn(async move {
                let _ = inlet.send(instruction).await;
            });
            return Some(res);
        }

        None
    }


    fn client_inlet(&self, target: PeerId) -> Option<InstructionSender> {
        ByzantineSystem::find_inlet(target, &self.client_inlets)
    }

    fn replica_inlet(&self, target: PeerId) -> Option<InstructionSender> {
        ByzantineSystem::find_inlet(target, &self.replica_inlets)
    }

    fn find_inlet(
        target: PeerId,
        inlets: &Vec<InstructionSender>,
    ) -> Option<InstructionSender> {
        if target < inlets.len() {
            return Some(inlets.get(target).unwrap().clone());
        }
        return None;
    }

    /// Create enought channels
    fn create_channels(size: usize) -> (Vec<InstructionSender>, Vec<InstructionReceiver>) {
        let mut inlets = Vec::<InstructionSender>::new();
        let mut outlets = Vec::<InstructionReceiver>::new();
        for _ in 0..size {
            let (tx, rx) = mpsc::channel::<Instruction>(32);
            inlets.push(tx);
            outlets.push(rx);
        }

        (inlets, outlets)
    }

    pub fn settings(&self) -> Settings {
        self.settings.clone()
    }


}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::oneshot::{self, Receiver, Sender};

    use crate::{system, talk::Feedback};

    use super::*;

    #[test]
    fn take_test() {
        let mut array = vec![1, 2, 3, 4, 5];
        let p1 = vec![array.pop(), array.pop()];
        let array = array.into_iter();
        println!("Array : {:?}, p1 : {:?}", array, p1);
    }

    #[tokio::test]
    async fn client_should_answer_command_1() {
        let system: ByzantineSystem = ByzantineSystem::setup(4, 0).await.into();
        let (tx, rx) = oneshot::channel::<Command>();
        system.send_command(Command::Testing(None), (PeerType::Client, 0));
        system.send_command(Command::Testing(None), (PeerType::Client, 1));
        system.send_command(Command::Testing(None), (PeerType::Client, 2));
        system.send_command(Command::Testing(None), (PeerType::Client, 3));
        system.send_command(Command::Testing(Some(tx)), (PeerType::Client, 0));
        if let Command::Answer = rx.await.unwrap() {
            println!("Test completed!");
        } else {
            panic!();
        }
    }

    #[tokio::test]
    async fn client_should_answer_command_2() {
        let system: ByzantineSystem = ByzantineSystem::setup(2, 2).await.into();
        let (tx1, rx1) = oneshot::channel::<Command>();
        let (tx2, rx2) = oneshot::channel::<Command>();
        let _ = system.send_command(Command::Testing(Some(tx1)), (PeerType::Client, 0));
        let _ = system.send_command(Command::Testing(Some(tx2)), (PeerType::Client, 1));
        // tokio::join!(t1, t2);
        let mut count = 0;
        if let Command::Answer = rx1.await.unwrap() {
            count += 1;
        }

        if let Command::Answer = rx2.await.unwrap() {
            count += 1;
        }

        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn client_database_registers_correctly() {
        let system: ByzantineSystem = ByzantineSystem::setup(4, 0).await.into();
        for i in 0..4 {
            let _ = system.send_command(Command::Testing(None), (PeerType::Client, i));
        }

    }

    #[tokio::test]
    async fn broadcast_test(){
        let fuse = Fuse::new();
        for _ in 0..10 {
            fuse.spawn(async move {
                tokio::time::sleep(Duration::from_secs(10)).await;
            });
        }
    }


}
