use crate::char::{CharId, RgaChar};

#[derive(Debug, Clone)]
pub enum Op{
    Insert{ c: RgaChar },
    Delete{ id: CharId },
}
impl Op{
    pub fn id(&self) -> &CharId{
        match self{
            Op::Insert{ c } => &c.id,
            Op::Delete{ id } => id,
        }
    }
}

pub struct OpLog{
    ops: Vec<Op>,
}
impl OpLog{
    pub fn new() -> Self{
        Self{ops: Vec::new()}
    }
    pub fn append(&mut self, op: Op){
        self.ops.push(op);
    }
    pub fn len(&self) -> usize{
        self.ops.len()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Op>{
        self.ops.iter()
    }
    pub fn ops_since(&self, since: usize) -> &[Op]{
        &self.ops[since.min(self.ops.len())..]
    }
}