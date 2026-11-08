use super::*;

use std::ffi::OsStr;

#[test]
fn aaa_replay_pins_floors_six_through_eleven_across_the_tengu_lifecycle() {
    let _journal = crate::fresh_journal();
    let fixtures: Vec<_> = [6, 7, 8, 9, 11]
        .into_iter()
        .map(|depth| {
            let name = format!("aaa-aaa-aaa-final-heaps-floor-{depth}.json");
            let path = fixture_paths()
                .into_iter()
                .find(|path| {
                    path.file_name()
                        .is_some_and(|file| file == OsStr::new(&name))
                })
                .unwrap_or_else(|| panic!("missing AAA floor-{depth} fixture"));
            read_fixture(&path)
        })
        .collect();

    let mut dungeon = dungeon_from_run(init_run(fixtures[0].input.numeric));
    for depth in 1_u32..=11 {
        dungeon.depth = depth as i32;
        let actual = create_level_partial(&mut dungeon);
        if depth < 6 || depth == 10 {
            continue;
        }
        let fixture_index = if depth == 11 { 4 } else { (depth - 6) as usize };
        let fixture = &fixtures[fixture_index];
        let expected = fixture.floors.first().expect("replay oracle floor");
        let context = format!("AAA floor-{depth}");
        assert_eq!(
            fixture.schema_version, EXTENDED_FINAL_HEAPS_SCHEMA_VERSION,
            "{context} schema"
        );
        assert_eq!(
            fixture.contract.as_deref(),
            Some("final_placed_heaps"),
            "{context} contract"
        );
        assert_eq!(fixture.input.depths, [depth], "{context} requested depth");
        assert_eq!(expected.depth, depth, "{context} floor depth");

        let map = actual.map.as_ref().expect("regular replay map");
        let mut rooms = actual.rooms.clone();
        rooms.sort();
        assert_eq!(rooms, expected.rooms, "{context} room classes");
        let bounds: Vec<_> = actual
            .room_bounds
            .iter()
            .map(|room| OracleRoomFact {
                class_name: room.class_name.clone(),
                left: room.left,
                top: room.top,
                right: room.right,
                bottom: room.bottom,
            })
            .collect();
        assert_eq!(bounds, expected.room_bounds, "{context} room bounds");
        assert_eq!(
            (map.width, map.height),
            (expected.width, expected.height),
            "{context} map bounds"
        );
        assert_eq!(
            actual.pre_paint_rng_probe, expected.pre_paint_rng,
            "{context} pre-paint RNG"
        );
        assert_eq!(
            expected.pre_mobs_rng.len(),
            8,
            "{context} pinned pre-mobs RNG"
        );
        assert_eq!(
            expected.pre_items_rng.len(),
            8,
            "{context} pinned pre-items RNG"
        );
        assert_eq!(
            expected.terrain.as_ref().map(Vec::len),
            Some(map.tiles.len()),
            "{context} pinned terrain"
        );
        assert_eq!(
            expected.discoverable.as_ref().map(Vec::len),
            Some(map.tiles.len()),
            "{context} pinned discoverability"
        );
        assert_eq!(
            expected.tile_variance.as_ref().map(Vec::len),
            Some(map.tiles.len()),
            "{context} pinned variance"
        );
        assert!(
            !expected.final_heaps.is_empty(),
            "{context} pinned final heaps"
        );
        assert!(
            !expected.final_mobs.is_empty(),
            "{context} pinned final mobs"
        );

        if matches!(depth, 6 | 8 | 9 | 11) {
            assert_eq!(
                actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
                "{context} pre-mobs RNG"
            );
            assert_eq!(
                actual.pre_items_rng_probe, expected.pre_items_rng,
                "{context} pre-items RNG"
            );
        }

        if depth == 7 {
            let oracle = expected.terrain.as_ref().unwrap();
            let garden = expected
                .room_bounds
                .iter()
                .find(|room| room.class_name == "GardenRoom")
                .expect("floor-7 garden bounds");
            let mut actual_cells = Vec::new();
            let mut oracle_cells = Vec::new();
            for y in garden.top..=garden.bottom {
                for x in garden.left..=garden.right {
                    let cell = (y * expected.width as i32 + x) as usize;
                    actual_cells.push(map.tiles[cell]);
                    oracle_cells.push(oracle[cell]);
                }
            }
            assert_eq!(
                actual_cells, oracle_cells,
                "{context} full GardenRoom terrain"
            );
            assert_armory_boundary(map, expected, &context);
            assert_library(map, expected, &context);
            assert_eq!(
                actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
                "{context} pre-mobs RNG after PitRoom crystal-door paint"
            );
            assert_eq!(
                actual.pre_items_rng_probe, expected.pre_items_rng,
                "{context} pre-items RNG after quest hook and createMobs"
            );
            let actual_mobs: Vec<_> = map
                .mobs
                .iter()
                .map(|mob| OracleMob {
                    cell: mob.cell,
                    class_name: mob.class_name().to_string(),
                })
                .collect();
            assert_eq!(actual_mobs, expected.final_mobs, "{context} exact mobs");
            let pit_heaps: Vec<_> = map
                .heaps
                .iter()
                .filter(|heap| heap.cell == 359)
                .map(|heap| OracleHeap {
                    cell: heap.cell,
                    heap_type: heap.heap_type.clone(),
                    items: heap
                        .items
                        .iter()
                        .map(|item| OracleItem {
                            class_name: item.class_name().to_string(),
                            quantity: item.quantity,
                            level: item.level,
                            cursed: item.cursed,
                        })
                        .collect(),
                })
                .collect();
            let expected_pit_heaps: Vec<_> = expected
                .final_heaps
                .iter()
                .filter(|heap| heap.cell == 359)
                .map(|heap| OracleHeap {
                    cell: heap.cell,
                    heap_type: heap.heap_type.clone(),
                    items: heap
                        .items
                        .iter()
                        .map(|item| OracleItem {
                            class_name: item.class_name.clone(),
                            quantity: item.quantity,
                            level: item.level,
                            cursed: item.cursed,
                        })
                        .collect(),
                })
                .collect();
            assert_eq!(
                pit_heaps, expected_pit_heaps,
                "{context} exact PitRoom heap"
            );
            let statue_room = expected
                .room_bounds
                .iter()
                .find(|room| room.class_name == "StatueRoom")
                .unwrap_or_else(|| panic!("{context} fixture StatueRoom bounds"));
            let statue = expected
                .final_mobs
                .iter()
                .find(|mob| mob.class_name == "Statue")
                .unwrap_or_else(|| panic!("{context} fixture StatueRoom mob"));
            let statue_x = (statue.cell % expected.width) as i32;
            let statue_y = (statue.cell / expected.width) as i32;
            assert!(
                statue_x > statue_room.left
                    && statue_x < statue_room.right
                    && statue_y > statue_room.top
                    && statue_y < statue_room.bottom,
                "{context} fixture StatueRoom mob is inside its room"
            );
            assert!(
                actual_mobs.contains(statue),
                "{context} pinned StatueRoom mob from fixture"
            );
        }

        if matches!(depth, 8 | 9 | 11) {
            let actual_mobs: Vec<_> = map
                .mobs
                .iter()
                .map(|mob| OracleMob {
                    cell: mob.cell,
                    class_name: mob.class_name().to_string(),
                })
                .collect();
            assert_eq!(actual_mobs, expected.final_mobs, "{context} exact mobs");

            let actual_heaps: Vec<_> = map
                .heaps
                .iter()
                .map(|heap| OracleHeap {
                    cell: heap.cell,
                    heap_type: heap.heap_type.clone(),
                    items: heap
                        .items
                        .iter()
                        .map(|item| OracleItem {
                            class_name: oracle_item_class(item.class_name()).into(),
                            quantity: item.quantity,
                            level: item.level,
                            cursed: item.cursed,
                        })
                        .collect(),
                })
                .collect();
            if depth == 11 {
                // ChooseBag ties depend on JVM identity-hash iteration. Keep
                // that single class portable while pinning every heap cell,
                // type, and all deterministic item facts on this floor.
                let deterministic_actual: Vec<_> = actual_heaps
                    .iter()
                    .filter(|heap| heap.cell != 267)
                    .collect();
                let deterministic_expected: Vec<_> = expected
                    .final_heaps
                    .iter()
                    .filter(|heap| heap.cell != 267)
                    .collect();
                assert_eq!(
                    deterministic_actual, deterministic_expected,
                    "{context} exact deterministic heaps"
                );
                for heaps in [&actual_heaps, &expected.final_heaps] {
                    let bag = heaps
                        .iter()
                        .find(|heap| heap.cell == 267)
                        .expect("floor-11 identity-hash-dependent shop bag");
                    assert_eq!(bag.heap_type, "for_sale");
                    assert_eq!(
                        (
                            bag.items[0].quantity,
                            bag.items[0].level,
                            bag.items[0].cursed
                        ),
                        (1, 0, false)
                    );
                }
                let hoard = expected
                    .room_bounds
                    .iter()
                    .find(|room| room.class_name == "SecretHoardRoom")
                    .expect("floor-11 SecretHoardRoom bounds");
                let in_hoard = |heap: &OracleHeap| {
                    let x = heap.cell as i32 % expected.width as i32;
                    let y = heap.cell as i32 / expected.width as i32;
                    x > hoard.left && x < hoard.right && y > hoard.top && y < hoard.bottom
                };
                let actual_hoard: Vec<_> =
                    actual_heaps.iter().filter(|heap| in_hoard(heap)).collect();
                let expected_hoard: Vec<_> = expected
                    .final_heaps
                    .iter()
                    .filter(|heap| in_hoard(heap))
                    .collect();
                assert_eq!(actual_hoard, expected_hoard, "{context} exact hoard heaps");
            } else {
                assert_eq!(actual_heaps, expected.final_heaps, "{context} exact heaps");
            }
        }

        if depth == 6 {
            let is_weapon =
                |class_name: &str| matches!(class_name, "Quarterstaff" | "HandAxe" | "Katana");
            let weapon_heaps: Vec<_> = map
                .heaps
                .iter()
                .filter(|heap| heap.items.iter().any(|item| is_weapon(item.class_name())))
                .map(|heap| OracleHeap {
                    cell: heap.cell,
                    heap_type: heap.heap_type.clone(),
                    items: heap
                        .items
                        .iter()
                        .map(|item| OracleItem {
                            class_name: item.class_name().to_string(),
                            quantity: item.quantity,
                            level: item.level,
                            cursed: item.cursed,
                        })
                        .collect(),
                })
                .collect();
            let expected_weapon_heaps: Vec<_> = expected
                .final_heaps
                .iter()
                .filter(|heap| heap.items.iter().any(|item| is_weapon(&item.class_name)))
                .map(|heap| OracleHeap {
                    cell: heap.cell,
                    heap_type: heap.heap_type.clone(),
                    items: heap
                        .items
                        .iter()
                        .map(|item| OracleItem {
                            class_name: item.class_name.clone(),
                            quantity: item.quantity,
                            level: item.level,
                            cursed: item.cursed,
                        })
                        .collect(),
                })
                .collect();
            assert_eq!(
                weapon_heaps, expected_weapon_heaps,
                "{context} v4 shop, main-drop, and chest weapon heaps"
            );
            let vault_heaps: Vec<_> = map
                .heaps
                .iter()
                .filter(|heap| heap.heap_type == "crystal_chest")
                .map(|heap| {
                    (
                        heap.cell,
                        heap.items[0].class_name(),
                        heap.items[0].level,
                        heap.items[0].cursed,
                    )
                })
                .collect();
            let expected_vault_heaps: Vec<_> = expected
                .final_heaps
                .iter()
                .filter(|heap| heap.heap_type == "crystal_chest")
                .map(|heap| {
                    (
                        heap.cell,
                        heap.items[0].class_name.as_str(),
                        heap.items[0].level,
                        heap.items[0].cursed,
                    )
                })
                .collect();
            assert_eq!(
                vault_heaps, expected_vault_heaps,
                "{context} CrystalVaultRoom chests"
            );
            let actual_mimics: Vec<_> = map
                .mobs
                .iter()
                .filter(|mob| mob.class_name() == "CrystalMimic")
                .map(|mob| mob.cell)
                .collect();
            let expected_mimics: Vec<_> = expected
                .final_mobs
                .iter()
                .filter(|mob| mob.class_name == "CrystalMimic")
                .map(|mob| mob.cell)
                .collect();
            assert_eq!(
                actual_mimics, expected_mimics,
                "{context} CrystalVault mimics"
            );
            assert_crystal_vault_terrain(map, expected, &context);
        }
    }
}

