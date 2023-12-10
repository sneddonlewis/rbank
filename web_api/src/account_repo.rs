use std::sync::Arc;
use async_trait::async_trait;
use crate::view_models::Account;

pub type DynAccountRepo = Arc<dyn AccountRepo + Send + Sync>;

pub type AccountRepoError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[async_trait]
pub trait AccountRepo {
    async fn find(&self, card_num: String) -> Result<Account, AccountRepoError>;

    async fn create(&self) -> Result<Account, AccountRepoError>;
}

pub struct AccountRepoImpl;

#[async_trait]
impl AccountRepo for AccountRepoImpl {
    async fn find(&self, card_num: String) -> Result<Account, AccountRepoError> {
        Ok(Account::new(card_num, "1111".to_string(), 0.0))
    }

    async fn create(&self) -> Result<Account, AccountRepoError> {
        Ok(Account::new("4000001111111111".to_string(), "1111".to_string(), 0.0))
    }
}
