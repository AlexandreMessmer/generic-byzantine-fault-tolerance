use std::time::Duration;

use genericbft::system::{peer_system::PeerSystem, command::Command, message::Message};
use tokio::sync::mpsc::Sender as MPSCSender;
#[tokio::main]
async fn main() {
    let system: PeerSystem = PeerSystem::setup(3).await.into();

    let inlet: MPSCSender<Command> = system.get_inlet(0).unwrap();
    let inlet2: MPSCSender<Command> = system.get_inlet(1).unwrap();
    let value: Command = Command::Send(1, Message::Plaintext(String::from("Hello")));
    let _ = inlet.send(value).await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let _ = inlet2
        .send(Command::Send(
            1,
            Message::Plaintext(String::from("Hello back")),
        ))
        .await;
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Observation: Sending two messages to the same peer make it fail.
    // Update: Send two messages to the same peer fails.
}
