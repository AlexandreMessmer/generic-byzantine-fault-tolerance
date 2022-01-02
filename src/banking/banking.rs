use std::collections::HashMap;

use crate::{error::BankingError, peer::peer::PeerId};

pub type Money = u64;

/// Represents a simplified banking system, distributed over some sets of replicas.
/// Client can register to a Banking system, and store coins.
/// This can be compared to a saving account: a customer frequently
/// deposit money, and rarely withdraw it.
/// Each replica has its own banking instance.

pub struct Banking {
    clients: HashMap<PeerId, Money>,
}

impl Banking {
    pub fn new() -> Self {
        Banking {
            clients: HashMap::new(),
        }
    }

    /// Register a new client, and returns true if the client was not previously registered.
    pub fn register(&mut self, client: PeerId) -> bool {
        if !self.clients.contains_key(&client) {
            self.clients.insert(client, 0);
            return true;
        }

        false
    }

    /// Returns true if the client was correctly removed
    pub fn unregister(&mut self, client: &PeerId) -> bool {
        self.clients
            .remove(client)
            .map(|_client| true)
            .unwrap_or(false)
    }

    /// Deposit the amount into the client account. Returns true if the deposit is successful, false otherwise.
    pub fn deposit(&mut self, client: &PeerId, amount: Money) -> Result<(), BankingError> {
        self.clients
            .get_mut(client)
            .map(|current| {
                *current += amount;
                ()
            })
            .ok_or(BankingError::ClientNotFound)
    }

    /// Transfer an amount of coins from one client to another. Returns true if the transfer is successful (and feasible)
    fn transfer_to(&mut self, from: &PeerId, to: &PeerId, amount: Money) -> bool {
        if self.clients.contains_key(from) && self.clients.contains_key(to) {
            let from_amout = self.clients.get_mut(from).unwrap();
            if *from_amout >= amount {
                *from_amout -= amount;

                let to_amount = self.clients.get_mut(to).unwrap();
                *to_amount += amount;
                return true;
            }
        }
        false
    }

    pub fn withdraw(&mut self, client: &PeerId, amount: Money) -> Result<(), BankingError> {
        self.clients
            .get_mut(client)
            .map(|current| {
                if *current >= amount {
                    *current -= amount;
                    return Ok(());
                }
                Err(BankingError::UnsufficientBalance)
            })
            .unwrap_or(Err(BankingError::ClientNotFound))
    }

    pub fn get(&self, client: &PeerId) -> Option<Money> {
        self.clients.get(client).map(|value| *value)
    }
}

#[cfg(test)]
mod tests {

    use crate::banking;

    use super::*;

    #[test]
    fn register_test() {
        let mut banking = Banking::new();
        let identity = 1;
        assert_eq!(banking.register(identity.clone()), true);

        assert_eq!(banking.clients.contains_key(&identity), true);
        assert_eq!(*banking.clients.get(&identity).unwrap(), 0 as u64);

        assert_eq!(banking.register(identity), false);
    }

    #[test]
    fn deposit_test() {
        let mut banking = Banking::new();
        let identity = 1;

        if let Ok(_) = banking.deposit(&identity, 77) {
            panic!();
        }

        banking.clients.insert(identity.clone(), 30);

        if let Ok(_) = banking.deposit(&identity, 77) {
            assert_eq!(banking.get(&identity), Some(107));
        } else {
            panic!();
        }
    }

    #[test]
    fn transfer_test() {
        let mut banking = Banking::new();
        let client1 = 1;
        let client2 = 2;

        banking.clients.insert(client1.clone(), 30);

        assert_eq!(banking.transfer_to(&client1, &client2, 0), false);

        assert_eq!(banking.transfer_to(&client2, &client1, 11), false);
        banking.clients.insert(client2.clone(), 1);

        assert_eq!(banking.transfer_to(&client1, &client2, 20), true);

        assert_eq!(*banking.clients.get(&client1).unwrap(), 10);
        assert_eq!(*banking.clients.get(&client2).unwrap(), 21);

        assert_eq!(banking.transfer_to(&client1, &client2, 11), false);
        assert_eq!(banking.transfer_to(&client1, &client2, 10), true);
    }

    #[test]
    fn withdraw_test() {
        let mut banking = Banking::new();
        banking.clients.insert(1, 40);

        let w = banking.withdraw(&1, 30).unwrap();
        assert_eq!(banking.get(&1), Some(10));

        let w = banking.withdraw(&1, 11).unwrap_err();
        assert_eq!(w, BankingError::UnsufficientBalance);

        assert_eq!(
            banking.withdraw(&2, 0).unwrap_err(),
            BankingError::ClientNotFound
        );
    }

    #[test]
    fn get_test() {
        let mut banking = Banking::new();
        banking.clients.insert(1, 30);
        assert_eq!(banking.get(&1).unwrap(), 30);
        assert_eq!(banking.get(&2), None);
    }
}
