//! Focused coverage for room-paint additions to `itemsToSpawn`.

use super::super::quest_rooms::mass_grave_prizes;
use super::super::secret_rooms::{secret_laboratory, secret_larder, secret_runestone};
use super::super::special_rooms::{
    armory_prizes_on_map, crypt_prize, library_prizes, paint_laboratory, pool_prize,
    runestone_prizes, runestone_prizes_on_map, statue_prizes, treasury_prizes_on_map,
};
use super::test_room;
use crate::geom::Point;
use crate::items::model::ItemCategory;
use crate::level::painter::DoorMap;
use crate::level::terrain::{paint_minimal, EMPTY, EMPTY_SP, STATUE, WALL, WALL_DECO};
use crate::random::Random;
use crate::rooms::room::Room;
use crate::rooms::types::RoomKind;
use crate::run::{dungeon_from_run, init_run};

#[test]
fn runestone_room_pushes_iron_key() {
    Random::reset_generators();
    let run = init_run(31);
    Random::push_generator_seeded(101);
    let mut d = dungeon_from_run(run);
    d.depth = 7;
    let room = test_room("RunestoneRoom", 8, 8);
    let mut spawn = Vec::new();
    let loot = runestone_prizes(&mut d, &room, &mut spawn);
    Random::pop_generator();

    assert!((2..=3).contains(&loot.len()));
    assert!(loot
        .iter()
        .all(|p| p.item.source.as_deref() == Some("RunestoneRoom")));
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "IronKey");
    assert_eq!(spawn[0].category, ItemCategory::Other);
}

#[test]
fn runestone_room_retains_exact_stone_cells_and_rng_tail() {
    Random::reset_generators();
    let run = init_run(31);
    Random::push_generator_seeded(101);
    let mut dungeon = dungeon_from_run(run);
    dungeon.depth = 7;
    let mut room = test_room("RunestoneRoom", 8, 8);
    room.connected.push(1);
    let mut neighbour = Room::new(1, "TunnelRoom", RoomKind::Connection, 1, 16, 3, 10, 3, 10);
    neighbour.left = 8;
    neighbour.top = 2;
    neighbour.right = 12;
    neighbour.bottom = 6;
    neighbour.connected.push(0);
    let rooms = vec![room, neighbour];
    let mut map = paint_minimal(&rooms).expect("test map");
    let mut doors = DoorMap::new();
    doors.insert_test_point(0, 1, Point::new(8, 4));
    super::super::geometry::paint(&mut map, &rooms[0], 0, &doors);
    let mut spawn = Vec::new();

    let loot = runestone_prizes_on_map(&mut dungeon, &rooms[0], &mut map, &mut spawn);
    let tail = Random::peek_ints(4);
    Random::pop_generator();

    let heaps: Vec<_> = map
        .known_heaps
        .iter()
        .enumerate()
        .filter_map(|(cell, heap)| {
            heap.as_ref()
                .map(|heap| (cell, heap.heap_type, heap.items[0].class_name.as_str()))
        })
        .collect();
    assert_eq!(loot.len(), heaps.len());
    assert_eq!(
        heaps,
        [
            (51, "heap", "StoneOfClairvoyance"),
            (63, "heap", "StoneOfAggression"),
            (108, "heap", "StoneOfAggression"),
        ]
    );
    assert_eq!(
        tail,
        [1_187_957_485, 1_420_579_380, 346_128_254, -1_944_953_493]
    );
}

#[test]
fn library_room_pushes_iron_key() {
    Random::reset_generators();
    let run = init_run(32);
    Random::push_generator_seeded(102);
    let mut d = dungeon_from_run(run);
    d.depth = 4;
    let room = test_room("LibraryRoom", 8, 8);
    let mut spawn = Vec::new();
    let mut map = paint_minimal(std::slice::from_ref(&room)).expect("test map");
    let loot = library_prizes(&mut d, &room, &mut map, &mut spawn);
    let tail = Random::peek_ints(4);
    Random::pop_generator();

    assert!((1..=3).contains(&loot.len()));
    assert!(matches!(
        loot[0].item.class_name.as_str(),
        "ScrollOfIdentify" | "ScrollOfRemoveCurse"
    ));
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "IronKey");
    assert_eq!(spawn[0].category, ItemCategory::Other);
    assert_eq!(
        map.known_heaps[36].as_ref().unwrap().items[0].class_name,
        "ScrollOfRemoveCurse"
    );
    assert_eq!(
        tail,
        [-1_945_822_120, -228_609_341, -1_193_648_905, -889_589_932]
    );
}

