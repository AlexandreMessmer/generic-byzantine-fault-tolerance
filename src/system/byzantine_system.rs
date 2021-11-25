use crate::crypto::identity_table::{IdentityTable};
use crate::system::peer::{Peer, PeerId};
use crate::talk::command::Command;
use crate::talk::message::Message;
use talk::crypto::Identity;
use talk::sync::fuse::Fuse;
use talk::unicast::test::UnicastSystem;
use talk::unicast::{Receiver as TalkReceiver, Sender as TalkSender};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::mpsc::Sender as MPSCSender;

use super::client::Client;
use super::peer_runner::PeerRunner;
use super::replica::Replica;
use super::settings::Settings;
use super::{PeerIdentifier, PeerType};

type Sender = TalkSender<Message>;
type Receiver = TalkReceiver<Message>;
/// Use a PeerSystem as model.
pub struct ByzantineSystem {
    settings: Settings,
    client_inlets: Vec<MPSCSender<Command>>,
    replica_inlets: Vec<MPSCSender<Command>>,
    pub fuse: Fuse,
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

        let clients: Vec<Client> = ByzantineSystem::compute_runner(
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

        let replicas: Vec<Replica> = ByzantineSystem::compute_runner(
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

    pub async fn send_command(&self, command: Command, (kind, id): PeerIdentifier) {
        let inlet = match kind {
            PeerType::Client => self.client_inlet(id),
            PeerType::Replica => self.replica_inlet(id),
        };
        if let Some(inlet) = inlet {
            tokio::spawn(async move {
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

    fn compute_runner(
        nbr: usize,
        keys: Vec<Identity>,
        senders: Vec<Sender>,
        receivers: Vec<Receiver>,
        outlets: Vec<MPSCReceiver<Command>>,
        keys_table: Vec<Identity>,
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
                )
            })
            .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_test() {
        let mut array = vec![1, 2, 3, 4, 5];
        let p1 = vec![array.pop(), array.pop()];
        let array = array.into_iter();
        println!("Array : {:?}, p1 : {:?}", array, p1);
    }
}
