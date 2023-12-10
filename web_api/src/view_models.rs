use serde::{Deserialize, Serialize};

pub struct Account {
    pub card_number: String,
    pub pin: String,
    pub balance: f64,
}

impl Account {
    // pub fn new() -> Account {
    //     Account {
    //         card_number: "4000001111111111".to_string(),
    //         pin: "1111".to_string(),
    //         balance: 0.0
    //     }
    // }
    pub fn new(card_number: String, pin: String) -> Account {
        Account {
            card_number,
            pin,
            balance: 0.0
        }
    }
}

impl From<Account> for AccountDetailView {
    fn from(value: Account) -> Self {
        AccountDetailView {
            card_number: value.card_number,
            balance: value.balance,
        }
    }
}

impl From<Account> for AccountAuthView {
    fn from(value: Account) -> Self {
        AccountAuthView {
            card_number: value.card_number,
            pin: value.pin,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountAuthView {
    pub card_number: String,
    pub pin: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccountDetailView {
    pub card_number: String,
    pub balance: f64,
}