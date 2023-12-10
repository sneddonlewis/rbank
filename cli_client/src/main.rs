mod models;
mod client;
mod error;

use std::str::FromStr;
use std::fmt::{Display, Formatter};
use std::io;
use reqwest::header::{AUTHORIZATION};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use crate::client::ApiClient;
use crate::models::{AccountAuthView, AccountDetailView};
use crate::error::{CommonResult, CommonError};

#[tokio::main]
async fn main() {
    println!("Rust Bank CLI client");
    let api_client = ApiClient::new();
    loop {
        let menu_choice = read_until_success(
            read_menu_command_from_stdin,
            menu_options(),
        );
        match menu_choice {
            MenuCommand::Exit => {
                exit();
            },
            MenuCommand::Login => {
                let token = login(&api_client).await.unwrap();
                let account = show_account(&api_client, token).await.unwrap();
                println!("{:?}", account);
            },
            MenuCommand::New => {
                let account = create_account(&api_client).await.unwrap();
                println!("{:?}", account);
            },
        };
    }
}

fn exit() -> ! {
    println!("bye");
    std::process::exit(0);
}

async fn show_account(client: &ApiClient, bearer_token: String) -> CommonResult<AccountDetailView> {
    let acc = client.account_detail(bearer_token)
        .await?;
    Ok(acc)
}

async fn login(client: &ApiClient) -> CommonResult<String>{
    let auth_request = AccountAuthView{
        card_number: "4000001111111111".to_string(),
        pin: "1111".to_string(),
    };

    let token = client.login(&auth_request)
        .await?;
    Ok(token)
}

async fn create_account(client: &ApiClient) -> CommonResult<AccountAuthView> {
    let account = client.create().await?;
    Ok(account)
}



fn menu_options() -> String {
    MenuCommand::iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_until_success<T>(
    read_input_fn: fn() -> CommonResult<T>,
    prompt: String,
) -> T {
    println!("{}", prompt);
    if let Ok(result) = read_input_fn() {
        result
    } else {
        println!("oops, try again");
        read_until_success(read_input_fn, prompt)
    }
}

fn read_menu_command_from_stdin() -> CommonResult<MenuCommand> {
    let input = read_u8_from_stdin()?;
    let cmd = MenuCommand::try_from(input)?;
    Ok(cmd)
}

fn read_u8_from_stdin() -> CommonResult<u8> {
    let mut input_string = String::new();
    io::stdin().read_line(&mut input_string)?;
    let result = u8::from_str(input_string.trim())?;
    Ok(result)
}

#[derive(EnumIter)]
enum MenuCommand {
    Exit,
    Login,
    New,
}

impl TryFrom<u8> for MenuCommand {
    type Error = CommonError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MenuCommand::Exit),
            1 => Ok(MenuCommand::Login),
            2 => Ok(MenuCommand::New),
            _ => Err(CommonError::try_from("oops".to_string()).unwrap())
        }
    }
}

impl Display for MenuCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MenuCommand::Exit => write!(f, "{}", "0. Exit"),
            MenuCommand::Login => write!(f, "{}", "1. Login"),
            MenuCommand::New => write!(f, "{}", "2. New"),
        }
    }
}
