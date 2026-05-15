//! The wire protocol with different message types
use logic::op::Op;
use serde::{Deserialize, Serialize};

/// The message over the network
#[derive(Debug, Serialize, Deserialize)]
pub enum Message{
    // First message sent after TCP connect. Client_id with it
    Hello{ client_id: u64 },
    // Full op history, from listener after connect
    Snapshot { ops: Vec<Op> },
    // Local op applied remotely
    Op(Op),
}