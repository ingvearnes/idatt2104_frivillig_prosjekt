use crate::char::{CharId, RgaChar};

pub struct Rga{
    pub chars: Vec<RgaChar>,
}

impl Rga{
    pub fn new() -> Self {
        Self{chars: Vec::new()}
    }

    // Simple, but has O(n**2)
    pub fn apply_insert(&mut self, new_char: RgaChar){
        // Start scanning, either beginning or right after parent char
        let start = match &new_char.origin{
            None => 0,
            Some(parent_id) => {
                self.chars.iter()
                    .position(|c| &c.id == parent_id)
                    .expect("parent must exits") + 1
            }
        };

        let mut pos = start;
        // Check if char is sibling or in subtree 
        while pos < self.chars.len(){
            let exsisting = &self.chars[pos];

            // If chars are sibling: tie-break.
            if exsisting.origin == new_char.origin {
                if exsisting.id > new_char.id{
                    pos = pos + 1;
                } else{
                    break;
                }
            } else{
                // in_subtree becomes a bool: existing.origin -> Option<CharId>. as_ref -> Option<&CharId>. is_some_and() -> true if Option is "Some"
                let in_subtree = exsisting.origin.as_ref().is_some_and(|o| {
                    // Check if any previuously char has Id equal to o
                    self.chars[start..pos].iter().any(|c| &c.id == o)
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

    // Tombstone delete
    pub fn apply_delete(&mut self, id: &CharId){
        if let Some(c) = self.chars.iter_mut().find(|c| &c.id == id){
            c.deleted = true;
        }
    }

    // Build visible string
    pub fn to_string(&self) -> String{
        self.chars.iter()
            .filter(|c| !c.deleted)
            .map(|c| c.value)
            .collect()
    }
}