#[test]
fn treasury_room_pushes_iron_key() {
    Random::reset_generators();
    let run = init_run(33);
    Random::push_generator_seeded(103);
    let mut d = dungeon_from_run(run);
    d.depth = 9;
    let room = test_room("TreasuryRoom", 8, 8);
    let mut spawn = Vec::new();
    let mut map = paint_minimal(std::slice::from_ref(&room)).expect("test map");
    let loot = treasury_prizes_on_map(&mut d, &room, &mut map, &mut spawn);
    Random::pop_generator();

    assert!((2..=3).contains(&loot.len()));
    assert!(loot.iter().all(|p| p
        .item
        .source
        .as_deref()
        .is_some_and(|s| s.starts_with("TreasuryRoom"))));
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "IronKey");
    assert_eq!(spawn[0].category, ItemCategory::Other);
}

#[test]
fn laboratory_room_records_fresh_prison_chapter_loot_blob_and_key() {
    Random::reset_generators();
    let run = init_run(34);
    Random::push_generator_seeded(104);
    let mut d = dungeon_from_run(run);
    d.depth = 8;
    let mut room = test_room("LaboratoryRoom", 8, 8);
    room.connected.push(1);
    let mut neighbour = Room::new(1, "TunnelRoom", RoomKind::Connection, 1, 16, 3, 10, 3, 10);
    neighbour.left = 8;
    neighbour.top = 2;
    neighbour.right = 12;
    neighbour.bottom = 6;
    neighbour.connected.push(0);
    let mut map = paint_minimal(&[room.clone(), neighbour]).expect("test map");
    let mut doors = DoorMap::new();
    doors.insert_test_point(0, 1, Point::new(8, 4));
    let mut spawn = Vec::new();
    let loot = paint_laboratory(&mut d, &room, 0, &mut map, &doors, &mut spawn);
    Random::pop_generator();

    assert!((4..=5).contains(&loot.len()));
    assert!(loot
        .iter()
        .all(|p| p.item.source.as_deref() == Some("LaboratoryRoom")));
    assert_eq!(
        loot.iter()
            .filter(|drop| drop.item.class_name == "AlchemyPage")
            .count(),
        2
    );
    assert_eq!(
        loot.iter()
            .filter(|drop| drop.item.class_name == "EnergyCrystal")
            .count(),
        1
    );
    assert_eq!(map.known_blobs.len(), 1);
    assert_eq!(map.known_blobs[0].class_name, "Alchemy");
    assert_eq!(map.known_blobs[0].cells.len(), 1);
    assert_eq!(
        map.known_heaps.iter().filter(|heap| heap.is_some()).count(),
        loot.len()
    );
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "IronKey");
    assert_eq!(spawn[0].category, ItemCategory::Other);
}

#[test]
fn crypt_room_pushes_iron_key() {
    Random::reset_generators();
    let run = init_run(35);
    Random::push_generator_seeded(105);
    let mut d = dungeon_from_run(run);
    d.depth = 11;
    let mut spawn = Vec::new();
    let loot = crypt_prize(&mut d, &mut spawn);
    Random::pop_generator();

    assert_eq!(loot.heap_type, "tomb");
    assert_eq!(loot.item.source.as_deref(), Some("CryptRoom"));
    assert!(loot.item.cursed);
    assert_eq!(loot.item.category, ItemCategory::Armor);
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "IronKey");
    assert_eq!(spawn[0].category, ItemCategory::Other);
}

