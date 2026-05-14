use std::collections::HashMap;
use crate::merge::Rga;
use crate::op::{Op, OpLog};
use crate::char::{CharId, RgaChar};
use serde::{Serialize, Deserialize};

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
    // Operations you apply, goes straight to RGA and log
    pub fn local_insert(&mut self, ch: RgaChar){
        self.rga.apply_insert(ch.clone());
        self.log.append(Op::Insert{ c: ch });
    }
    pub fn local_delete(&mut self, id: CharId){
        self.rga.apply_delete(&id);
        self.log.append(Op::Delete{ id });
    }
    // Operation from network
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
    // Checks if any operation is pending (no parent) -> recurse
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
    
    // de-duplicate operation (if two peers has same operation/message, ignore one)
    fn already_seen(&self, op: &Op) -> bool {
        self.log.iter().any(|e| match (e, op) {
            (Op::Insert { c: a }, Op::Insert { c: b }) => a.id == b.id,
            (Op::Delete { id: a }, Op::Delete { id: b }) => a == b,
            _ => false,
        })
    }
}


#[cfg(test)]
mod tests{
    use serde::de::value;

use super::*;
    use crate::char::{CharId, RgaChar};

    fn id(counter: u64, client_id: u64) -> CharId {
        CharId { counter, client_id }
    }

    fn rga_char(counter: u64, client_id: u64, origin: Option<CharId>, value: char) -> RgaChar{
        RgaChar { id: id(counter, client_id), origin, value, deleted: false }
    }

    fn insert_op(counter: u64, client_id: u64, origin: Option<CharId>, value: char) -> Op{
        Op::Insert { c: rga_char(counter, client_id, origin, value) }
    }

    #[test]
    fn local_insert_appears_in_text() {
        let mut doc = Document::new();
        doc.local_insert(rga_char(1, 1, None, 'h'));
        doc.local_insert(rga_char(2, 1, Some(id(1, 1)), 'i'));
        assert_eq!(doc.rga.to_string(), "hi");
    }

    #[test]
    fn local_delete_removes_char_from_text() {
        let mut doc = Document::new();
        doc.local_insert(rga_char(1, 1, None, 'a'));
        doc.local_delete(id(1, 1));
        assert_eq!(doc.rga.to_string(), "");
    }

    #[test]
    fn remote_apply_duplicate_is_ignored() {
        let mut doc = Document::new();
        let op = insert_op(1, 1, None, 'a');
        doc.remote_apply(op.clone());
        doc.remote_apply(op);
        assert_eq!(doc.rga.to_string(), "a");
        assert_eq!(doc.log.len(), 1);
    }

    #[test]
    fn remote_apply_out_of_order_resolves_when_parent_arrives() {
        let mut doc = Document::new();
        // 'b' depends on 'a', but arrives first
        doc.remote_apply(insert_op(2, 1, Some(id(1, 1)), 'b'));
        assert_eq!(doc.rga.to_string(), ""); // parked in pending
        doc.remote_apply(insert_op(1, 1, None, 'a'));
        assert_eq!(doc.rga.to_string(), "ab");
    }

    #[test]
    fn pending_chain_resolves_in_one_go() {
        let mut doc = Document::new();
        // c -> b -> a, but arrive in reverse order
        doc.remote_apply(insert_op(3, 1, Some(id(2, 1)), 'c'));
        doc.remote_apply(insert_op(2, 1, Some(id(1, 1)), 'b'));
        assert_eq!(doc.rga.to_string(), "");
        doc.remote_apply(insert_op(1, 1, None, 'a')); // triggers the whole chain
        assert_eq!(doc.rga.to_string(), "abc");
    }
}