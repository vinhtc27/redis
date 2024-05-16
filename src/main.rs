//! redis server.
//!
//! This file is the entry point for the server implemented in the library. It
//! performs command line parsing and passes the arguments on to
//! `redis::server`.
//!
//! The `clap` crate is used for parsing arguments.

use redis::{config::Config, server, DEFAULT_PORT};

use clap::Parser;
use tokio::net::TcpListener;
use tokio::signal;

#[tokio::main]
pub async fn main() -> redis::Result<()> {
    let cli = Cli::parse();

    let listener =
        TcpListener::bind(&format!("localhost:{}", cli.port.unwrap_or(DEFAULT_PORT))).await?;

    let (config, master) = match cli.replicaof {
        Some(replicaof) => {
            let parts: Vec<_> = replicaof.split_whitespace().collect();
            if parts.len() == 2 {
                let master_host = parts[0];
                let master_port = parts[1].parse::<u16>().unwrap_or_else(|_| {
                    std::process::exit(1);
                });
                (
                    Config::new(cli.port, true),
                    Some(format!("{}:{}", master_host, master_port)),
                )
            } else {
                std::process::exit(1);
            }
        }
        None => (Config::new(cli.port, false), None),
    };

    server::run(listener, config, master, signal::ctrl_c()).await?;

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(name = "redis-server", version, author, about = "A Redis server")]
struct Cli {
    #[clap(long, value_name = "PORT")]
    port: Option<u16>,
    #[arg(long, value_name = "REPLICAOF")]
    replicaof: Option<String>,
}
