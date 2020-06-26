use crate::api;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Cli {
    #[structopt(subcommand)]
    pub command: AnyCommand,
}

#[derive(StructOpt, Debug)]
pub enum AnyCommand {
    #[structopt(name = "command")]
    Authenticated(AuthenticatedCommand),
    #[structopt(flatten)]
    Unauthenticated(UnauthenticatedCommand),
}

#[derive(StructOpt, Debug)]
pub enum Command {
    #[structopt(flatten)]
    ApiCommand(api::Command),
    StartRegions(api::StartRegions),
}

#[derive(StructOpt, Debug)]
pub struct AuthenticatedCommand {
    #[structopt(subcommand)]
    pub command: Option<Command>,
    pub uri: String,
    pub username: String,
    pub password: String,
}

#[derive(StructOpt, Debug)]
pub enum UnauthenticatedCommand {
    FindIp,
    GetPassword { address: String },
}

impl Command {
    pub fn into_command_with_extra(self) -> (api::Command, Option<api::Extra>) {
        match self {
            Command::StartRegions(x) => (api::Command::Start, Some(api::Extra::StartRegions(x))),
            Command::ApiCommand(x) => (x, None),
        }
    }
}