#[test]
fn statue_room_consumes_center_jitter_before_weapon_and_pushes_iron_key() {
    Random::reset_generators();
    let run = init_run(36);
    Random::push_generator_seeded(106);
    let mut d = dungeon_from_run(run);
    d.depth = 8;
    let room = test_room("StatueRoom", 7, 7);
    let mut spawn = Vec::new();
    let mut map = paint_minimal(std::slice::from_ref(&room)).expect("map");
    let entrance = Point::new(room.left, room.top + 2);
    let loot = &statue_prizes(&mut d, &room, &mut map, entrance, &mut spawn)[0];
    let after_statue = Random::peek_ints(3);
    Random::pop_generator();
    Random::push_generator_seeded(106);
    let _ = Random::int_max(2);
    let _ = Random::int_max(2);
    let _ = Random::float();
    let _ = Random::long();
    assert_eq!(
        after_statue,
        Random::peek_ints(3),
        "parent stream is center jitter, armored float, and one child-seed long"
    );
    Random::pop_generator();

    assert_eq!(loot.heap_type, "statue");
    assert_eq!(loot.item.source.as_deref(), Some("StatueRoom"));
    assert!(!loot.item.cursed);
    assert_eq!(loot.item.category, ItemCategory::Weapon);
    assert!(loot.item.enchantment.is_some());
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "IronKey");
    assert_eq!(spawn[0].category, ItemCategory::Other);
    let statues: Vec<_> = map
        .known_mobs
        .iter()
        .enumerate()
        .filter_map(|(cell, mob)| matches!(*mob, Some("Statue" | "ArmoredStatue")).then_some(cell))
        .collect();
    assert_eq!(statues.len(), 1);
    assert!(map.mob_occupied[statues[0]]);
}

#[test]
fn mass_grave_rerolls_the_skeleton_loop_bound() {
    Random::reset_generators();
    let run = init_run(38);
    Random::push_generator_seeded(108);
    let mut d = dungeon_from_run(run);
    d.depth = 8;
    let mut room = test_room("MassGraveRoom", 11, 10);
    room.right = room.left + 10;
    room.bottom = room.top + 9;
    let mut spawn = Vec::new();
    let mut map = paint_minimal(std::slice::from_ref(&room)).expect("map");

    let loot = mass_grave_prizes(&mut d, &room, &mut map, &mut spawn);
    let tail = Random::peek_ints(4);
    Random::pop_generator();

    assert_eq!(spawn[0].class_name, "PotionOfLiquidFlame");
    assert!(!loot.is_empty());
    let skeletons: Vec<_> = map
        .known_mobs
        .iter()
        .enumerate()
        .filter_map(|(cell, mob)| (*mob == Some("Skeleton")).then_some(cell))
        .collect();
    assert!(
        (1..=2).contains(&skeletons.len()),
        "Java re-evaluates Int(2) as the skeleton loop bound"
    );
    for &cell in &skeletons {
        let y = map.origin_y + (cell as i32 / map.width);
        assert!(y <= room.top + 3);
    }
    for (cell, heap) in map.known_heaps.iter().enumerate() {
        if heap.is_some() {
            let y = map.origin_y + (cell as i32 / map.width);
            assert!(y <= room.top + 5);
        }
    }
    let cell = |x, y| map.point_to_cell(x, y).expect("cell");
    assert_eq!(map.map[cell(room.left + 5, room.top)], WALL_DECO);
    assert_eq!(map.map[cell(room.left + 3, room.top + 2)], STATUE);
    assert_eq!(map.map[cell(room.right - 3, room.top + 2)], STATUE);
    assert_eq!(map.map[cell(room.left + 1, room.top + 1)], WALL);
    assert_eq!(map.custom_tiles.len(), 1);
    assert_eq!(map.custom_tiles[0].class_name, "MassGraveDeco");
    assert_eq!(map.custom_tiles[0].width, 9);
    assert_eq!(map.custom_tiles[0].height, 9);
    assert_eq!(map.custom_terrain.len(), 2);
    assert!(map
        .custom_terrain
        .iter()
        .all(|layer| layer.class_name == "StatueRaised" && layer.static_data == [4]));
    assert_eq!(
        tail,
        [1_604_420_154, 1_430_481_976, -639_515_360, -416_095_344],
        "pinned MassGraveRoom skeleton-loop and loot boundary"
    );
}

#[test]
fn armory_room_pushes_iron_key() {
    Random::reset_generators();
    let run = init_run(37);
    Random::push_generator_seeded(107);
    let mut d = dungeon_from_run(run);
    d.depth = 13;
    let room = test_room("ArmoryRoom", 8, 8);
    let mut map = paint_minimal(std::slice::from_ref(&room)).expect("map");
    let entrance = crate::geom::Point::new(room.left, room.top + 2);
    let mut spawn = Vec::new();
    let loot = armory_prizes_on_map(&mut d, &room, &mut map, entrance, &mut spawn);
    Random::pop_generator();

    assert!((2..=3).contains(&loot.len()));
    assert!(loot
        .iter()
        .all(|p| p.item.source.as_deref() == Some("ArmoryRoom")));
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "IronKey");
    assert_eq!(spawn[0].category, ItemCategory::Other);
}

