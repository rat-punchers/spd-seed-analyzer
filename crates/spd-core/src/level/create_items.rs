//! Port of `RegularLevel.createItems` main random drop loop.

use crate::dungeon::DungeonState;
use crate::items::model::{GeneratedItem, ItemCategory};
use crate::level::terrain::{TerrainMap, EXIT, FURROWED_GRASS, GRASS, HIGH_GRASS, WATER};
use crate::random::Random;
use crate::rooms::room::Room;
use crate::rooms::types::RoomKind;
use crate::trinkets::Challenge;

/// Cracked Spyglass loot: `drop(...).hidden = true` (`RegularLevel.java:681-689`).
pub(crate) const SPYGLASS_SOURCE: &str = "cracked_spyglass";
/// Into Darkness floor torch (`RegularLevel.java:472-490`).
pub(crate) const DARKNESS_TORCH_SOURCE: &str = "into_darkness";

#[derive(Debug, Clone)]
pub struct PlacedLoot {
    pub item: GeneratedItem,
    pub heap_type: &'static str,
}

#[derive(Debug, Clone)]
pub struct CreatedLoot {
    pub loot: PlacedLoot,
    /// Exact `randomDropCell` result; absent for unplaced/unsupported late drops.
    pub cell: Option<usize>,
    /// Run history or player state can change this drop or its cell.
    pub may_differ: bool,
}

impl CreatedLoot {
    fn new(item: GeneratedItem, heap_type: &'static str, cell: Option<usize>) -> Self {
        Self {
            may_differ: Random::stream_tainted() || item.history_variable,
            loot: PlacedLoot { item, heap_type },
            cell,
        }
    }
}

