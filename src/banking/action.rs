use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::banking::Money;

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    Register,
    Get,
    Deposit(Money),
    Withdraw(Money),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            Action::Register => format!("Register"),
            Action::Get => format!("Get balance"),
            Action::Deposit(amount) => format!("Deposit {:5.5}", amount),
            Action::Withdraw(amount) => format!("Withdraw {:4.4}", amount),
        };

        write!(f, "{:<16.16}", str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comp() {
        println!("{:?}", Action::Register.cmp(&Action::Register));
        println!("{:?}", Action::Deposit(10).cmp(&Action::Withdraw(10)));
    }
}
