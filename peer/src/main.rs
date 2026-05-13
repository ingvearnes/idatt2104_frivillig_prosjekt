// cargo run -p peer -- --replica-id 1 listen
// cargo run -p peer -- --replica-id 2 connect 127.0.0.1:9000

mod protocol;
mod session;
mod transport;

use anyhow::Result;
use clap::{Parser, Subcommand};
use logic::doc::Document;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};

#[derive(Parser)]
struct Cli{
    // Unique id for this peer
    #[arg(long)]
    replica_id: u64,
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode{
    Listen{ #[arg(long, default_value = "0.0.0.0:9000")] addr: String},
    Connect { addr: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let doc = Arc::new(Mutex::new(Document::new()));

    let (_local_tx, local_rx) = mpsc::channel(64);
    // Spawn worker thread later

    match cli.mode{
        Mode::Listen { addr } => {
            let l = TcpListener::bind(&addr).await?;
            println!("listening on {addr}");
            let (stream, who) = l.accept().await?;
            print!("peer from {who}");
            session::run(stream, doc, cli.replica_id, local_rx, true).await?;
        }
        Mode::Connect { addr } => {
            let stream = TcpStream::connect(&addr).await?;
            session::run(stream, doc, cli.replica_id, local_rx, false).await?;
        }
    }
    Ok(())
}