/// Main createItems random drops + forced itemsToSpawn placement.
/// `randomDropCell` matches Java: full-list reshuffle per try, StandardRoom
/// instanceof scan (Entrance/Exit/Standard kinds), one persistent in-place
/// room permutation across all drops, and exact exit-cell exclusion.
pub fn create_items_main(
    dungeon: &mut DungeonState,
    rooms: &[Room],
    map: &mut TerrainMap,
    feeling_large: bool,
    items_to_spawn: Vec<GeneratedItem>,
) -> Vec<CreatedLoot> {
    let mut out = Vec::new();
    let mut items_to_spawn = items_to_spawn;
    let mut occupied = map.heap_occupied.clone();
    for (occupied, &mob) in occupied.iter_mut().zip(&map.mob_occupied) {
        *occupied |= mob;
    }
    // Java shuffles the level's `rooms` ArrayList itself. Preserve that
    // permutation across every random and forced drop on this floor.
    let mut room_order: Vec<usize> = (0..rooms.len()).collect();

    // nItems = 3 + chances{6,3,1}; +2 if LARGE
    let mut n_items = 3 + Random::chances(&[6., 3., 1.]);
    if feeling_large {
        n_items += 2;
    }

    for _ in 0..n_items {
        let mut to_drop = dungeon.generator.random(dungeon.depth);
        to_drop.source = Some("heap".into());

        let cell = random_drop_cell(rooms, &mut room_order, map, &mut occupied);
        if cell < 0 {
            continue;
        }
        flatten_grass(map, cell as usize);

        // Heap type. Mimic arms `continue` after recording the carried items.
        let heap_type = match Random::int_max(20) {
            0 => "skeleton",
            1..=4 => {
                // Regular chests gain 25 percentage points for each +1x
                // Mimic Tooth multiplier beyond the normal rate.
                let mimic_chance = (crate::level::trinkets::mimic_chance_multiplier() - 1.0) / 4.0;
                if Random::float() < mimic_chance {
                    emit_mimic(
                        dungeon,
                        &mut out,
                        cell as usize,
                        "mimic",
                        to_drop,
                        MimicPrize::Ordinary,
                    );
                    continue;
                }
                "chest"
            }
            5 => {
                if dungeon.depth > 1 {
                    emit_mimic(
                        dungeon,
                        &mut out,
                        cell as usize,
                        "mimic",
                        to_drop,
                        MimicPrize::Ordinary,
                    );
                    continue;
                }
                "chest"
            }
            _ => "heap",
        };
        // locked chest upgrade path
        let upgradable = matches!(
            to_drop.category,
            ItemCategory::Weapon
                | ItemCategory::Armor
                | ItemCategory::Missile
                | ItemCategory::Wand
                | ItemCategory::Ring
        );
        let is_artifact = to_drop.category == ItemCategory::Artifact;
        let mut heap_type = heap_type;
        let level = to_drop.level.max(0);
        if (is_artifact && Random::int_max(2) == 0)
            || (upgradable && Random::int_max((4 - level).max(1)) == 0)
        {
            let mimic_chance = 0.1 * crate::level::trinkets::mimic_chance_multiplier();
            if dungeon.depth > 1 && Random::float() < mimic_chance {
                emit_mimic(
                    dungeon,
                    &mut out,
                    cell as usize,
                    "golden_mimic",
                    to_drop,
                    MimicPrize::Golden,
                );
                continue;
            } else {
                heap_type = "locked_chest";
                // Java appends the matching key to itemsToSpawn immediately;
                // it is placed after the remaining main-loop drops.
                items_to_spawn.push(GeneratedItem::new("GoldenKey", ItemCategory::Other));
            }
        }
        out.push(CreatedLoot::new(to_drop, heap_type, Some(cell as usize)));
    }

    // place itemsToSpawn as heaps
    for mut item in items_to_spawn {
        // Pre-build forced drops arrive already tagged "forced"; room-paint
        // additions (IronKey, PoolRoom potion, …) are tagged separately so the
        // schema-v2 pre-build forced-queue oracle can tell the two apart.
        if item.source.is_none() {
            item.source = Some("items_to_spawn".into());
        }
        let cell = random_drop_cell(rooms, &mut room_order, map, &mut occupied);
        let cell = (cell >= 0).then_some(cell as usize);
        if let Some(cell) = cell {
            flatten_grass(map, cell);
        }
        if item.class_name == "TrinketCatalyst" {
            let catalyst_provenance = item.provenance;
            out.push(CreatedLoot::new(item, "locked_chest", cell));
            let key_cell = random_drop_cell(rooms, &mut room_order, map, &mut occupied);
            if key_cell >= 0 {
                flatten_grass(map, key_cell as usize);
                let mut key = GeneratedItem::new("GoldenKey", ItemCategory::Other);
                key.source = Some("forced".into());
                key.provenance = catalyst_provenance;
                out.push(CreatedLoot::new(key, "heap", Some(key_cell as usize)));
            }
        } else {
            out.push(CreatedLoot::new(item, "heap", cell));
        }
    }

    // Separate generators for torch/bones/rose/rations/guide — consume Long seeds
    Random::push_generator_seeded(Random::long());
    if dungeon.challenges & Challenge::IntoDarkness.mask() != 0 {
        let torches = if feeling_large { 2 } else { 1 };
        for _ in 0..torches {
            let cell = random_drop_cell(rooms, &mut room_order, map, &mut occupied);
            let cell = (cell >= 0).then_some(cell as usize);
            if let Some(cell) = cell {
                flatten_grass(map, cell);
            }
            let mut torch = GeneratedItem::new("Torch", ItemCategory::Other);
            torch.source = Some(DARKNESS_TORCH_SOURCE.into());
            out.push(CreatedLoot::new(torch, "heap", cell));
        }
    }
    Random::pop_generator();

    Random::push_generator_seeded(Random::long());
    Random::pop_generator();

    Random::push_generator_seeded(Random::long());
    Random::pop_generator();

    Random::push_generator_seeded(Random::long());
    Random::pop_generator();

    // Dried Rose petals and Cached Rations drop here only for a hero holding
    // them, and every drop permutes the level's room order. From depth 2 on,
    // every later placement therefore depends on player state.
    let player_state_tail = out.len();

    Random::push_generator_seeded(Random::long());
    let drop_chance = 0.25f32 * (dungeon.depth - 1) as f32;
    // A complete guide leaves `missingPages` empty, which short-circuits the
    // roll. A fresh journal has pages missing (apart from GUIDE_SEARCHING,
    // which Java removes from the random pool). Creating and selecting the
    // first missing page consumes no RNG; placement does mutate the level's
    // persistent room order using this nested generator.
    if !crate::journal::lore_complete() && Random::float() < drop_chance {
        let cell = random_drop_cell(rooms, &mut room_order, map, &mut occupied);
        if cell >= 0 {
            flatten_grass(map, cell as usize);
            let page = GeneratedItem::new("GuidePage", ItemCategory::Other);
            out.push(CreatedLoot::new(page, "heap", Some(cell as usize)));
        }
    }
    Random::pop_generator();

    Random::push_generator_seeded(Random::long());
    Random::pop_generator();

    Random::push_generator_seeded(Random::long());
    if Random::float() < crate::level::trinkets::ebony_mimic_chance() {
        if let Some((cell, items)) = ebony::spawn(dungeon, rooms, map, &out) {
            for mut item in items {
                item.source = Some("ebony_mimic".into());
                out.push(CreatedLoot::new(item, "ebony_mimic", Some(cell)));
            }
        }
    }
    Random::pop_generator();

    Random::push_generator_seeded(Random::long());
    let items = (Random::float() + crate::level::trinkets::spyglass_extra_loot_chance()) as i32;
    for _ in 0..items {
        let cell = random_drop_cell(rooms, &mut room_order, map, &mut occupied);
        let cell = (cell >= 0).then_some(cell as usize);
        if let Some(cell) = cell {
            flatten_grass(map, cell);
        }
        let mut item = dungeon.generator.random_using_defaults_any(dungeon.depth);
        item.source = Some(SPYGLASS_SOURCE.into());
        out.push(CreatedLoot::new(item, "heap", cell));
    }
    Random::pop_generator();

    if dungeon.depth > 1 {
        for created in &mut out[player_state_tail..] {
            created.may_differ = true;
        }
    }
    out
}

