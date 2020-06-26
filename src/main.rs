mod api;
mod cli;

use paho_mqtt as mqtt;
use std::collections::HashSet;
use std::io::Write;
use std::net::{UdpSocket, TcpStream, ToSocketAddrs};
use std::process;
use structopt::StructOpt;
use std::io::Read;
use openssl::ssl::{SslMethod, SslConnector, SslVerifyMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::net::ToSocketAddrs;
    sign_up("10.10.0.90:8883".to_socket_addrs().unwrap().next().unwrap())?;
    std::process::exit(0);
    let cli = cli::Cli::from_args();

    match cli.command {
        cli::AnyCommand::Unauthenticated(cli::UnauthenticatedCommand::FindIp) => {
            find_ip_address()?;
        }
        cli::AnyCommand::Authenticated(cli) => {
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
                    let message = api::Message::new_command(command, extra);

                    message.send_message(&client)?;
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
        }
    }

    Ok(())
}

fn find_ip_address() -> std::io::Result<()> {
    let mut found = HashSet::new();
    let mut stdout = std::io::stdout();
    let packet = b"irobotmcs";
    let socket = UdpSocket::bind("0.0.0.0:5678")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(std::time::Duration::from_secs(3)))?;
    let mut data = [0; 800];

    loop {
        socket.send_to(&packet[..], "255.255.255.255:5678").unwrap();
        loop {
            if let Ok(length) = socket.recv(&mut data) {
                if &data[..length] != packet {
                    if let Ok(info) = serde_json::from_slice::<api::Info>(&data[..length]) {
                        if !found.contains(&info.ip) {
                            let _ = writeln!(
                                stdout,
                                "found.\nHostname: {}\nIP: {}\nblid/robot_id/username: {}",
                                info.hostname, info.ip, info.robot_id
                            );
                            found.insert(info.ip);
                        }
                    }
                }
            }

            let mut fh = stdout.lock();
            let _ = fh.write(b".");
            let _ = fh.flush();
        }
    }
}

fn sign_up<A: ToSocketAddrs + std::fmt::Display>(addr: A) -> std::io::Result<()> {
    let packet: &[u8] = &[0xf0, 0x05, 0xef, 0xcc, 0x3b, 29, 00];

    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    //builder.set_read_ahead(true);
    //builder.set_mode(openssl::ssl::SslMode::ENABLE_PARTIAL_WRITE);
    //builder.set_cipher_list("AES128-SHA256").unwrap();
    let connector = builder.build();
    let tcp_stream = TcpStream::connect(addr)?;
    let mut stream = connector.connect("10.10.0.90", tcp_stream).unwrap();

    println!("handshake complete");

    stream.write(&packet)?;
    stream.flush()?;
    //let mut plaintext = vec![];
    //stream.read_to_end(&mut plaintext).unwrap();
    loop {
        let mut plaintext = [0; 1000];
        println!("reading...");
        let length = stream.read(&mut plaintext).unwrap();

        println!("{:?}", plaintext[..length].to_vec());
        println!("{}", String::from_utf8_lossy(&plaintext[..length]));
        break;
    }

    Ok(())
}
