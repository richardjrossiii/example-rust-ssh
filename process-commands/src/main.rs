use anyhow::Context;
use anyhow::Result;
use async_ssh2_tokio::client::{AuthMethod, Client, ServerCheckMethod};
use futures::stream;
use futures::stream::StreamExt;
use itertools::Itertools;
use serde::Deserialize;

#[derive(Deserialize)]
struct Host {
    name: String,
    port: u16,
    command: String,
}

async fn run_command(command: &Host) -> Result<String> {
    let auth_method = AuthMethod::with_key_file("/root/.ssh/id_rsa", None);
    let client = Client::connect(
        (command.name.as_str(), command.port),
        "root",
        auth_method,
        ServerCheckMethod::NoCheck,
    )
    .await
    .context("failed to connect")?;

    let result = client
        .execute(&command.command)
        .await
        .context("failed to execute command")?;

    Ok(result.stdout.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let process_args: Vec<_> = std::env::args().collect();

    let input = std::fs::read_to_string(&process_args[1])?;
    let commands_to_process: Vec<Host> = serde_json::from_str(&input)?;

    let ouptuts = stream::iter(
        commands_to_process
            .iter()
            .chunk_by(|command| &command.name)
            .into_iter(),
    )
    .map(async |(_, commands)| {
        let mut output = String::new();
        for command in commands {
            let result = run_command(command).await?;

            output += &result;
        }

        Ok(output)
    })
    .buffer_unordered(128)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?;

    println!("{}", ouptuts.join("\n"));

    Ok(())
}