#[test]
fn aaa_floor_twelve_matches_the_pinned_layout_and_item_lifecycle() {
    let _journal = crate::fresh_journal();
    let name = OsStr::new("aaa-aaa-aaa-final-heaps-floor-12.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .expect("missing AAA floor-12 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-12 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1_i32..=12 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-12 replay");
    let mut actual_rooms = actual.rooms.clone();
    actual_rooms.sort();

    let is_connection = |name: &&String| {
        matches!(
            name.as_str(),
            "TunnelRoom"
                | "BridgeRoom"
                | "PerimeterRoom"
                | "WalkwayRoom"
                | "RingTunnelRoom"
                | "RingBridgeRoom"
                | "MazeConnectionRoom"
        )
    };
    let actual_selected: Vec<_> = actual_rooms
        .iter()
        .filter(|name| !is_connection(name))
        .cloned()
        .collect();
    let expected_selected: Vec<_> = expected
        .rooms
        .iter()
        .filter(|name| !is_connection(name))
        .cloned()
        .collect();

    assert_eq!(
        actual_selected, expected_selected,
        "floor-12 room selection is aligned before builder-added connections"
    );
    assert_eq!(actual_rooms, expected.rooms, "floor-12 room classes");
    let bounds: Vec<_> = actual
        .room_bounds
        .iter()
        .map(|room| OracleRoomFact {
            class_name: room.class_name.clone(),
            left: room.left,
            top: room.top,
            right: room.right,
            bottom: room.bottom,
        })
        .collect();
    assert_eq!(bounds, expected.room_bounds, "floor-12 room bounds");
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "floor-12 pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "floor-12 pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "floor-12 pre-items RNG boundary"
    );
    let actual_heaps: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-12 map")
        .heaps
        .iter()
        .map(|heap| OracleHeap {
            cell: heap.cell,
            heap_type: heap.heap_type.clone(),
            items: heap
                .items
                .iter()
                .map(|item| OracleItem {
                    class_name: oracle_item_class(item.class_name()).into(),
                    quantity: item.quantity,
                    level: item.level,
                    cursed: item.cursed,
                })
                .collect(),
        })
        .collect();
    assert_eq!(
        actual_heaps, expected.final_heaps,
        "floor-12 normalized heaps"
    );
}

