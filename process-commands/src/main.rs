use serde::Deserialize;
use ssh2::Session;
use anyhow::Result;
use std::net::TcpStream;
use std::io::Read;
use std::path::Path;
use anyhow::Context;

#[derive(Deserialize)]
struct Host {
    name: String,
    port: u16,
    command: String,
}

fn run_command(command: Host) -> Result<String> {
    let tcp = TcpStream::connect((command.name, command.port)).context("failed to connect")?;

    let mut output = Vec::new();
    let mut sess = Session::new().context("failed to create session")?;
    sess.set_tcp_stream(tcp);
    sess.handshake().context("failed to perform handshake")?;
    sess.userauth_pubkey_file(
        "root", 
        Some(Path::new("/root/.ssh/id_rsa.pub")), 
        Path::new("/root/.ssh/id_rsa"),
        None,
    ).unwrap();

    let mut channel = sess.channel_session().context("Failed to create channel")?;
    channel.exec(&command.command).context("Failed to exec")?;
    channel.read_to_end(&mut output).context("Failed to read output")?;

    String::from_utf8(output).context("invalid utf-8")
}

fn main() -> Result<()> {
    let process_args: Vec<_> = std::env::args().collect();
    let mut output = String::new(); 
    
    let input = std::fs::read_to_string(&process_args[1])?;
    let commands_to_process: Vec<Host> = serde_json::from_str(&input)?;

    for command in commands_to_process {
        let result = run_command(command)?;

        output += &result; 
    }

    println!("{}", output);

    Ok(())
}
