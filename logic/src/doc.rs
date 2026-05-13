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
    pending: HashMap<CharId, Vec<Op>>, // key = missing parent id
}
impl Document{
    pub fn new() -> Self{
        Self{rga: Rga::new(), log: OpLog::new(), pending: HashMap::new() }
    }
    pub fn local_insert(&mut self, ch: RgaChar){
        self.rga.apply_insert(ch.clone());
        self.log.append(Op::Insert{ c: ch });
    }
    pub fn local_delete(&mut self, id: CharId){
        self.rga.apply_delete(&id);
        self.log.append(Op::Delete{ id });
    }
    pub fn remote_apply(&mut self, op: Op){
        if self.log.iter().any(|e| e.id() == op.id()){
            return;
        }
        if let Op::Insert { c: ref ch } = op {
            if let Some(ref parent_id) = ch.origin{
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
    pub fn apply_and_drain(&mut self, op: Op){
        let already_seen = self.log.iter().any(|existing| existing.id() == op.id());
        if already_seen{
            return;
        }
        let landed_id = op.id().clone();
        match &op{
            Op::Insert{ c } => self.rga.apply_insert(c.clone()),
            Op::Delete{ id } => self.rga.apply_delete(id),
        }
        self.log.append(op);

        if let Some(unblocked) = self.pending.remove(&landed_id){
            for waiting_op in unblocked{
                self.apply_and_drain(waiting_op);
            }
        }
    }
}