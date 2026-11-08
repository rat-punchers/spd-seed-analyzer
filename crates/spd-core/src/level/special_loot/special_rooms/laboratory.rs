//! Exact regular `LaboratoryRoom.paint` geometry, blobs, and placed loot.

use super::super::placement::find_prize_item;
use crate::dungeon::DungeonState;
use crate::generator::Category;
use crate::geom::Point;
use crate::items::model::{GeneratedItem, ItemCategory};
use crate::level::create_items::PlacedLoot;
use crate::level::painter::DoorMap;
use crate::level::terrain::{TerrainMap, ALCHEMY, EMPTY_SP, WALL};
use crate::random::Random;
use crate::rooms::room::Room;

pub(in crate::level::special_loot) fn paint(
    dungeon: &mut DungeonState,
    room: &Room,
    room_index: usize,
    map: &mut TerrainMap,
    doors: &DoorMap,
    items_to_spawn: &mut Vec<GeneratedItem>,
) -> Vec<PlacedLoot> {
    fill_room(map, room, WALL);
    fill_margin(map, room, 1, EMPTY_SP);

    let entrance = room
        .connected
        .iter()
        .find_map(|&other| doors.get(room_index, other))
        .expect("placed LaboratoryRoom has an entrance");
    let pot = if entrance.x == room.left {
        Point::new(
            room.right - 1,
            if Random::int_max(2) == 0 {
                room.top + 1
            } else {
                room.bottom - 1
            },
        )
    } else if entrance.x == room.right {
        Point::new(
            room.left + 1,
            if Random::int_max(2) == 0 {
                room.top + 1
            } else {
                room.bottom - 1
            },
        )
    } else if entrance.y == room.top {
        Point::new(
            if Random::int_max(2) == 0 {
                room.left + 1
            } else {
                room.right - 1
            },
            room.bottom - 1,
        )
    } else {
        Point::new(
            if Random::int_max(2) == 0 {
                room.left + 1
            } else {
                room.right - 1
            },
            room.top + 1,
        )
    };
    let pot_cell = map
        .point_to_cell(pot.x, pot.y)
        .expect("LaboratoryRoom pot is inside the map");
    map.map[pot_cell] = ALCHEMY;
    // Java `Alchemy` instance initializer sets `alwaysVisible = true`.
    map.record_blob_cell("Alchemy", true, pot_cell, 1);

    let mut out = Vec::new();
    let mut crystal = GeneratedItem::new("EnergyCrystal", ItemCategory::Other);
    crystal.quantity = 5;
    crystal.source = Some("LaboratoryRoom".into());
    drop_item(map, room, crystal, &mut out);

    let n = Random::normal_int_range(1, 2);
    for _ in 0..n {
        // Java chooses the free cell before generating the prize.
        let cell = random_empty_sp_cell(map, room);
        let mut item = if let Some(catalyst) =
            find_prize_item(items_to_spawn, Some("TrinketCatalyst"))
        {
            catalyst
        } else if let Some(strength) = find_prize_item(items_to_spawn, Some("PotionOfStrength")) {
            strength
        } else {
            let category = *Random::one_of(&[Category::Potion, Category::Stone]);
            dungeon.generator.random_category(category, dungeon.depth)
        };
        item.source = Some("LaboratoryRoom".into());
        record_drop(map, cell, item, &mut out);
    }

    // A complete journal drops no pages. A fresh one has all nine alchemy
    // pages missing, so at chapter two (prison) Java drops two pages, one per
    // chapter since target 1.
    let chapter = 1 + dungeon.depth / 5;
    let pages = if crate::journal::lore_complete() {
        0
    } else {
        chapter.min(9)
    };
    for _ in 0..pages {
        let mut page = GeneratedItem::new("AlchemyPage", ItemCategory::Other);
        page.source = Some("LaboratoryRoom".into());
        drop_item(map, room, page, &mut out);
    }

    items_to_spawn.push(GeneratedItem::new("IronKey", ItemCategory::Other));
    out
}

fn drop_item(map: &mut TerrainMap, room: &Room, item: GeneratedItem, out: &mut Vec<PlacedLoot>) {
    let cell = random_empty_sp_cell(map, room);
    record_drop(map, cell, item, out);
}

fn record_drop(map: &mut TerrainMap, cell: usize, item: GeneratedItem, out: &mut Vec<PlacedLoot>) {
    map.record_heap(cell, "heap", item.clone());
    out.push(PlacedLoot {
        item,
        heap_type: "heap",
    });
}

fn random_empty_sp_cell(map: &TerrainMap, room: &Room) -> usize {
    loop {
        let point = room.random();
        let cell = map
            .point_to_cell(point.x, point.y)
            .expect("LaboratoryRoom point is inside the map");
        if map.map[cell] == EMPTY_SP && !map.heap_occupied[cell] {
            return cell;
        }
    }
}

fn fill_room(map: &mut TerrainMap, room: &Room, terrain: i32) {
    fill_rect(map, room.left, room.top, room.right, room.bottom, terrain);
}

fn fill_margin(map: &mut TerrainMap, room: &Room, margin: i32, terrain: i32) {
    fill_rect(
        map,
        room.left + margin,
        room.top + margin,
        room.right - margin,
        room.bottom - margin,
        terrain,
    );
}

fn fill_rect(map: &mut TerrainMap, left: i32, top: i32, right: i32, bottom: i32, terrain: i32) {
    for y in top..=bottom {
        for x in left..=right {
            if let Some(cell) = map.point_to_cell(x, y) {
                map.map[cell] = terrain;
            }
        }
    }
}
