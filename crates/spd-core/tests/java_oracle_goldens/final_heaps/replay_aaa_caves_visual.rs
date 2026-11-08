use super::*;

use std::ffi::OsStr;

#[test]
fn aaa_caves_replay_matches_complete_pre_items_visual_facts() {
    let _journal = crate::fresh_journal();
    let fixtures: Vec<_> = [11, 12, 13, 14]
        .into_iter()
        .map(fixture_for_depth)
        .collect();
    let mut dungeon = dungeon_from_run(init_run(fixtures[0].input.numeric));

    for depth in 1_i32..=14 {
        dungeon.depth = depth;
        let actual = create_level_partial(&mut dungeon);
        if depth < 11 {
            continue;
        }

        let fixture = &fixtures[(depth - 11) as usize];
        let expected = fixture.floors.first().expect("Caves oracle floor");
        let map = actual.map.as_ref().expect("Caves regular map");
        let context = format!("AAA Caves floor-{depth}");

        assert_eq!(
            fixture.input.depths,
            [depth as u32],
            "{context} requested depth"
        );
        assert_eq!(expected.depth, depth as u32, "{context} floor depth");
        assert_eq!(
            actual.pre_items_rng_probe, expected.pre_items_rng,
            "{context} pre-items RNG boundary"
        );
        assert_eq!(
            (map.width, map.height),
            (expected.width, expected.height),
            "{context} map bounds"
        );
        assert_terrain(map, expected, &context);
        assert_eq!(
            map.discoverable,
            *expected
                .discoverable
                .as_ref()
                .expect("Caves discoverability"),
            "{context} discoverability"
        );
        assert_eq!(
            map.tile_variance,
            *expected
                .tile_variance
                .as_ref()
                .expect("Caves tile variance"),
            "{context} tile variance"
        );
        assert_eq!(
            transitions(map),
            *expected.transitions.as_ref().expect("Caves transitions"),
            "{context} transitions"
        );
        assert_eq!(
            traps(map),
            *expected.traps.as_ref().expect("Caves traps"),
            "{context} traps including metadata"
        );
        assert_eq!(
            plants(map),
            *expected.plants.as_ref().expect("Caves plants"),
            "{context} plants"
        );
        assert_eq!(
            blobs(map),
            *expected.blobs.as_ref().expect("Caves blobs"),
            "{context} blobs"
        );
        assert_eq!(
            map.custom_tiles.len(),
            expected.custom_tiles.len(),
            "{context} custom tiles"
        );
        assert_eq!(
            map.custom_walls.len(),
            expected.custom_walls.len(),
            "{context} custom walls"
        );
        for (actual, expected) in map.custom_tiles.iter().zip(&expected.custom_tiles) {
            assert_eq!(actual.class_name, expected.class_name, "{context}");
            assert_eq!(actual.texture, "caves_quest", "{context}");
            assert_eq!(
                (actual.x, actual.y, actual.width, actual.height),
                (expected.x, expected.y, expected.width, expected.height),
                "{context}"
            );
            assert_eq!(
                actual.static_data.len(),
                (actual.width * actual.height) as usize,
                "{context}"
            );
        }
        for (actual, expected) in map.custom_walls.iter().zip(&expected.custom_walls) {
            assert_eq!(actual.class_name, expected.class_name, "{context}");
            assert_eq!(actual.texture, "caves_quest", "{context}");
            assert_eq!(
                (actual.x, actual.y, actual.width, actual.height),
                (expected.x, expected.y, expected.width, expected.height),
                "{context}"
            );
            assert_eq!(actual.static_data, [3], "{context}");
        }
    }
}

fn fixture_for_depth(depth: i32) -> OracleFixture {
    let name = format!("aaa-aaa-aaa-final-heaps-floor-{depth}.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|file| file == OsStr::new(&name))
        })
        .unwrap_or_else(|| panic!("missing AAA Caves floor-{depth} fixture"));
    read_fixture(&path)
}

fn assert_terrain(map: &spd_core::report::FloorMap, expected: &OracleFloor, context: &str) {
    let terrain = expected.terrain.as_ref().expect("Caves terrain");
    let mut mismatch_pairs = std::collections::BTreeMap::new();
    for (&actual, &oracle) in map.tiles.iter().zip(terrain) {
        if actual != oracle {
            *mismatch_pairs.entry((actual, oracle)).or_insert(0_usize) += 1;
        }
    }
    let mismatches: Vec<_> = map
        .tiles
        .iter()
        .zip(terrain)
        .enumerate()
        .filter_map(|(cell, (&actual, &oracle))| {
            (actual != oracle).then_some((cell, actual, oracle))
        })
        .take(32)
        .collect();
    assert!(
        mismatches.is_empty(),
        "{context} raw terrain mismatches: {mismatches:?}; pair counts: {mismatch_pairs:?}"
    );
}

fn transitions(map: &spd_core::report::FloorMap) -> Vec<OracleTransition> {
    map.transitions
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
        .collect()
}

fn traps(map: &spd_core::report::FloorMap) -> Vec<OracleTrap> {
    map.traps
        .iter()
        .map(|trap| OracleTrap {
            cell: trap.cell,
            class_name: trap.class_name().to_string(),
            visible: trap.visible,
            active: trap.active,
            color: trap.color,
            shape: trap.shape,
        })
        .collect()
}

fn plants(map: &spd_core::report::FloorMap) -> Vec<OraclePlant> {
    map.plants
        .iter()
        .map(|plant| OraclePlant {
            cell: plant.cell,
            class_name: plant.class_name().to_string(),
            image: plant.image,
        })
        .collect()
}

fn blobs(map: &spd_core::report::FloorMap) -> Vec<OracleBlob> {
    map.blobs
        .iter()
        .map(|blob| OracleBlob {
            class_name: blob.class_name().to_string(),
            volume: blob.volume,
            always_visible: blob.always_visible,
            cells: blob
                .cells
                .iter()
                .map(|cell| OracleBlobCell {
                    cell: cell.cell,
                    value: cell.value,
                })
                .collect(),
        })
        .collect()
}
