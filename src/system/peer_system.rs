use talk::sync::fuse::Fuse;
use talk::unicast::test::UnicastSystem;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender as MPSCSender;
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use crate::system::peer::Peer;
use crate::system::Command;

use super::peer_runner::PeerRunner;

/// Define a system of peers. Command can be given to peer through the MPSC channel.
/// 
/// Each peer runs a `PeerRunner` that handles incoming request. We can pass information to each peer through 
/// message handling (see https://tokio.rs/tokio/tutorial/channels).
pub struct PeerSystem {
    size: usize,
    runner_inlets: Vec<MPSCSender<Command>>,
    pub fuse: Fuse,
}

impl PeerSystem {

    /// Setup a new system of peers. Each peer runs on a `PeerRunner`, that handle the incoming messages (from the runner 
    /// channel and the talk messages)
    /// 
    /// The system of peers has size [`peers`]
    pub async fn setup(peers: usize) -> Self{
        let mut inlets: Vec<MPSCSender<Command>> = Vec::new();
        let mut outlets: Vec<MPSCReceiver<Command>> = Vec::new();
        for _ in 0..peers {
            let (tx, rx) = mpsc::channel::<Command>(32);
            inlets.push(tx);
            outlets.push(rx);
        }
        let inlets = inlets;
        let outlets = outlets;

        let UnicastSystem {
            keys,
            senders,
            receivers,
        } = UnicastSystem::setup(peers).await.into();

        let keys_table = keys.clone();
        let keys = keys.into_iter();
        let senders = senders.into_iter();
        let receivers = receivers.into_iter();
        let outlets = outlets.into_iter();

        let peer_runners: Vec<PeerRunner> = keys.zip(senders)
            .zip(receivers)
            .zip(outlets)
            .map(|(((key, sender), receiver), runner_outlet)| {
                let keys_table = keys_table.clone();
                let peer = Peer {key, sender, receiver};
                PeerRunner::new(peer, runner_outlet, keys_table)
            })
            .collect::<Vec<_>>();

        let fuse = Fuse::new();

        {
            for mut runner in peer_runners {
                fuse.spawn(async move{
                    let _ = runner.run().await;
                });
            }
        }
        

        PeerSystem {
            size: peers,
            runner_inlets: inlets,
            fuse,
        }
    }

    pub fn get_inlet(&self, peer_id: usize) -> Option<MPSCSender<Command>> {
        if peer_id < self.size {
            match self.runner_inlets.get(peer_id) {
                Some(inlet) => {
                    return Some(inlet.clone());
                },
                _ => {
                    return None;
                }
            }
        }

        return None;
    }

}

#[cfg(test)]
mod tests{

    use tokio::sync::mpsc::channel;

    use crate::system::{Command::Send, Message::{Plaintext}, peer_system::PeerSystem};

    use super::*;
    #[tokio::test]
    async fn basic_setup(){
        let system: PeerSystem = PeerSystem::setup(3).await.into();

        let inlet: MPSCSender<Command> = system.get_inlet(0).unwrap(); 
        let value: Command = Send(1, Plaintext(String::from("Hello")));
        let _ = inlet.send(value);
        println!("Complete");
        // Do something with the fuse ?
    }

    async fn handle_command(cmd: Command){
        match cmd {
            Command::Send(_, Plaintext(msg)) => println!("{}", msg),
            //_ => println!("Echec")
        }
    }
    #[tokio::test]
    async fn channel_test(){
        let (tx, mut rx) = channel::<Command>(32);
        
        let task = tokio::spawn(async move{
            loop {
                tokio::select! {
                    Some(cmd) = rx.recv() => {
                        handle_command(cmd).await;
                    }
                }
            }
        });

        let _ = tx.send(Command::Send(0, Plaintext(String::from("Hello")))).await;
        // Without the await, it is never executed
        task.await.unwrap();
    }
}