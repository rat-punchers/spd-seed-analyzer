//! `RegularLevel.createItems` ebony mimic (`RegularLevel.java:647-673`).
//!
//! Runs inside its own `Random.pushGenerator(Random.Long())`, so the ambient
//! stream is untouched. The prize still matters: `randomUsingDefaults()` can
//! draw an artifact and consume the artifact deck for later floors.

use super::{mimic_carried_items, CreatedLoot, MimicPrize};
use crate::builders::int_map::IntMap;
use crate::dungeon::DungeonState;
use crate::level::terrain::{TerrainMap, DOOR, EXIT};
use crate::random::Random;
use crate::rooms::room::Room;
use crate::rooms::types::RoomKind;

/// Created loot that Java spawns as a mob rather than dropping as a heap.
fn is_mob(heap_type: &str) -> bool {
    matches!(heap_type, "mimic" | "golden_mimic")
}

/// Called after the ebony roll succeeded. Returns the cell and carried items
/// when the cell is seed-determined.
pub(super) fn spawn(
    dungeon: &mut DungeonState,
    rooms: &[Room],
    map: &mut TerrainMap,
    created: &[CreatedLoot],
) -> Option<(usize, Vec<crate::items::model::GeneratedItem>)> {
    let mob_at = |cell: usize| {
        map.mob_occupied[cell]
            || created
                .iter()
                .any(|loot| loot.cell == Some(cell) && is_mob(loot.loot.heap_type))
    };

    let mut candidates = Vec::new();
    // On a fresh journal, floor 2's entrance guide page is a plain heap too,
    // but `EntranceRoom.placeEarlyGuidePages` drops it with an unseeded
    // generator: it counts as a candidate while its `IntMap` position, and so
    // the pick, is unknown.
    let mut unplaced_page = false;
    if Random::int_max(2) == 0 {
        let heaps = level_heaps(map, created);
        for key in heaps.key_array() {
            let cell = key as usize;
            let plain_heap = heaps.get(key).copied().unwrap_or(false);
            if plain_heap && !in_special_room(rooms, map, cell) && !mob_at(cell) {
                candidates.push(cell);
            }
        }
        unplaced_page = dungeon.depth == 2 && !crate::journal::lore_complete();
    }

    if unplaced_page {
        let _ = Random::int_max(candidates.len() as i32 + 1);
        let _ = mimic_carried_items(dungeon, Vec::new(), false, MimicPrize::Ebony);
        return None;
    }

    if candidates.is_empty() {
        let exit = map.map.iter().position(|&tile| tile == EXIT);
        if Random::int_max(5) == 0 && exit.is_some_and(|cell| !mob_at(cell)) {
            candidates.extend(exit);
        } else {
            candidates
                .extend((0..map.len()).filter(|&cell| map.map[cell] == DOOR && !mob_at(cell)));
        }
    }

    let &cell = Random::element(&candidates)?;
    let items = mimic_carried_items(dungeon, Vec::new(), false, MimicPrize::Ebony);
    map.mob_occupied[cell] = true;
    map.place_mob(cell, "EbonyMimic");
    Some((cell, items))
}

/// `Level.heaps` at this point: room-paint heaps, then createItems drops, in
/// `heaps.put` order. Values say whether the heap is `Heap.Type.HEAP`.
fn level_heaps(map: &TerrainMap, created: &[CreatedLoot]) -> IntMap<bool> {
    let mut heaps = IntMap::new();
    for &cell in &map.heap_order {
        let plain = map.known_heaps[cell]
            .as_ref()
            .is_some_and(|heap| heap.heap_type == "heap");
        heaps.put(cell as i32, plain);
    }
    for loot in created {
        let Some(cell) = loot.cell else { continue };
        if is_mob(loot.loot.heap_type) {
            continue;
        }
        heaps.put(cell as i32, loot.loot.heap_type == "heap");
    }
    heaps
}

/// `room(pos) instanceof SpecialRoom`: the first room, in level order, whose
/// interior holds the cell.
fn in_special_room(rooms: &[Room], map: &TerrainMap, cell: usize) -> bool {
    let x = (cell % map.width as usize) as i32 + map.origin_x;
    let y = (cell / map.width as usize) as i32 + map.origin_y;
    rooms
        .iter()
        .find(|room| x > room.left && y > room.top && x < room.right && y < room.bottom)
        .is_some_and(|room| {
            matches!(
                room.kind,
                RoomKind::Special | RoomKind::Secret | RoomKind::Shop
            )
        })
}