#[test]
fn aaa_floor_thirteen_matches_the_pinned_final_facts() {
    let _journal = crate::fresh_journal();
    let name = OsStr::new("aaa-aaa-aaa-final-heaps-floor-13.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .expect("missing AAA floor-13 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-13 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1_i32..=13 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-13 replay");
    let mut actual_rooms = actual.rooms.clone();
    actual_rooms.sort();
    assert_eq!(actual_rooms, expected.rooms, "floor-13 room classes");

    let bounds: Vec<_> = actual
        .room_bounds
        .iter()
        .map(|room| OracleRoomFact {
            class_name: room.class_name.clone(),
            left: room.left,
            top: room.top,
            right: room.right,
            bottom: room.bottom,
        })
        .collect();
    assert_eq!(bounds, expected.room_bounds, "floor-13 room bounds");
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "floor-13 pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "floor-13 pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "floor-13 pre-items RNG boundary"
    );
    let actual_mobs: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-13 map")
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(actual_mobs, expected.final_mobs, "floor-13 final mobs");

    let actual_heaps: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-13 map")
        .heaps
        .iter()
        .map(|heap| OracleHeap {
            cell: heap.cell,
            heap_type: heap.heap_type.clone(),
            items: heap
                .items
                .iter()
                .map(|item| OracleItem {
                    class_name: oracle_item_class(item.class_name()).into(),
                    quantity: item.quantity,
                    level: item.level,
                    cursed: item.cursed,
                })
                .collect(),
        })
        .collect();
    assert_eq!(actual_heaps, expected.final_heaps, "floor-13 final heaps");
}

