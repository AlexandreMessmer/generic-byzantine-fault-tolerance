use talk::unicast::Message;

use super::{handler::Handler, runner::Runner};
use crate::{network::network_info::NetworkInfo, talk::Instruction, types::*};
pub type PeerId = usize;
pub struct Peer<T: UnicastMessage> {
    receiver: UnicastReceiver<T>,
    network_outlet: InstructionReceiver,
    handler: Box<dyn Handler<T>>,
}

/// Structure that defines a `Peer`, i.e. an entity that can send and receive `Message`
/// Its behavior is defined by a `PeerRunner`

impl<T> Peer<T>
where
    T: UnicastMessage,
{
    /// Create a new `Peer`from the given arguments
    ///
    /// This is essentially a single unit of a `UnicastSystem` from talk crate
    pub fn new(
        receiver: UnicastReceiver<T>,
        network_outlet: InstructionReceiver,
        handler: Box<dyn Handler<T>>,
    ) -> Self {
        Peer {
            receiver,
            network_outlet,
            handler,
        }
    }

    pub fn id(&self) -> &usize {
        self.handler.id()
    }

    pub fn network_info(&self) -> &NetworkInfo {
        self.handler.network_info()
    }
}

#[async_trait::async_trait]
impl<T> Runner for Peer<T>
where
    T: Message,
{
    async fn run(mut self) {
        let id = self.id().clone();
        let network_info = self.network_info().clone();
        println!("Peer #{}: running", id);

        let handler = &mut self.handler;
        loop {
            tokio::select! {
                (id, message, acknowledger) = self.receiver.receive() => {
                    handler.handle_message(id, message, acknowledger).await;
                }

                Some(instruction) = self.network_outlet.recv() => {
                    println!("[{:#?}] #{} received instruction", network_info.elapsed().unwrap() ,id);
                    match instruction {
                        Instruction::Shutdown => {
                            break;
                        },
                        _ => handler.handle_instruction(instruction).await,
                    }
                }
            }
        }

        println!("Peer #{}: shutdown", self.id())
    }
}
