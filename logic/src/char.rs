#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharId{
    pub clock: u64,
    pub replica_id: u64,
}

pub struct RgaChar{
    pub id: CharId,
    pub origin: Option<CharId>,  //parent id
    pub value: char,
    pub deleted: bool,
}