#[test]
fn aaa_floor_fourteen_matches_the_pinned_final_facts() {
    let _journal = crate::fresh_journal();
    let name = OsStr::new("aaa-aaa-aaa-final-heaps-floor-14.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .expect("missing AAA floor-14 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-14 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1_i32..=14 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-14 replay");
    let mut actual_rooms = actual.rooms.clone();
    actual_rooms.sort();
    assert_eq!(actual_rooms, expected.rooms, "floor-14 room classes");

    let bounds: Vec<_> = actual
        .room_bounds
        .iter()
        .map(|room| OracleRoomFact {
            class_name: room.class_name.clone(),
            left: room.left,
            top: room.top,
            right: room.right,
            bottom: room.bottom,
        })
        .collect();
    assert_eq!(bounds, expected.room_bounds, "floor-14 room bounds");
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "floor-14 pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "floor-14 pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "floor-14 pre-items RNG boundary"
    );
    let map = actual.map.as_ref().expect("floor-14 map");
    let toxic_vents: Vec<_> = map
        .traps
        .iter()
        .filter(|trap| trap.class_name() == "ToxicVent")
        .map(|trap| OracleTrap {
            cell: trap.cell,
            class_name: trap.class_name().to_string(),
            visible: trap.visible,
            active: trap.active,
            color: trap.color,
            shape: trap.shape,
        })
        .collect();
    let expected_toxic_vents: Vec<_> = expected
        .traps
        .as_ref()
        .expect("floor-14 traps")
        .iter()
        .filter(|trap| trap.class_name == "ToxicVent")
        .map(|trap| OracleTrap {
            cell: trap.cell,
            class_name: trap.class_name.clone(),
            visible: trap.visible,
            active: trap.active,
            color: trap.color,
            shape: trap.shape,
        })
        .collect();
    assert_eq!(
        toxic_vents, expected_toxic_vents,
        "floor-14 v4 ToxicGasRoom vents"
    );
    let actual_mobs: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-14 map")
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(actual_mobs, expected.final_mobs, "floor-14 final mobs");

    let actual_heaps: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-14 map")
        .heaps
        .iter()
        .map(|heap| OracleHeap {
            cell: heap.cell,
            heap_type: heap.heap_type.clone(),
            items: heap
                .items
                .iter()
                .map(|item| OracleItem {
                    class_name: oracle_item_class(item.class_name()).into(),
                    quantity: item.quantity,
                    level: item.level,
                    cursed: item.cursed,
                })
                .collect(),
        })
        .collect();
    assert_eq!(actual_heaps, expected.final_heaps, "floor-14 final heaps");
}

