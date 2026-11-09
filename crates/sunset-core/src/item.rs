//! Items and item stacks (PRD M3). Items are a distinct durable namespace from
//! block states: a block item is the placeable "item form" of a block, while
//! non-block items (sticks, tools, …) have no block form.

use std::sync::{LazyLock, RwLock};

use crate::block::{block_def, state_by_id, BlockState};
use crate::registry::Registry;

/// A stackable item (an index into the item space: block items first, then
/// non-block items).
pub type Item = u32;

/// The empty item (an empty inventory slot).
pub const EMPTY_ITEM: Item = u32::MAX;

/// The definition of a single item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemDef {
    /// Namespaced id, e.g. `minecraft:stone` or `minecraft:stick`.
    pub id: &'static str,
    /// Maximum stack size (64 for most items).
    pub max_stack: u32,
    /// The block placed when this item is used, if it is a block item.
    pub block: Option<BlockState>,
}

/// Non-block items, indexed after the block items (core content).
const CORE_NON_BLOCK_ITEMS: &[ItemDef] = &[
    ItemDef {
        id: "minecraft:stick",
        max_stack: 64,
        block: None,
    },
    ItemDef {
        id: "minecraft:wooden_pickaxe",
        max_stack: 1,
        block: None,
    },
    ItemDef {
        id: "minecraft:stone_pickaxe",
        max_stack: 1,
        block: None,
    },
];

/// The runtime non-block item registry (core content plus mod-registered items).
static NON_BLOCK_ITEMS: LazyLock<RwLock<Registry<ItemDef>>> =
    LazyLock::new(|| RwLock::new(core_non_block_items()));

fn core_non_block_items() -> Registry<ItemDef> {
    let mut r = Registry::new();
    for def in CORE_NON_BLOCK_ITEMS {
        r.register(def.id, *def);
    }
    r
}

/// Number of block items (one per block state).
fn block_item_count() -> u32 {
    crate::block::block_count()
}

/// The definition of an item, or `None` for [`EMPTY_ITEM`].
pub fn item_def(item: Item) -> Option<ItemDef> {
    if item == EMPTY_ITEM {
        return None;
    }
    let n = block_item_count();
    if item < n {
        // Block item: shares its block's id and places that block.
        let state = item as BlockState;
        Some(ItemDef {
            id: block_def(state).id,
            max_stack: 64,
            block: Some(state),
        })
    } else {
        NON_BLOCK_ITEMS
            .read()
            .expect("item registry")
            .get(item - n)
            .copied()
    }
}

/// The item corresponding to a block state (its "item form").
pub fn item_for_block(state: BlockState) -> Item {
    state
}

/// Register a new non-block item (mod content), returning its item id.
pub fn register_item(def: ItemDef) -> Item {
    let idx = NON_BLOCK_ITEMS
        .write()
        .expect("item registry")
        .register(def.id, def);
    block_item_count() + idx
}

/// The namespaced id of an item, or `None` for [`EMPTY_ITEM`].
pub fn item_id(item: Item) -> Option<&'static str> {
    item_def(item).map(|d| d.id)
}

/// Look up an item by namespaced id (block items first, then non-block items).
pub fn item_by_id(id: &str) -> Option<Item> {
    if let Some(state) = state_by_id(id) {
        return Some(item_for_block(state));
    }
    NON_BLOCK_ITEMS
        .read()
        .expect("item registry")
        .find(id)
        .map(|i| block_item_count() + i)
}

/// Whether an item is a pickaxe (tool eligibility for mining stone, M3).
pub fn is_pickaxe(item: Item) -> bool {
    matches!(
        item_id(item),
        Some("minecraft:wooden_pickaxe") | Some("minecraft:stone_pickaxe")
    )
}

/// What a block drops when broken (M3). Stone requires a pickaxe and drops
/// cobblestone; every other block drops its own item form with no tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropInfo {
    pub item: Item,
    pub needs_pickaxe: bool,
}

/// The drop for breaking `state`.
pub fn block_drop(state: BlockState) -> DropInfo {
    match block_def(state).id {
        "minecraft:stone" => DropInfo {
            item: item_by_id("minecraft:cobblestone").expect("registered item"),
            needs_pickaxe: true,
        },
        _ => DropInfo {
            item: item_for_block(state),
            needs_pickaxe: false,
        },
    }
}

/// A stack of a single item type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

impl ItemStack {
    /// An empty stack.
    pub const EMPTY: ItemStack = ItemStack {
        item: EMPTY_ITEM,
        count: 0,
    };

    pub fn new(item: Item, count: u32) -> Self {
        Self { item, count }
    }

    pub fn is_empty(&self) -> bool {
        self.item == EMPTY_ITEM || self.count == 0
    }

    /// The maximum count a stack of this item can hold (0 when empty).
    pub fn max_stack(&self) -> u32 {
        item_def(self.item).map(|d| d.max_stack).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_items_round_trip_through_ids() {
        let stone = state_by_id("minecraft:stone").unwrap();
        let item = item_for_block(stone);
        assert_eq!(item_id(item), Some("minecraft:stone"));
        assert_eq!(item_by_id("minecraft:stone"), Some(item));
        assert_eq!(item_def(item).unwrap().block, Some(stone));
    }

    #[test]
    fn non_block_item_lookup() {
        let stick = item_by_id("minecraft:stick").unwrap();
        assert_eq!(item_def(stick).unwrap().id, "minecraft:stick");
        assert_eq!(item_def(stick).unwrap().block, None);
    }

    #[test]
    fn stack_limits() {
        let stone = item_for_block(state_by_id("minecraft:stone").unwrap());
        assert_eq!(ItemStack::new(stone, 1).max_stack(), 64);
        assert!(ItemStack::EMPTY.is_empty());
    }
}
