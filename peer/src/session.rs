use crate::protocol::Message;
use crate::transport::{recv, send};
use anyhow::Result;
use logic::doc::Document;
use logic::op::Op;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

pub async fn run(
    stream: TcpStream,
    doc: Arc<Mutex<Document>>,
    my_client_id: u64,
    mut local_ops_rx: mpsc::Receiver<Op>,
    is_listener: bool,
    delay_ms: u64,
) -> Result<()> {
    let (mut r, mut w) = stream.into_split();

    // Handshake
    send(&mut w, &Message::Hello { client_id: my_client_id }).await?;
    let Message::Hello { client_id: peer_id } = recv(&mut r).await? 
    else {
        anyhow::bail!("expected Hello");
    };
    println!("connected to client {peer_id}");

    // Listener send current state (op's); joiner waits for it
    if is_listener{
        let snapshot: Vec<Op> = {
            let d = doc.lock().await;
            d.log.iter().cloned().collect()
        };
        send(&mut w, &Message::Snapshot { ops: snapshot }).await?;
    } else if let Message::Snapshot { ops } = recv(&mut r).await? {
        let mut d = doc.lock().await;
        for op in ops { d.remote_apply(op); }
        println!("synced {} chars", d.rga.to_string().len());
    }

    
    // Outbound: give local op's to peer
    let _ = tokio::spawn(async move{
        while let Some(op) = local_ops_rx.recv().await {
            if delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            }
            if send(&mut w, &Message::Op(op)).await.is_err() { break; }
        }
    });
    // Inbound: apply remote op's
    loop{
        match recv(&mut r).await? {
            Message::Op(op) => {
                let mut d = doc.lock().await;
                d.remote_apply(op);
            }
            Message::Snapshot { .. } | Message::Hello { .. } => {}
        }
    }
}