/// One mimic's carried items, each tagged with `heap_type` so the report can
/// say which mimic drops them. The seed item's previous source is replaced.
fn emit_mimic(
    dungeon: &mut DungeonState,
    out: &mut Vec<CreatedLoot>,
    cell: usize,
    heap_type: &'static str,
    seed: GeneratedItem,
    kind: MimicPrize,
) {
    let mut items = mimic_carried_items(dungeon, vec![seed], true, kind);
    for item in &mut items {
        item.source = Some(heap_type.into());
    }
    for item in items {
        out.push(CreatedLoot::new(item, heap_type, Some(cell)));
    }
}

/// `Mimic.generatePrize(useDecks)` base reward, before trinket extras.
pub(super) fn generate_base_mimic_prize(
    dungeon: &mut DungeonState,
    use_decks: bool,
) -> GeneratedItem {
    match Random::int_max(5) {
        0 => {
            let mut gold = GeneratedItem::new("Gold", ItemCategory::Gold);
            gold.quantity =
                Random::int_range_inclusive(30 + dungeon.depth * 10, 60 + dungeon.depth * 20);
            gold
        }
        1 => dungeon
            .generator
            .random_missile(dungeon.depth / 5, !use_decks, dungeon.depth),
        2 => dungeon
            .generator
            .random_armor(dungeon.depth / 5, dungeon.depth),
        3 => dungeon
            .generator
            .random_weapon(dungeon.depth / 5, !use_decks, dungeon.depth),
        4 if use_decks => dungeon
            .generator
            .random_category(crate::generator::Category::Ring, dungeon.depth),
        4 => dungeon
            .generator
            .random_using_defaults(crate::generator::Category::Ring, dungeon.depth),
        _ => unreachable!("Random.Int(5) stays in range"),
    }
}

