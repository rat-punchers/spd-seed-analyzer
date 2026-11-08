use super::*;

fn fixture() -> OracleFixture {
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "hkt-jzn-xqq-final-heaps-floor-6.json")
        })
        .expect("HKT floor-6 schema-v3 fixture");
    read_fixture(&path)
}

#[test]
fn gfx_floor_six_secret_library_restores_preserved_run_lifecycle() {
    let _journal = crate::fresh_journal();
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "gfx-pzh-dch-final-heaps-floor-6.json")
        })
        .expect("GFX floor-6 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-6 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1..=6 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-6 replay");
    let actual_bounds = actual
        .room_bounds
        .iter()
        .map(|room| OracleRoomFact {
            class_name: room.class_name.clone(),
            left: room.left,
            top: room.top,
            right: room.right,
            bottom: room.bottom,
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_bounds, expected.room_bounds, "GFX floor-6 bounds");
    assert_eq!(actual.pre_paint_rng_probe, expected.pre_paint_rng);
    assert_eq!(actual.pre_mobs_rng_probe, expected.pre_mobs_rng);
    assert_eq!(actual.pre_items_rng_probe, expected.pre_items_rng);

    let map = actual.map.as_ref().expect("GFX floor-6 map");
    let actual_mobs = map
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_mobs, expected.final_mobs, "GFX floor-6 mobs");

    let secret_library_cells = [1380, 1428, 1527];
    let secret_library_drops = map
        .heaps
        .iter()
        .filter(|heap| secret_library_cells.contains(&heap.cell))
        .map(|heap| (heap.cell, heap.items[0].class_name()))
        .collect::<Vec<_>>();
    assert_eq!(
        secret_library_drops,
        [(1380, "ScrollOfTransmutation")],
        "GFX floor-6 SecretLibrary private weighted draws"
    );
}

#[test]
fn hkt_floor_six_lifecycle_matches_oracle() {
    let _journal = crate::fresh_journal();
    let fixture = fixture();
    let expected = fixture.floors.first().expect("floor-6 oracle facts");
    assert_eq!(fixture.schema_version, EXTENDED_FINAL_HEAPS_SCHEMA_VERSION);
    assert_eq!(fixture.contract.as_deref(), Some("final_placed_heaps"));
    assert_eq!(fixture.input.depths, [6]);
    assert_eq!(fixture.floors.len(), 1);
    assert_eq!(expected.depth, 6);
    assert_eq!((expected.width, expected.height), (55, 48));
    assert_eq!(expected.pre_paint_rng.len(), 8);
    assert_eq!(expected.pre_mobs_rng.len(), 8);
    assert_eq!(expected.pre_items_rng.len(), 8);
    assert_eq!(expected.terrain.as_ref().map(Vec::len), Some(55 * 48));
    assert_eq!(expected.discoverable.as_ref().map(Vec::len), Some(55 * 48));
    assert_eq!(expected.tile_variance.as_ref().map(Vec::len), Some(55 * 48));
    assert_eq!(expected.final_heaps.len(), 34);
    assert_eq!(
        expected
            .final_heaps
            .iter()
            .filter(|heap| heap.heap_type == "for_sale")
            .count(),
        20
    );
    assert_eq!(expected.final_mobs.len(), 6);
    assert_eq!(expected.transitions.as_ref().map(Vec::len), Some(2));
    assert_eq!(expected.traps.as_ref().map(Vec::len), Some(2));
    assert_eq!(expected.plants.as_ref().map(Vec::len), Some(0));
    assert_eq!(expected.blobs.as_ref().map(Vec::len), Some(0));

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1..=6 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-6 Rust facts");
    let map = actual.map.as_ref().expect("floor-6 regular map");
    let mut actual_rooms = actual.rooms.clone();
    actual_rooms.sort();

    assert_eq!(actual.feeling.as_str(), "water", "HKT floor-6 feeling");
    assert_eq!(actual_rooms, expected.rooms, "HKT floor-6 room classes");
    assert_eq!(
        (map.width, map.height),
        (expected.width, expected.height),
        "HKT floor-6 map bounds"
    );
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "HKT floor-6 pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "HKT floor-6 pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "HKT floor-6 pre-items RNG boundary"
    );

    let terrain = expected.terrain.as_ref().expect("floor-6 terrain");
    let terrain_mismatches: Vec<_> = map
        .tiles
        .iter()
        .zip(terrain)
        .enumerate()
        .filter_map(|(cell, (&actual, &expected))| {
            (actual != expected).then_some((cell, actual, expected))
        })
        .collect();
    assert!(
        terrain_mismatches.is_empty(),
        "HKT floor-6 terrain mismatches: {terrain_mismatches:?}"
    );
    assert_eq!(
        map.discoverable,
        *expected
            .discoverable
            .as_ref()
            .expect("floor-6 discoverable mask"),
        "HKT floor-6 discoverable mask"
    );
    assert_eq!(
        map.tile_variance,
        *expected
            .tile_variance
            .as_ref()
            .expect("floor-6 tile variance"),
        "HKT floor-6 tile variance"
    );

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
                    class_name: item.class_name().to_string(),
                    quantity: item.quantity,
                    level: item.level,
                    cursed: item.cursed,
                })
                .collect(),
        })
        .collect();
    assert_eq!(actual_heaps, expected.final_heaps, "HKT floor-6 heaps");

    let actual_mobs: Vec<_> = map
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(actual_mobs, expected.final_mobs, "HKT floor-6 mobs");

    let actual_transitions: Vec<_> = map
        .transitions
        .iter()
        .map(|transition| OracleTransition {
            cell: transition.cell,
            transition_type: transition.transition_type.clone(),
            left: transition.left,
            top: transition.top,
            right: transition.right,
            bottom: transition.bottom,
            dest_depth: transition.dest_depth,
            dest_branch: transition.dest_branch,
            dest_type: transition.dest_type.clone(),
        })
        .collect();
    assert_eq!(
        actual_transitions,
        *expected.transitions.as_ref().expect("floor-6 transitions"),
        "HKT floor-6 transitions"
    );

    let actual_traps: Vec<_> = map
        .traps
        .iter()
        .map(|trap| OracleTrap {
            cell: trap.cell,
            class_name: trap.class_name().to_string(),
            visible: trap.visible,
            active: trap.active,
            color: trap.color,
            shape: trap.shape,
        })
        .collect();
    assert_eq!(
        actual_traps,
        *expected.traps.as_ref().expect("floor-6 traps"),
        "HKT floor-6 traps"
    );
    assert!(map.plants.is_empty(), "HKT floor-6 plants");
    assert!(map.blobs.is_empty(), "HKT floor-6 blobs");
}
