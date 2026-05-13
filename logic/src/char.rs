#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharId{
    pub clock: u64,
    pub replica_id: u64,
}

// self = value of CharId, Self = type CharId.
// Returns Option<CharId>
impl PartialOrd for CharId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// compare clock first, then replica_id
impl Ord for CharId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.clock
            .cmp(&other.clock)
            .then(self.replica_id.cmp(&other.replica_id))
    }
}

pub struct RgaChar{
    pub id: CharId,
    pub origin: Option<CharId>,  //parent id
    pub value: char,
    pub deleted: bool,
}