//! Crafting recipes and grid matching (M3).

use std::sync::{LazyLock, RwLock};

use crate::item::{item_by_id, Item, ItemStack};

/// A shaped crafting recipe on a 3×3 grid; `None` cells must be empty.
pub struct Recipe {
    pub pattern: [[Option<Item>; 3]; 3],
    pub output: (Item, u32),
}

/// The supported recipes (a small M3 subset, not full vanilla parity).
fn core_recipes() -> Vec<Recipe> {
    let planks = item_by_id("minecraft:oak_planks").unwrap();
    let stick = item_by_id("minecraft:stick").unwrap();
    let table = item_by_id("minecraft:crafting_table").unwrap();
    let wooden_pickaxe = item_by_id("minecraft:wooden_pickaxe").unwrap();
    let cobble = item_by_id("minecraft:cobblestone").unwrap();
    let stone_pickaxe = item_by_id("minecraft:stone_pickaxe").unwrap();

    let mut out = Vec::new();

    // Any oak log -> 4 planks (three axis variants, shapeless 1×1).
    for log in [
        "minecraft:oak_log[axis=x]",
        "minecraft:oak_log[axis=y]",
        "minecraft:oak_log[axis=z]",
    ] {
        let log = item_by_id(log).unwrap();
        out.push(Recipe {
            pattern: [[Some(log), None, None], [None, None, None], [None, None, None]],
            output: (planks, 4),
        });
    }
    // 2×2 planks -> crafting table.
    out.push(Recipe {
        pattern: [
            [Some(planks), Some(planks), None],
            [Some(planks), Some(planks), None],
            [None, None, None],
        ],
        output: (table, 1),
    });
    // 2 planks vertical -> 4 sticks.
    out.push(Recipe {
        pattern: [
            [Some(planks), None, None],
            [Some(planks), None, None],
            [None, None, None],
        ],
        output: (stick, 4),
    });
    // 3 planks top + 2 sticks middle -> wooden pickaxe.
    out.push(Recipe {
        pattern: [
            [Some(planks), Some(planks), Some(planks)],
            [None, Some(stick), None],
            [None, Some(stick), None],
        ],
        output: (wooden_pickaxe, 1),
    });
    // 3 cobblestone top + 2 sticks middle -> stone pickaxe.
    out.push(Recipe {
        pattern: [
            [Some(cobble), Some(cobble), Some(cobble)],
            [None, Some(stick), None],
            [None, Some(stick), None],
        ],
        output: (stone_pickaxe, 1),
    });

    out
}

/// The runtime recipe list (core recipes plus mod-registered recipes).
static RECIPES: LazyLock<RwLock<Vec<Recipe>>> =
    LazyLock::new(|| RwLock::new(core_recipes()));

/// Register a new recipe (mod content).
pub fn register_recipe(recipe: Recipe) {
    RECIPES.write().expect("recipe registry").push(recipe);
}

/// Whether `grid` matches `recipe` (non-empty pattern cells must match with a
/// non-zero count; empty pattern cells must be empty).
fn matches(grid: &[ItemStack; 9], recipe: &Recipe) -> bool {
    for r in 0..3 {
        for c in 0..3 {
            let idx = r * 3 + c;
            match recipe.pattern[r][c] {
                Some(item) => {
                    if grid[idx].item != item || grid[idx].count == 0 {
                        return false;
                    }
                }
                None => {
                    if !grid[idx].is_empty() {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Match `grid` against the recipes, returning the first matching output.
pub fn craft(grid: &[ItemStack; 9]) -> Option<(Item, u32)> {
    RECIPES
        .read()
        .expect("recipe registry")
        .iter()
        .find(|r| matches(grid, r))
        .map(|r| r.output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> [ItemStack; 9] {
        [ItemStack::EMPTY; 9]
    }

    #[test]
    fn log_crafts_into_planks() {
        let mut g = empty();
        g[0] = ItemStack::new(item_by_id("minecraft:oak_log[axis=y]").unwrap(), 1);
        assert_eq!(
            craft(&g),
            Some((item_by_id("minecraft:oak_planks").unwrap(), 4))
        );
    }

    #[test]
    fn planks_craft_into_table_and_sticks() {
        let planks = item_by_id("minecraft:oak_planks").unwrap();
        let mut table = empty();
        table[0] = ItemStack::new(planks, 1);
        table[1] = ItemStack::new(planks, 1);
        table[3] = ItemStack::new(planks, 1);
        table[4] = ItemStack::new(planks, 1);
        assert_eq!(
            craft(&table),
            Some((item_by_id("minecraft:crafting_table").unwrap(), 1))
        );

        let mut sticks = empty();
        sticks[0] = ItemStack::new(planks, 1);
        sticks[3] = ItemStack::new(planks, 1);
        assert_eq!(
            craft(&sticks),
            Some((item_by_id("minecraft:stick").unwrap(), 4))
        );
    }

    #[test]
    fn wooden_pickaxe_recipe() {
        let planks = item_by_id("minecraft:oak_planks").unwrap();
        let stick = item_by_id("minecraft:stick").unwrap();
        let mut g = empty();
        g[0] = ItemStack::new(planks, 1);
        g[1] = ItemStack::new(planks, 1);
        g[2] = ItemStack::new(planks, 1);
        g[4] = ItemStack::new(stick, 1);
        g[7] = ItemStack::new(stick, 1);
        assert_eq!(
            craft(&g),
            Some((item_by_id("minecraft:wooden_pickaxe").unwrap(), 1))
        );
    }
}
