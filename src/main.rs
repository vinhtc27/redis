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
        TcpListener::bind(&format!("127.0.0.1:{}", cli.port.unwrap_or(DEFAULT_PORT))).await?;

    let (config, master) = match cli.replicaof {
        Some(_) => (
            Config::new(cli.port, true),
            Some(format!(
                "{}:{}",
                cli.replicaof.clone().unwrap()[0],
                cli.replicaof.clone().unwrap()[1]
            )),
        ),
        None => (Config::new(cli.port, false), None),
    };

    server::run(listener, config, master, signal::ctrl_c()).await?;

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(name = "redis-server", version, author, about = "A Redis server")]
struct Cli {
    #[clap(long, value_names = ["PORT"])]
    port: Option<u16>,
    #[arg(long, num_args = 2, value_names = ["MASTER_HOST", "MASTER_PORT"])]
    replicaof: Option<Vec<String>>,
}
