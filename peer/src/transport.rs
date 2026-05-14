use crate::protocol::Message;
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

// Sender of serialized bytes
pub async fn send(w: &mut OwnedWriteHalf, msg: &Message) -> Result<()>{
    let bytes = bincode::serialize(msg)?;
    w.write_u32(bytes.len() as u32).await?; //send length prefix (4 bytes) as a "intro"
    w.write_all(&bytes).await?; //send rest of payload
    w.flush().await?;
    Ok(())
}

// Receiver of serialized bytes
pub async fn recv(r: &mut OwnedReadHalf) -> Result<Message>{
    let len = r.read_u32().await? as usize; //read prefix and learn payloads size
    let mut buf = vec! [0u8; len]; //allocate exactly payload-size space
    r.read_exact(&mut buf).await?; 
    Ok(bincode::deserialize(&buf)?)
}