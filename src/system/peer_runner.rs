use talk::crypto::Identity;
use tokio::sync::mpsc::Receiver as MPSCReceiver;

use crate::system::Command;
use crate::system::peer::Peer;
use crate::system::Message::Plaintext;

use super::Message;

pub struct PeerRunner{
    peer: Peer,
    runner_outlet: MPSCReceiver<Command>,
    keys_table: Vec<Identity>,
}

impl PeerRunner {
    pub fn new(peer: Peer, runner_outlet: MPSCReceiver<Command>, keys_table: Vec<Identity>) -> Self{
        PeerRunner{
            peer,
            runner_outlet,
            keys_table,
        }
    }

    pub async fn run(&mut self){
        loop {
            tokio::select! {
                (id, message, _) = self.peer.receiver.receive() => {
                    println!("Reached");
                    self.handle_message(id, message).await;
                }

                Some(command) = self.runner_outlet.recv() => {
                    println!("Reached");
                    self.handle_command(command).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Send(id, message) => {
                let id = self.keys_table.get(id).unwrap().clone();
                self.peer.sender.send(id, message).await.unwrap();
                println!("Sent");
            }
        }
    }

    async fn handle_message(&mut self, _id: Identity, message: Message) {
        match message {
            Plaintext(str) => println!("{}", str)
        }
    }
}
