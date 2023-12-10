use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use crate::error::CommonResult;
use crate::models::{AccountAuthView, AccountDetailView};

pub struct ApiClient {
    client: Client,
    base_uri: &'static str,
}

impl ApiClient {
    pub fn new() -> ApiClient {
        ApiClient {
            client: Client::new(),
            base_uri: "http://localhost:3000",
        }
    }

    pub async fn create(&self) -> CommonResult<AccountAuthView> {
        let uri = format!("{}/new", self.base_uri);
        let response = self.client
            .get(uri)
            .send()
            .await?;

        let account: AccountAuthView = response.json().await?;
        Ok(account)
    }

    pub async fn login(&self, request: &AccountAuthView) -> CommonResult<String> {
        let uri = format!("{}/login", self.base_uri);
        let response = self.client
            .post(uri)
            .json(request)
            .send()
            .await?;
        let token = response
            .headers()
            .get(AUTHORIZATION)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        Ok(token)
    }

    pub async fn account_detail(&self, bearer_token: String) -> CommonResult<AccountDetailView> {
        let uri = format!("{}/account", self.base_uri);
        let response = self.client
            .get(uri)
            .header(AUTHORIZATION, format!("Bearer {}", bearer_token))
            .send()
            .await?;

        let account: AccountDetailView = response.json().await?;
        Ok(account)
    }
}
