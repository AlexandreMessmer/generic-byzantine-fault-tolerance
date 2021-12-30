use std::{collections::BTreeSet, time::SystemTime};

use talk::{crypto::Identity, unicast::Acknowledger};

use crate::{
    database::{self, replica_database::ReplicaDatabase},
    network::NetworkInfo,
    peer::{peer::PeerId, coordinator::{ProposalSignedData, ProposalData}},
    talk::{Command, Instruction, Message, Phase, RoundNumber},
    types::*,
};


use super::{communicator::Communicator, Handler};

pub struct ReplicaHandler {
    communicator: Communicator<Message>,
    proposal_inlet: MPSCSender<ProposalSignedData>,
    proposal_outlet: BroadcastReceiver<ProposalData>,
    database: ReplicaDatabase,
}

impl ReplicaHandler {
    pub fn new(communicator: Communicator<Message>, 
        proposal_inlet: MPSCSender<ProposalSignedData>,
        proposal_outlet: BroadcastReceiver<ProposalData>) -> Self {
        ReplicaHandler {
            communicator,
            proposal_inlet,
            proposal_outlet,
            database: ReplicaDatabase::new(),
        }
    }

    fn handle_command(&mut self, command: Command) {
        self.database.receive_command(command);
    }

    /// Implements task 1b and 1c
    fn handle_command_broadcast(
        &mut self,
        command: Command,
        round: RoundNumber,
        set: BTreeSet<Command>,
        phase: Phase,
    ) {
        if round.eq(self.database.round()) {
            self.database.receive_set(&set);
        }
    }
}

#[async_trait::async_trait]
impl Handler<Message> for ReplicaHandler {
    async fn handle_message(&mut self, _id: Identity, message: Message, _ack: Acknowledger) {
        match message {
            Message::Testing => {
                println!("Replica #{} received the test", self.communicator.id())
            }
            Message::Command(command) => self.handle_command(command),
            Message::BroadcastCommand(command, k, set, phase) => {}
            _ => {}
        }
    }
    async fn handle_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Testing => {
                println!("Replica #{} received the test", self.communicator.id())
            }
            _ => {}
        }
    }

    fn id(&self) -> &PeerId {
        self.communicator.id()
    }

    fn network_info(&self) -> &NetworkInfo {
        self.communicator.network_info()
    }
}
