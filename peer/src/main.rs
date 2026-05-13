// cargo run -p peer -- --replica-id 1 listen
// cargo run -p peer -- --replica-id 2 connect 127.0.0.1:9000

mod protocol;
mod session;
mod transport;

use anyhow::Result;
use clap::{Parser, Subcommand};
use logic::char::{CharId, RgaChar};
use logic::doc::Document;
use logic::op::Op;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
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
    
    // FOR LATER: Spawn worker thread 
    let (local_tx, local_rx) = mpsc::channel(64);
    let doc_clone = Arc::clone(&doc);
    let replica_id = cli.replica_id;

    tokio::spawn(async move {
        let stdin = BufReader::new(tokio::io::stdin());
        let mut lines = stdin.lines();
        let mut clock = 0u64;
        let mut prev: Option<CharId> = None; 

        while let Ok(Some(line)) = lines.next_line().await{
            let ops: Vec<Op> = {
                let mut d = doc_clone.lock().await;
                line.chars().chain(std::iter::once('\n')).map(|ch| {
                    clock += 1;
                    let id = CharId{ clock, replica_id };
                    let rga_char = RgaChar{ id: id.clone(), origin: prev.take(), value: ch, deleted: false};
                    prev = Some(id);
                    let op = Op::Insert { c: rga_char.clone() };
                    d.local_insert(rga_char);
                    op
                }).collect()
            };
            for op in ops{
                let _ = local_tx.send(op).await;
            }
        }
    });

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
