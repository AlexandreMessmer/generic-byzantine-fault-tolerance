use crate::crypto::identity_table::{self, IdentityTable};
use crate::system::peer::{Peer};
use crate::talk::command::Command;
use crate::talk::message::Message;
use talk::crypto::Identity;
use talk::sync::fuse::Fuse;
use talk::unicast::{Receiver as TalkReceiver, Sender as TalkSender};
use talk::unicast::test::UnicastSystem;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::mpsc::Sender as MPSCSender;

use super::client::Client;
use super::peer_runner::PeerRunner;
use super::replica::Replica;
type Sender = TalkSender<Message>;
type Receiver = TalkReceiver<Message>;
/// Use a PeerSystem as model.
pub struct ByzantineSystem {
    nbr_client: usize, // TODO: Group into settings
    nbr_replica: usize,
    client_inlets: Vec<MPSCSender<Command>>,
    replica_inlets: Vec<MPSCSender<Command>>,
    pub fuse: Fuse,
}

impl ByzantineSystem {
    pub async fn setup(nbr_client: usize, nbr_replica: usize) -> Self {
        let (client_inlets, client_outlets) = ByzantineSystem::create_channels(nbr_client);
        let (replica_inlets, replica_outlets) = ByzantineSystem::create_channels(nbr_replica);

        let size = nbr_client + nbr_replica;
        let UnicastSystem {
            mut keys,
            mut senders,
            mut receivers
        } = UnicastSystem::<Message>::setup(size).await.into();

        let mut replica_keys = Vec::<Identity>::new();
        let mut replica_senders = Vec::<Sender>::new();
        let mut replica_receivers = Vec::<Receiver>::new();
        for _ in 0..nbr_replica {
            replica_keys.push(keys.pop().unwrap());
            replica_senders.push(senders.pop().unwrap());
            replica_receivers.push(receivers.pop().unwrap());
        }

        let identity_table = IdentityTable::new(keys.clone(), replica_keys.clone());

        let (client_keys, client_senders, client_receivers) = (keys.into_iter(), senders.into_iter(), receivers.into_iter());
        let (replica_keys, replica_senders, replica_receivers) = (replica_keys.into_iter(), replica_senders.into_iter(), replica_receivers.into_iter());
        let client_ids = (0..nbr_client).into_iter();
        let replica_ids = (0..nbr_replica).into_iter();

        let clients: Vec<Client> = client_ids
            .zip(client_keys)
            .zip(client_senders)
            .zip(client_receivers)
            .zip(client_outlets)
            .map(|((((id, key), sender), receiver), outlet)| {
                Client::new(Peer::new(id, key, sender, receiver), outlet, &identity_table)
            })
            .collect::<Vec<_>>();

        let replicas: Vec<Replica> = replica_ids
            .zip(replica_keys)
            .zip(replica_senders)
            .zip(replica_receivers)
            .zip(replica_outlets)
            .map(|((((id, key), sender), receiver), outlet)| {
                Replica::new(Peer::new(id, key, sender, receiver), outlet, &identity_table)
            })
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
            nbr_client,
            nbr_replica,
            client_inlets,
            replica_inlets,
            fuse,
        }
    }

    pub async fn send_command(&self, command: Command, target: PeerId) {
        let inlet = self.get_inlet(target);
        if let Some(inlet) = inlet {
            tokio::spawn(async move {
                inlet.send(command).await.unwrap();
            });
        }
    }

    fn replica_inlet(&self, target: usize) -> Option<MPSCSender<Command>> {
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
    use super::*;

    #[test]
    fn take_test(){
        let mut array = vec![1, 2, 3, 4, 5];
        let p1 = vec![array.pop(), array.pop()];
        let array = array.into_iter();
        println!("Array : {:?}, p1 : {:?}", array, p1);
    }
}
