use crate::crypto::identity_table::{IdentityTable};
use crate::system::{Peer, PeerId};
use crate::talk::command::Command;
use crate::talk::message::Message;
use talk::crypto::Identity;
use talk::sync::fuse::Fuse;
use talk::unicast::test::UnicastSystem;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::mpsc::Sender as MPSCSender;

use super::client::Client;
use super::replica::Replica;
use super::settings::Settings;
use super::{PeerIdentifier, PeerType};
use super::*;

/// Use a PeerSystem as model.
pub struct ByzantineSystem {
    settings: Settings,
    client_inlets: Vec<MPSCSender<Command>>,
    replica_inlets: Vec<MPSCSender<Command>>,
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

        let clients: Vec<Client> = PeerRunner::compose_runners(
            nbr_clients,
            keys,
            senders,
            receivers,
            client_outlets,
            Vec::new(),
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
            settings: Settings::default_settings(nbr_clients, nbr_replicas),
            client_inlets,
            replica_inlets,
            fuse,
        }
    }

    pub fn send_command(&self, command: Command, (kind, id): PeerIdentifier) {
        let inlet = match kind {
            PeerType::Client => self.client_inlet(id),
            PeerType::Replica => self.replica_inlet(id),
        };
        if let Some(inlet) = inlet {
            self.fuse.spawn(async move {
                inlet.send(command).await.unwrap();
            });
        }
    }

    fn client_inlet(&self, target: PeerId) -> Option<MPSCSender<Command>> {
        ByzantineSystem::find_inlet(target, &self.client_inlets)
    }

    fn replica_inlet(&self, target: PeerId) -> Option<MPSCSender<Command>> {
        ByzantineSystem::find_inlet(target, &self.replica_inlets)
    }

    fn find_inlet(
        target: PeerId,
        inlets: &Vec<MPSCSender<Command>>,
    ) -> Option<MPSCSender<Command>> {
        if target < inlets.len() {
            return Some(inlets.get(target).unwrap().clone());
        }
        return None;
    }

    /// Create enought channels
    fn create_channels(size: usize) -> (Vec<MPSCSender<Command>>, Vec<MPSCReceiver<Command>>) {
        let mut inlets = Vec::<MPSCSender<Command>>::new();
        let mut outlets = Vec::<MPSCReceiver<Command>>::new();
        for _ in 0..size {
            let (tx, rx) = mpsc::channel::<Command>(32);
            inlets.push(tx);
            outlets.push(rx);
        }

        (inlets, outlets)
    }

}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::oneshot;

    use crate::{system, talk::feedback::Feedback};

    use super::*;

    #[test]
    fn take_test() {
        let mut array = vec![1, 2, 3, 4, 5];
        let p1 = vec![array.pop(), array.pop()];
        let array = array.into_iter();
        println!("Array : {:?}, p1 : {:?}", array, p1);
    }

    #[tokio::test]
    async fn client_should_answer_command_1(){
        let system: ByzantineSystem = ByzantineSystem::setup(4, 0).await.into();
        let (tx, rx) = oneshot::channel::<Command>();
        system.send_command(Command::Testing(None), (PeerType::Client, 0));
        system.send_command(Command::Testing(None), (PeerType::Client, 1));
        system.send_command(Command::Testing(None), (PeerType::Client, 2));
        system.send_command(Command::Testing(None), (PeerType::Client, 3));
        system.send_command(Command::Testing(Some(tx)), (PeerType::Client, 0));
        if let Command::Answer = rx.await.unwrap() {
            println!("Test completed!");
        }
        else {
            panic!();
        }
    }

    #[tokio::test]
    async fn client_should_answer_command_2(){
        let system: ByzantineSystem = ByzantineSystem::setup(2, 2).await.into();
        let (tx1, rx1) = oneshot::channel::<Command>();
        let (tx2, rx2) = oneshot::channel::<Command>();
        let t1 = system.send_command(Command::Testing(Some(tx1)), (PeerType::Client, 0));
        let t2 = system.send_command(Command::Testing(Some(tx2)), (PeerType::Client, 1));
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
    async fn client_database_registers_correctly(){
        let system: ByzantineSystem = ByzantineSystem::setup(4, 0).await.into();
        for i in 0..4 {
            let _ = system.send_command(Command::Testing(None), (PeerType::Client, i));
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
        for i in 0..4 {
            let (tx, rx) = oneshot::channel();
            let _ = system.send_command(Command::AskStatus(Message::Testing, tx), (PeerType::Client, i));
            let res = rx.await;
            println!("{:?}", res);
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
