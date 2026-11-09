//! A named, index-addressed content registry (mod-extensibility foundation).
//!
//! Content families — blocks, items, recipes, and later mobs, particles,
//! effects, block entities and dimensions — register here and are addressed by
//! a dense index plus a namespaced name. This keeps content data-driven and
//! lets mods register new entries without touching the core game logic.

use std::collections::HashMap;

/// A registry of named entries, addressed by a dense `u32` index or by name.
pub struct Registry<T> {
    entries: Vec<T>,
    by_name: HashMap<String, u32>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            by_name: HashMap::new(),
        }
    }

    /// Register `entry` under `name`, returning its index. Names must be unique.
    pub fn register(&mut self, name: &str, entry: T) -> u32 {
        let idx = self.entries.len() as u32;
        self.entries.push(entry);
        self.by_name.insert(name.to_string(), idx);
        idx
    }

    /// Borrow the entry at `index`, if present.
    pub fn get(&self, index: u32) -> Option<&T> {
        self.entries.get(index as usize)
    }

    /// Look up an entry's index by name.
    pub fn find(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
    }

    /// Number of registered entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate over `(index, entry)`.
    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.entries.iter().enumerate().map(|(i, e)| (i as u32, e))
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut r = Registry::new();
        let a = r.register("minecraft:stone", "stone");
        let b = r.register("minecraft:dirt", "dirt");
        assert_eq!(a, 0);
        assert_eq!(b, 1);
        assert_eq!(r.get(a), Some(&"stone"));
        assert_eq!(r.find("minecraft:dirt"), Some(b));
        assert_eq!(r.find("minecraft:missing"), None);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn iter_yields_indices() {
        let mut r = Registry::new();
        r.register("a", 1);
        r.register("b", 2);
        let v: Vec<(u32, &i32)> = r.iter().collect();
        assert_eq!(v, vec![(0, &1), (1, &2)]);
    }
}
