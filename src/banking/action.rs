use std::{cmp::Ordering, fmt::Display};

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
        match *self {
            Action::Register => write!(f, "Register"),
            Action::Get => write!(f, "Get current balance"),
            Action::Deposit(amount) => write!(f, "Deposit {}", amount),
            Action::Withdraw(amount) => write!(f, "Withdraw {}", amount),
        }
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
