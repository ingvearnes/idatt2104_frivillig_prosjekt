use crate::protocol::Message;
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

pub async fn send(w: &mut OwnedWriteHalf, msg: &Message) -> Result<()>{
    let bytes = bincode::serialize(msg)?;
    w.write_u32(bytes.len() as u32).await?;
    w.write_all(&bytes).await?;
    w.flush().await?;
    Ok(())
}

pub async fn recv(r: &mut OwnedReadHalf) -> Result<Message>{
    let len = r.read_u32().await? as usize;
    let mut buf = vec! [0u8; len];
    r.read_exact(&mut buf).await?;
    Ok(bincode::deserialize(&buf)?)
}