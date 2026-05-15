//! This file is responsible for the merge algorithm in the case of a conflict
use serde::{Serialize, Deserialize};
use crate::char::{CharId, RgaChar};

/// Replicated Growable Array with characters in it
#[derive(Serialize, Deserialize)]
pub struct Rga{
    pub chars: Vec<RgaChar>, //all chars in doc (including deleted one without garbage collection)
}
impl Rga{
    /// Constructor
    pub fn new() -> Self {
        Self{chars: Vec::new()}
    }

    /// Simple rga-merge. It has O(n**2)
    pub fn apply_insert(&mut self, new_char: RgaChar){
        // Start scanning and set beginning index, or right after parents char
        let start = match &new_char.origin {
            None => 0,
            Some(parent_id) => { //parent_id is our naming of the value already inside Some()
                self.chars.iter()
                    .position(|c| &c.id == parent_id) // |c| &c.id == parant_id means for each element |c| check if c.id equals parent_id. It returns Option<usize> (search might fail, so gives Some(index) or None)                       
                    .expect("parent must exits") //unwraps Option<usize> and gets index-value
                    + 1
            }
        };

        let mut pos = start;
        // Check if char is sibling or in subtree 
        while pos < self.chars.len() {
            let exsisting = &self.chars[pos];

            // If chars are sibling, tie-break: our char continues walk vector until it meets a lesser id of a char. Line 52 appends it right before this lesser char
            if exsisting.origin == new_char.origin { // Same origin means same parent. Option<CharId> == Option<CharId> --> Some(V) == Some(V)
                if exsisting.id > new_char.id{
                    pos = pos + 1;
                } else{
                    break;
                }
            } else{ // if siblings type at same time
                // in_subtree becomes a bool: as_ref -> Option<&CharId> to turn origin into reference (can't move origin itself out of Option<>)
                let in_subtree = exsisting.origin.as_ref().is_some_and(|o| { // is_some_and() -> true if Option is "Some(o)"
                    self.chars[start..pos].iter().any(|c| &c.id == o) // Check if any previuously char has Id equal to origin
                });
                if in_subtree {
                    pos += 1;
                } else{
                    break;
                }
            }
        }
        self.chars.insert(pos, new_char);
    }

    /// Tombstone-applier
    pub fn apply_delete(&mut self, id: &CharId){
        if let Some(c) = self.chars.iter_mut().find(|c| &c.id == id){
            c.deleted = true;
        }
    }

    /// Build visible string
    pub fn to_string(&self) -> String{
        self.chars.iter()
            .filter(|c| !c.deleted)
            .map(|c| c.value)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::char::{CharId, RgaChar};

    fn id(counter: u64, client_id: u64) -> CharId{
        CharId { counter, client_id }
    }
    fn ch(counter: u64, client_id: u64, origin: Option<CharId>, value: char) -> RgaChar {
        RgaChar { id: id(counter, client_id), origin, value, deleted: false } 
    }

    #[test]
    fn insert_single_char(){
        let mut rga = Rga::new();
        rga.apply_insert(ch(1, 1, None, 'a'));
        assert_eq!(rga.to_string(), "a");
    }

    #[test]
    fn insert_after_exsisting_builds_word(){
        let mut rga = Rga::new();
        rga.apply_insert(ch(1, 1, None, 'h'));
        rga.apply_insert(ch(2, 1, Some(id(1,1)), 'i'));
        assert_eq!(rga.to_string(), "hi");
    }

    #[test]
    fn delete_tombstones_char_but_keeps_it_in_vec(){
        let mut rga = Rga::new();
        rga.apply_insert(ch(1, 1, None, 'a'));
        rga.apply_delete(&id(1, 1));
        assert_eq!(rga.to_string(), "");
        assert!(rga.chars[0].deleted); //still physically present though
    }

    #[test]
    fn concurrent_insert_higher_id_wins_left_position(){
        //higher charid is put left
        let mut rga = Rga::new();
        rga.apply_insert(ch(1, 1, None, 'a'));
        rga.apply_insert(ch(2, 2, Some(id(1,1)), 'x'));
        rga.apply_insert(ch(2, 1, Some(id(1,1)), 'y'));
        assert_eq!(rga.to_string(), "axy")
    }
}