fn flatten_grass(map: &mut TerrainMap, cell: usize) {
    if matches!(map.map[cell], HIGH_GRASS | FURROWED_GRASS) {
        map.map[cell] = GRASS;
    }
}

/// Returns index into `map.map` / `occupied`, or -1.
///
/// Port of `RegularLevel.randomDropCell(StandardRoom.class)`:
/// - Every try calls `randomRoom`, i.e. `Random.shuffle(rooms)` on the ENTIRE
///   room list (entrance, exit, standard, special, secret, connection, shop,
///   AND set-empty rooms), then linear-scans the shuffled order for the first
///   `StandardRoom` instance. In Java `EntranceRoom`/`ExitRoom` extend
///   `StandardRoom`, so Rust kinds Entrance/Exit/Standard all match.
/// - If the scan finds no match, Java returns -1 on the FIRST try (exactly one
///   full-list shuffle burned). There is no fallback room set.
/// - The entrance room never hosts a drop (`room != roomEntrance`): picking it
///   wastes the try with no `Room.random()` draws.
/// - The exit room CAN host a drop, just never on the exit cell itself.
/// - Set-empty rooms stay in the list; if picked, watabou `Int(max <= 0)`
///   returns 0 WITHOUT consuming a draw, so the degenerate
///   `IntRange(left+1, right-1)` pair on a zeroed rect burns zero draws and
///   yields (1, 1), which then fails the terrain checks below.
fn random_drop_cell(
    rooms: &[Room],
    order: &mut [usize],
    map: &TerrainMap,
    occupied: &mut [bool],
) -> i32 {
    // Java `exit()` is the REGULAR_EXIT transition cell — the single cell the
    // exit room painter set to Terrain.EXIT. Invariant during this loop.
    let exit_cell = map.map.iter().position(|&t| t == EXIT);

    let mut tries = 100;
    while tries > 0 {
        tries -= 1;
        // `randomRoom`: Collections.shuffle on the full list, every try.
        Random::shuffle_list(order);
        // `room(type)`: first instanceof-StandardRoom in shuffled order.
        let room = order.iter().map(|&ri| &rooms[ri]).find(|r| {
            matches!(
                r.kind,
                RoomKind::Entrance | RoomKind::Exit | RoomKind::Standard
            )
        });
        let Some(room) = room else {
            return -1;
        };
        if room.is_entrance() {
            // `room != roomEntrance` — try wasted, no point draws.
            continue;
        }
        // `Room.random()` = IntRange(left+1, right-1), IntRange(top+1, bottom-1).
        let x = Random::int_range_inclusive(room.left + 1, room.right - 1);
        let y = Random::int_range_inclusive(room.top + 1, room.bottom - 1);
        let Some(idx) = map.point_to_cell(x, y) else {
            continue;
        };
        if idx >= map.passable.len() || !map.passable[idx] {
            continue;
        }
        if map.is_solid(idx) {
            continue;
        }
        if Some(idx) == exit_cell {
            continue;
        }
        if idx >= occupied.len() || occupied[idx] {
            continue;
        }
        if !map.item_allowed.get(idx).copied().unwrap_or(false) {
            continue;
        }
        // AquariumRoom checks the final terrain dynamically, including water
        // added by RegularPainter after the room's own pool was painted.
        if room.name == "AquariumRoom" && map.map[idx] == WATER {
            continue;
        }
        // `findMob(pos) == null`: room-painted and depth-one ambient mobs are
        // folded into `occupied` by the caller.
        // Items cannot spawn on traps that destroy items (Burning/Frost/…/Pitfall).
        if map.trap_destroys_items.get(idx).copied().unwrap_or(false) {
            continue;
        }
        occupied[idx] = true;
        return idx as i32;
    }
    -1
}

mod ebony;
mod prize;

pub(crate) use prize::{mimic_carried_items, MimicPrize};

#[cfg(test)]
mod tests;
