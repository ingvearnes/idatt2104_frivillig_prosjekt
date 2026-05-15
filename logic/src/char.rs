//! A singel character and its ID to be used in the doc. 
//! Char needs to be uniquely identified for RGA to work
use serde::{Serialize, Deserialize};

/// Identifier for a char
/// 
/// If clock (insert-time) is identical to another, the replica_id will be tie-breaker -> first user wins
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharId{
    pub counter: u64, //incrementing counter
    pub client_id: u64, // (peer_id)
}

// self = value of CharId, Self = type CharId.
// Returns Option<CharId>
impl PartialOrd for CharId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other)) //must some Some, never None
    }
}
// compare clock first, then, if clock is equal, tie-break on replica_id 
impl Ord for CharId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.counter
            .cmp(&other.counter)
            .then(self.client_id.cmp(&other.client_id))
    }
}

/// Represent a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgaChar{
    pub id: CharId, //id
    // Option<> gives to possible values: Some(value) - there is a value; None - no value
    pub origin: Option<CharId>,  // charId for char left for new insert/old insert
    pub value: char, //actuall char
    pub deleted: bool, //tombstone flag
}