#[test]
fn pool_room_pushes_invisibility() {
    Random::reset_generators();
    let run = init_run(38);
    Random::push_generator_seeded(108);
    let mut d = dungeon_from_run(run);
    d.depth = 6;
    let room = test_room("PoolRoom", 8, 8);
    let mut spawn = Vec::new();
    let loot = pool_prize(&mut d, &room, &mut spawn);
    Random::pop_generator();

    assert_eq!(loot.heap_type, "chest");
    assert_eq!(loot.item.source.as_deref(), Some("PoolRoom"));
    assert!(!loot.item.cursed);
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "PotionOfInvisibility");
    assert_eq!(spawn[0].category, ItemCategory::Potion);
}

#[test]
fn secret_runestone_pushes_liquid_flame() {
    Random::reset_generators();
    let run = init_run(39);
    Random::push_generator_seeded(109);
    let mut d = dungeon_from_run(run);
    d.depth = 10;
    let room = test_room("SecretRunestoneRoom", 8, 8);
    let mut map = paint_minimal(std::slice::from_ref(&room)).unwrap();
    for y in (room.top + 1)..room.bottom {
        for x in (room.left + 1)..room.right {
            let cell = map.point_to_cell(x, y).unwrap();
            map.map[cell] = if x > room.left + room.width() / 2 {
                EMPTY_SP
            } else {
                EMPTY
            };
        }
    }
    let mut spawn = Vec::new();
    let loot = secret_runestone(&mut d, &room, &mut map, &mut spawn);
    Random::pop_generator();

    assert_eq!(loot.len(), 3);
    assert_eq!(loot[2].item.class_name, "StoneOfEnchantment");
    assert!(loot
        .iter()
        .all(|p| p.item.source.as_deref() == Some("SecretRunestoneRoom")));
    assert_eq!(spawn.len(), 1);
    assert_eq!(spawn[0].class_name, "PotionOfLiquidFlame");
    assert_eq!(spawn[0].category, ItemCategory::Potion);
}

#[test]
fn secret_laboratory_no_iron_key() {
    // Java SecretLaboratoryRoom extends SecretRoom with its own paint() that
    // never calls addItemToSpawn — reusing the lab prize body must not append
    // the LaboratoryRoom IronKey here.
    Random::reset_generators();
    let _run = init_run(40);
    Random::push_generator_seeded(110);
    let room = test_room("SecretLaboratoryRoom", 8, 8);
    let mut map = paint_minimal(std::slice::from_ref(&room)).expect("secret lab map");
    let loot = secret_laboratory(&room, &mut map);
    Random::pop_generator();

    assert!((4..=5).contains(&loot.len()));
    assert!(loot
        .iter()
        .all(|p| p.item.source.as_deref() == Some("SecretLaboratoryRoom")));
    assert!(loot
        .iter()
        .filter(|p| p.item.class_name == "EnergyCrystal")
        .all(|p| (3..=5).contains(&p.item.quantity)));
    assert_eq!(
        map.known_heaps
            .iter()
            .flatten()
            .flat_map(|heap| &heap.items)
            .filter(|item| item.class_name == "EnergyCrystal")
            .count(),
        2
    );
}

#[test]
fn secret_larder_uses_pinned_food_energy_units() {
    let cases = [
        (4, vec!["ChargrilledMeat"]),
        (6, vec!["ChargrilledMeat", "ChargrilledMeat"]),
        (10, vec!["Pasty"]),
    ];

    for (depth, expected) in cases {
        Random::reset_generators();
        Random::push_generator_seeded(200 + i64::from(depth));
        let room = test_room("SecretLarderRoom", 8, 8);
        let mut map = paint_minimal(std::slice::from_ref(&room)).expect("secret larder map");

        let loot = secret_larder(depth, &room, &mut map, false);

        Random::pop_generator();
        assert_eq!(
            loot.iter()
                .map(|drop| drop.item.class_name.as_str())
                .collect::<Vec<_>>(),
            expected,
            "depth {depth} food composition"
        );
        assert_eq!(
            map.known_heaps.iter().flatten().count(),
            loot.len(),
            "depth {depth} heap count"
        );
    }
}
