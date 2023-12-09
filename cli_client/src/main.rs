use std::str::FromStr;
use std::fmt::{Display, Formatter};
use std::io;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[tokio::main]
async fn main() {
    println!("Rust Bank CLI client");
    let token = create_account().await.unwrap();
    loop {
        let menu_choice = read_until_success(
            read_menu_command_from_stdin,
            menu_options(),
        );
        match menu_choice {
            MenuCommand::Exit => exit(),
            MenuCommand::Login => login(token.as_str()).await.unwrap(),
            MenuCommand::New => {
                let token = create_account().await.unwrap();
                println!("{}", token);
            },
        };
    }
}

fn exit() -> ! {
    println!("bye");
    std::process::exit(0);
}

async fn login(bearer_token: &str) -> CommonResult<()>{
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token).unwrap());

    let client = reqwest::Client::new();
    let uri = "http://localhost:3000/login";
    let response = client
        .get(uri)
        .headers(headers)
        .send()
        .await?
        .text()
        .await?;
    println!("{}", response);
    Ok(())
}

async fn create_account() -> CommonResult<String> {
    let client = reqwest::Client::new();
    let uri = "http://localhost:3000/new";
    let response = client
        .get(uri)
        .send()
        .await?;
    let token = response.headers().get(AUTHORIZATION).unwrap().to_str().unwrap().to_string();
    Ok(token)
}


type CommonError = Box<dyn std::error::Error>;
type CommonResult<T> = Result<T, CommonError>;

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