fn oracle_item_class(class_name: &str) -> &str {
    // Java's `getSimpleName()` cannot distinguish nested Plant.Seed classes.
    match class_name {
        "RotberrySeed" | "SungrassSeed" | "FadeleafSeed" | "IcecapSeed" | "FirebloomSeed"
        | "SorrowmossSeed" | "SwiftthistleSeed" | "BlindweedSeed" | "StormvineSeed"
        | "EarthrootSeed" | "MageroyalSeed" | "StarflowerSeed" => "Seed",
        _ => class_name,
    }
}

fn assert_library(map: &spd_core::report::FloorMap, expected: &OracleFloor, context: &str) {
    let room = expected
        .room_bounds
        .iter()
        .find(|room| room.class_name == "LibraryRoom")
        .unwrap_or_else(|| panic!("{context} LibraryRoom bounds"));
    let oracle = expected.terrain.as_ref().expect("pinned terrain");

    // EMPTY_SP floors may be decorated later, while the wall/bookshelf shell
    // and entrance passage persist through the completed painter.
    for y in room.top..=room.bottom {
        for x in room.left..=room.right {
            let cell = (y * expected.width as i32 + x) as usize;
            if matches!(oracle[cell], 0 | 10 | 27) {
                assert_eq!(
                    map.tiles[cell], oracle[cell],
                    "{context} Library cell {cell}"
                );
            }
        }
    }

    let heaps: Vec<_> = map
        .heaps
        .iter()
        .filter(|heap| {
            let x = heap.cell as i32 % expected.width as i32;
            let y = heap.cell as i32 / expected.width as i32;
            x > room.left && x < room.right && y > room.top && y < room.bottom
        })
        .map(|heap| {
            (
                heap.cell,
                heap.heap_type.as_str(),
                heap.items[0].class_name(),
            )
        })
        .collect();
    let oracle_heaps: Vec<_> = expected
        .final_heaps
        .iter()
        .filter(|heap| {
            let x = heap.cell as i32 % expected.width as i32;
            let y = heap.cell as i32 / expected.width as i32;
            x > room.left && x < room.right && y > room.top && y < room.bottom
        })
        .map(|heap| {
            (
                heap.cell,
                heap.heap_type.as_str(),
                heap.items[0].class_name.as_str(),
            )
        })
        .collect();
    assert!(
        !oracle_heaps.is_empty(),
        "{context} pinned Java Library prizes"
    );
    assert_eq!(heaps, oracle_heaps, "{context} exact Library prizes");
}

