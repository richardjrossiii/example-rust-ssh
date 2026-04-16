use serde::Deserialize;
use ssh2::Session;
use anyhow::Result;
use std::net::TcpStream;
use std::io::Read;

#[derive(Deserialize)]
struct Host {
    name: String,
    port: u16,
    command: String,
}

fn main() -> Result<()> {
    let process_args: Vec<_> = std::env::args().collect();
    let mut output = Vec::new();
    
    let input = std::fs::read_to_string(&process_args[1])?;
    let commands_to_process: Vec<Host> = serde_json::from_str(&input)?;

    for command in commands_to_process {
        let tcp = TcpStream::connect(format!("{}:{}", command.name, command.port)).unwrap();

        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        let mut channel = sess.channel_session().unwrap();
        channel.exec(&command.command).unwrap();
        channel.read_to_end(&mut output).unwrap();

        output.push(b'\n');
    }

    println!("{}", String::from_utf8_lossy(&output));

    Ok(())
}
