use paho_mqtt as mqtt;
use serde::{Deserialize, Serialize};
use std::process;
use std::time::SystemTime;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    command: Option<Command>,
    uri: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct Region {
    region_id: String,
    #[serde(rename = "type")]
    type_: String,
}

impl Region {
    fn from_id(id: u64) -> Self {
        Self {
            region_id: id.to_string(),
            type_: "rid".to_string(),
        }
    }
}

impl std::str::FromStr for Region {
    type Err = std::num::ParseIntError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(src, 10).map(|id| Region::from_id(id))
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[derive(StructOpt)]
struct StartRegions {
    pmap_id: String,
    user_pmapv_id: String,
    #[structopt(long, parse(from_flag))]
    ordered: i64,
    regions: Vec<Region>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[derive(StructOpt)]
enum Command {
    Start,
    #[serde(skip)]
    StartRegions(StartRegions),
    Clean,
    Pause,
    Stop,
    Resume,
    Dock,
    Evac,
    Train,
}

impl Command {
    fn into_command_with_extra(self) -> (Self, Option<Extra>) {
        match self {
            Command::StartRegions(x) => (Command::Start, Some(Extra::StartRegions(x))),
            x => (x, None),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", untagged)]
enum Extra {
    StartRegions(StartRegions),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", untagged)]
enum Message {
    Cmd {
        command: Command,
        time: u64,
        initiator: String,
        #[serde(flatten)]
        extra: Option<Extra>,
    },
    Delta,
}

impl Message {
    fn new_command(command: Command, extra: Option<Extra>) -> Self {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self::Cmd {
            command,
            time,
            initiator: "localApp".to_string(),
            extra,
        }
    }

    fn topic(&self) -> &'static str {
        match self {
            Self::Cmd { .. } => "cmd",
            _ => todo!(),
        }
    }

    fn payload(&self) -> String {
        serde_json::to_string(self).expect("serialization failed")
    }
}

fn send_message(cli: &mqtt::Client, message: &Message) -> mqtt::Result<()> {
    cli.publish(
        mqtt::MessageBuilder::new()
            .topic(message.topic())
            //.payload(format!("{{\"command\":\"dock\",\"time\":{},\"initiator\":\"localApp\"}}", 0))
            .payload(message.payload())
            .qos(0)
            .finalize(),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::from_args();

    // Create a client & define connect options
    let opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(cli.uri)
        .finalize();

    // Create a client & define connect options
    let mut client = mqtt::Client::new(opts).unwrap_or_else(|e| {
        println!("Error creating the client: {:?}", e);
        process::exit(1);
    });

    let ssl_opts = mqtt::SslOptionsBuilder::new()
        .enable_server_cert_auth(false)
        .finalize();

    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .ssl_options(ssl_opts)
        .user_name(cli.username)
        .password(cli.password)
        .finalize();

    let rx = client.start_consuming();
    client.connect(conn_opts)?;

    match cli.command {
        Some(command) => {
            let (command, extra) = command.into_command_with_extra();
            let message = Message::new_command(command, extra);

            send_message(&client, &message)?;
        }
        None => {
            for msg in rx.iter() {
                if let Some(msg) = msg {
                    println!("{}", msg);
                }
            }
        }
    }

    // Disconnect from the broker
    client.disconnect(None).unwrap();

    Ok(())
}
