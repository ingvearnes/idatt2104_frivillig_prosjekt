use logic::op::Op;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message{
    // First message sent after TCP connect. Replica_id with it
    Hello{ replica_id: u64 },
    // Full op history, from listener after connect
    Snapshort { ops: Vec<Op> },
    // Local op applied remotely
    Op(Op),
}