fn assert_armory_boundary(map: &spd_core::report::FloorMap, expected: &OracleFloor, context: &str) {
    let room = expected
        .room_bounds
        .iter()
        .find(|room| room.class_name == "ArmoryRoom")
        .unwrap_or_else(|| panic!("{context} ArmoryRoom bounds"));
    let oracle = expected.terrain.as_ref().expect("pinned terrain");

    // Later water/grass/trap/decorate passes may replace EMPTY tiles. Pin the
    // Armory painter's persistent boundary, locked entrance, and statue.
    for y in room.top..=room.bottom {
        for x in room.left..=room.right {
            let cell = (y * expected.width as i32 + x) as usize;
            if x == room.left
                || x == room.right
                || y == room.top
                || y == room.bottom
                || oracle[cell] == 25
            {
                assert_eq!(
                    map.tiles[cell], oracle[cell],
                    "{context} Armory cell {cell}"
                );
            }
        }
    }

    let heaps: Vec<_> = map
        .heaps
        .iter()
        .filter(|heap| {
            let x = heap.cell as i32 % expected.width as i32;
            let y = heap.cell as i32 / expected.width as i32;
            x > room.left && x < room.right && y > room.top && y < room.bottom
        })
        .map(|heap| {
            let item = &heap.items[0];
            (
                heap.cell,
                heap.heap_type.as_str(),
                item.class_name(),
                item.quantity,
                item.level,
                item.cursed,
            )
        })
        .collect();
    let expected_heaps: Vec<_> = expected
        .final_heaps
        .iter()
        .filter(|heap| {
            let x = heap.cell as i32 % expected.width as i32;
            let y = heap.cell as i32 / expected.width as i32;
            x > room.left && x < room.right && y > room.top && y < room.bottom
        })
        .map(|heap| {
            let item = &heap.items[0];
            (
                heap.cell,
                heap.heap_type.as_str(),
                item.class_name.as_str(),
                item.quantity,
                item.level,
                item.cursed,
            )
        })
        .collect();
    assert_eq!(heaps, expected_heaps, "{context} Armory prize placement");
}

fn assert_crystal_vault_terrain(
    map: &spd_core::report::FloorMap,
    expected: &OracleFloor,
    context: &str,
) {
    let vault = expected
        .room_bounds
        .iter()
        .find(|room| room.class_name == "CrystalVaultRoom")
        .expect("floor-6 vault bounds");
    assert_eq!(
        (vault.right - vault.left, vault.bottom - vault.top),
        (6, 6),
        "{context} fixed 7x7 vault bounds"
    );
    let oracle = expected.terrain.as_ref().expect("floor-6 terrain");
    let mut actual_cells = Vec::new();
    let mut oracle_cells = Vec::new();
    let mut locked = Vec::new();
    let mut pedestals = Vec::new();
    let mut actual_pedestals = Vec::new();
    for y in vault.top..=vault.bottom {
        for x in vault.left..=vault.right {
            let cell = (y * expected.width as i32 + x) as usize;
            let tile = oracle[cell];
            actual_cells.push(map.tiles[cell]);
            oracle_cells.push(tile);
            if map.tiles[cell] == 11 {
                actual_pedestals.push(cell);
            }
            if x == vault.left || x == vault.right || y == vault.top || y == vault.bottom {
                assert!(
                    matches!(tile, 4 | 10 | 12),
                    "{context} vault boundary cell {cell}"
                );
            } else if x == vault.left + 1
                || x == vault.right - 1
                || y == vault.top + 1
                || y == vault.bottom - 1
            {
                assert_eq!(tile, 14, "{context} EMPTY_SP inset at cell {cell}");
            }
            if tile == 10 {
                locked.push(cell);
            } else if tile == 11 {
                pedestals.push(cell);
            }
        }
    }
    assert_eq!(actual_cells, oracle_cells, "{context} full vault terrain");
    assert_eq!(locked.len(), 1, "{context} locked vault entrance");
    assert_eq!(actual_pedestals, pedestals, "{context} vault pedestals");
}
