//! Representing document that will be used by each peer
use std::collections::HashMap;
use crate::merge::Rga;
use crate::op::{Op, OpLog};
use crate::char::{CharId, RgaChar};
use serde::{Serialize, Deserialize};

/// A singel document with updated operations for a synchronized text
#[derive(Serialize, Deserialize)]
pub struct Document{
    pub rga: Rga,
    pub log: OpLog, 
    #[serde(default)]
    pending: HashMap<CharId, Vec<Op>>, // a store for operations with missing parent id (still in flight)
}
impl Document{
    pub fn new() -> Self{
        Self{rga: Rga::new(), log: OpLog::new(), pending: HashMap::new() }
    }
    /// Operations you apply, goes straight to RGA and log
    pub fn local_insert(&mut self, ch: RgaChar){
        self.rga.apply_insert(ch.clone());
        self.log.append(Op::Insert{ c: ch });
    }
    pub fn local_delete(&mut self, id: CharId){
        self.rga.apply_delete(&id);
        self.log.append(Op::Delete{ id });
    }
    /// Operation from network
    pub fn remote_apply(&mut self, op: Op){
        if self.already_seen(&op){
            return;
        }
        if let Op::Insert { c: ref ch } = op {
            if let Some(ref parent_id) = ch.origin{
                // if insert does not have parent in rga (vector) yet, park in pending
                if !self.rga.chars.iter().any(|c| &c.id == parent_id){
                    self.pending
                        .entry(parent_id.clone())
                        .or_default()
                        .push(op);
                    return;
                }
            }
        }
        self.apply_and_drain(op);
    }
    /// Checks if any operation is pending (no parent) -> recurse
    pub fn apply_and_drain(&mut self, op: Op){
        //dedup guard: stop the recurson from making a double-apply
        if self.already_seen(&op){
            return;
        }
        let landed_id = op.id().clone(); //catch op's id before consuming it
        //apply operation to rga (vector)
        match &op{
            Op::Insert{ c } => self.rga.apply_insert(c.clone()),
            Op::Delete{ id } => self.rga.apply_delete(id),
        }
        self.log.append(op);

        // drain pending: remove landing_id and apply each waiting op (so their children get unblocked and we continue with the chain)
        if let Some(unblocked) = self.pending.remove(&landed_id){
            for waiting_op in unblocked{
                self.apply_and_drain(waiting_op);
            }
        }
    }
    
    /// de-duplicate operation (if two peers has same operation/message, ignore one)
    fn already_seen(&self, op: &Op) -> bool {
        self.log.iter().any(|e| match (e, op) {
            (Op::Insert { c: a }, Op::Insert { c: b }) => a.id == b.id,
            (Op::Delete { id: a }, Op::Delete { id: b }) => a == b,
            _ => false,
        })
    }
}