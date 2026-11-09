//! Block entities: furnace and chest data, plus furnace smelting (M3).

use crate::item::{item_by_id, item_id, Item, ItemStack};

/// Chest container size.
pub const CHEST_SIZE: usize = 27;

/// A furnace's contents and its one-item smelting logic.
#[derive(Debug, Clone)]
pub struct Furnace {
    pub fuel: ItemStack,
    pub input: ItemStack,
    pub output: ItemStack,
}

impl Furnace {
    pub fn new() -> Self {
        Self {
            fuel: ItemStack::EMPTY,
            input: ItemStack::EMPTY,
            output: ItemStack::EMPTY,
        }
    }
}

impl Default for Furnace {
    fn default() -> Self {
        Self::new()
    }
}

/// A chest's stored items.
#[derive(Debug, Clone)]
pub struct Chest {
    pub items: [ItemStack; CHEST_SIZE],
}

impl Chest {
    pub fn new() -> Self {
        Self {
            items: [ItemStack::EMPTY; CHEST_SIZE],
        }
    }
}

impl Default for Chest {
    fn default() -> Self {
        Self::new()
    }
}

/// Block entity data, keyed by the block's world coordinates.
#[derive(Debug, Clone)]
pub enum BlockEntity {
    Furnace(Furnace),
    Chest(Chest),
}

/// The item produced by smelting `item`, or `None` if it is not smeltable.
pub fn smelt_result(item: Item) -> Option<Item> {
    match item_id(item) {
        Some("minecraft:cobblestone") => item_by_id("minecraft:stone"),
        Some("minecraft:sand") => item_by_id("minecraft:glass"),
        _ => None,
    }
}

/// Whether `item` is a valid furnace fuel.
pub fn is_fuel(item: Item) -> bool {
    matches!(
        item_id(item),
        Some("minecraft:oak_planks")
            | Some("minecraft:oak_log[axis=x]")
            | Some("minecraft:oak_log[axis=y]")
            | Some("minecraft:oak_log[axis=z]")
    )
}

/// Advance a furnace one step: if it has fuel and a smeltable input and an empty
/// (or stacking) output, consume one fuel + one input and produce one output.
pub fn tick_furnace(f: &mut Furnace) {
    if f.fuel.is_empty() || f.input.is_empty() {
        return;
    }
    let Some(out_item) = smelt_result(f.input.item) else {
        return;
    };
    // Output must be empty or the same item with room to stack.
    if !f.output.is_empty() && (f.output.item != out_item || f.output.count >= f.output.max_stack()) {
        return;
    }
    f.fuel.count -= 1;
    if f.fuel.count == 0 {
        f.fuel = ItemStack::EMPTY;
    }
    f.input.count -= 1;
    if f.input.count == 0 {
        f.input = ItemStack::EMPTY;
    }
    if f.output.is_empty() {
        f.output = ItemStack::new(out_item, 1);
    } else {
        f.output.count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn furnace_smelts_cobblestone_into_stone() {
        let mut f = Furnace::new();
        f.fuel = ItemStack::new(item_by_id("minecraft:oak_planks").unwrap(), 1);
        f.input = ItemStack::new(item_by_id("minecraft:cobblestone").unwrap(), 1);
        tick_furnace(&mut f);
        assert_eq!(f.output.item, item_by_id("minecraft:stone").unwrap());
        assert_eq!(f.output.count, 1);
        assert!(f.fuel.is_empty());
        assert!(f.input.is_empty());
    }

    #[test]
    fn furnace_needs_fuel() {
        let mut f = Furnace::new();
        f.input = ItemStack::new(item_by_id("minecraft:cobblestone").unwrap(), 1);
        tick_furnace(&mut f);
        assert!(f.output.is_empty(), "no fuel -> no smelt");
        assert!(!f.input.is_empty());
    }

    #[test]
    fn fuel_and_smelt_results() {
        assert!(is_fuel(item_by_id("minecraft:oak_planks").unwrap()));
        assert!(!is_fuel(item_by_id("minecraft:stone").unwrap()));
        assert_eq!(
            smelt_result(item_by_id("minecraft:sand").unwrap()),
            item_by_id("minecraft:glass")
        );
        assert_eq!(smelt_result(item_by_id("minecraft:dirt").unwrap()), None);
    }
}
