use crate::system::PeerId;
use crate::talk::Command;
use talk::sync::fuse::Fuse;
use talk::unicast::test::UnicastSystem;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::mpsc::Sender as MPSCSender;

use super::*;

impl PeerSystem {
    /// Setup a new system of peers. Each peer runs on a `PeerRunner`, that handle the incoming messages (from the runner
    /// channel and the talk messages)
    ///
    /// The system of peers has size [`peers`]
    pub async fn setup(peers: usize) -> Self {
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
        } = UnicastSystem::<Message>::setup(peers).await.into();

        let keys_table = keys.clone();

        let peer_runners: Vec<PeerRunner> = PeerRunner::compose_runners(
            peers,
            keys,
            senders,
            receivers,
            outlets,
            keys_table,
            RunnerSettings::default(),
        );

        let fuse = Fuse::new();

        {
            for mut runner in peer_runners {
                fuse.spawn(async move {
                    runner.run().await;
                });
            }
        }

        PeerSystem {
            size: peers,
            runner_inlets: inlets,
            fuse,
        }
    }

    pub async fn send_command(&self, command: Command, target: PeerId) {
        let inlet = self.get_inlet(target);
        if let Some(inlet) = inlet {
            tokio::spawn(async move {
                inlet.send(command).await;
            });
        }
    }
    fn get_inlet(&self, target: PeerId) -> Option<MPSCSender<Command>> {
        if target < self.size {
            if let Some(inlet) = self.runner_inlets.get(target) {
                return Some(inlet.clone());
            }
        }
        return None;
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::{system::peer_system::PeerSystem, talk::Message};

    use super::*;

    #[tokio::test]
    async fn basic_setup() {
        let system: PeerSystem = PeerSystem::setup(3).await.into();

        let inlet: MPSCSender<Command> = system.get_inlet(0).unwrap();
        let value: Command = Command::Send(1, Message::Plaintext(String::from("Hello")));
        let _ = inlet.send(value).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("____________________ END ________________");
        println!("Expected result: \n   Got: Hello");
        println!("_________________________________________");
    }

    // Observation: Never completes when sending two times to 1
    // Remark: A peer 1 cannot send message to itself -> Why ?
    #[tokio::test]
    async fn double_message() {
        use tokio::join;
        let system: PeerSystem = PeerSystem::setup(3).await.into();
        let t1 = system.send_command(
            Command::Send(2, Message::Plaintext(String::from("Hello"))),
            0,
        );
        let t2 = system.send_command(
            Command::Send(0, Message::Plaintext(String::from("Good bye !"))),
            1,
        );
        join!(t1, t2);
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Expected result: \n   Got: Hello \n   Got: Good bye !");
    }

    #[tokio::test]
    async fn double_messages_2() {
        let system: PeerSystem = PeerSystem::setup(3).await.into();
        let t1 = system.send_command(
            Command::Send(2, Message::Plaintext(String::from("Hello"))),
            0,
        );
        let t2 = system.send_command(
            Command::Send(2, Message::Plaintext(String::from("Good bye !"))),
            1,
        );
        tokio::join!(t1, t2);
        tokio::time::sleep(Duration::from_secs(5)).await;
        // How to make it wait until everything as completed ?
        // -> What about waiting response from a channel sent to everyone ?
        println!("Expected result: \n   Got: Hello \n   Got: Good bye !");
    }

    #[tokio::test]
    async fn message_to_itself() {
        let system: PeerSystem = PeerSystem::setup(1).await.into();
        system
            .send_command(
                Command::Send(0, Message::Plaintext(String::from("Hello myself"))),
                0,
            )
            .await;
        tokio::time::sleep(Duration::from_secs(4)).await;
        // Question: Why can't a peer send a message to itself ? (talk dependent)
    }
}
