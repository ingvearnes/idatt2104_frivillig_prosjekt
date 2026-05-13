use crate::merge::Rga;
use crate::op::{Op, OpLog};
use crate::char::{CharId, RgaChar};

pub struct Document{
    pub rga: Rga,
    pub log: OpLog,
}
impl Document{
    pub fn new() -> Self{
        Self{rga: Rga::new(), log: OpLog::new()}
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
        match &op{
            Op::Insert{ c } => self.rga.apply_insert(c.clone()),
            Op::Delete{ id } => self.rga.apply_delete(id),
        }
        self.log.append(op);
    }
}