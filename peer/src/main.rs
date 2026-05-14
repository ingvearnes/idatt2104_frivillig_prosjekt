// Connect to same subnet. Use ifconfig/ipconfig and ipv4-adress WITH :9000 at the end
// cargo run -p peer -- listen
// cargo run -p peer -- connect 127.0.0.1:9000


mod protocol;
mod session;
mod transport;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use logic::char::{CharId, RgaChar};
use logic::doc::Document;
use logic::op::Op;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use std::io::{self, Write};

#[derive(Parser)]
struct Cli{
    // Unique id for this peer
    #[arg(long)]
    client_id: Option<u64>,
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
    let client_id = cli.client_id.unwrap_or_else(rand::random);

    tokio::task::spawn_blocking(move || {
        enable_raw_mode()?;
        let result = input_loop(doc_clone, local_tx, client_id);
        disable_raw_mode()?;
        result
    });

    let render_doc = Arc::clone(&doc);
    tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        loop {
            {
                let d = rt.block_on(render_doc.lock());
                print!("\x1b[H\x1b[J{}", d.rga.to_string());
                io::stdout().flush().unwrap();
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    match cli.mode{
        Mode::Listen { addr } => {
            let l = TcpListener::bind(&addr).await?;
            println!("listening on {addr}");
            let (stream, who) = l.accept().await?;
            print!("peer from {who}");
            session::run(stream, doc, client_id, local_rx, true).await?;
        }
        Mode::Connect { addr } => {
            let stream = TcpStream::connect(&addr).await?;
            session::run(stream, doc, client_id, local_rx, false).await?;
        }
    }
    Ok(())
}

fn input_loop(
    doc: Arc<Mutex<Document>>,
    local_tx: mpsc::Sender<Op>,
    client_id: u64,
) -> Result<()> {
    let rt = tokio::runtime::Handle::current();
    let mut counter = 0u64;
    let mut prev: Option<CharId> = None;

    loop{
        let Event::Key(KeyEvent { code, .. }) = read()? else{
            continue;
        };

        match code{
            KeyCode::Left => {
                let d = rt.block_on(doc.lock());
                prev = match &prev {
                    None => None, 
                    Some(cur) => {
                        let pos = d.rga.chars.iter().position(|c| &c.id == cur);
                        pos.and_then(|p| {
                            d.rga.chars[..p]
                                .iter()
                                .rev()
                                .find(|c| !c.deleted)
                                .map(|c| c.id.clone())
                        })
                    }
                };
                continue;
            }
            KeyCode::Right => {
                let d = rt.block_on(doc.lock());
                let next = match &prev {
                    None => d.rga.chars.iter().find(|c| !c.deleted),
                    Some(cur) => {
                        let pos = d.rga.chars.iter().position(|c| &c.id == cur);
                        pos.and_then(|p| d.rga.chars[p + 1..].iter().find(|c| !c.deleted))
                    }
                };
                if let Some(c) = next {
                    prev = Some(c.id.clone());
                }
                continue;
            }
            _ => {}
        }

        let op = match code {
            KeyCode::Char(ch) => {
                counter += 1;
                let id = CharId { counter, client_id };
                let rga_char = RgaChar{
                    id: id.clone(),
                    origin: prev.clone(),
                    value: ch,
                    deleted: false,
                };
                let op = Op::Insert { c: rga_char.clone() };
                let mut d = rt.block_on(doc.lock());
                d.local_insert(rga_char);
                prev = Some(id);
                op
            }
            KeyCode::Enter => {
                counter += 1;
                let id = CharId { counter, client_id };
                let rga_char = RgaChar{
                    id: id.clone(),
                    origin: prev.clone(),
                    value: '\n',
                    deleted: false,
                };
                let op = Op::Insert { c: rga_char.clone() };
                let mut d = rt.block_on(doc.lock());
                d.local_insert(rga_char);
                prev = Some(id);
                op
            }
            KeyCode::Backspace => {
                let Some(del_id) = prev.clone() else { continue };
                let op = Op::Delete { id: del_id.clone() };
                let mut d = rt.block_on(doc.lock());
                d.local_delete(del_id.clone());

                let pos = d.rga.chars.iter().position(|c| c.id == del_id);
                prev = pos.and_then(|p| {
                    d.rga.chars[..p]
                        .iter()
                        .rev()
                        .find(|c| !c.deleted)
                        .map(|c| c.id.clone())  
                });
                op
            }
            KeyCode::Esc => break,
            _ => continue,
        };

        if local_tx.blocking_send(op).is_err(){
            break;
        }
    }
    Ok(())
}
