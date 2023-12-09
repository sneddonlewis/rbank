use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AccountView {
    pub card_number: String,
    pub pin: String,
}

impl AccountView {
    pub fn new() -> AccountView {
        AccountView {
            card_number: "4000001111111111".to_string(),
            pin: "1111".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccountDetailView {
    pub card_number: String,
    pub balance: f64,
}