use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn generate_pin_id(node_id: usize, pin_identifier: &str) -> usize {
    let mut hasher = DefaultHasher::new();
    node_id.hash(&mut hasher);
    pin_identifier.hash(&mut hasher);
    hasher.finish() as usize
}
