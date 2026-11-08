use super::*;

use std::ffi::OsStr;

#[test]
fn last_level_matches_raw_java_layout_for_preserved_runs() {
    let _journal = crate::fresh_journal();
    for name in [
        "aaa-aaa-aaa-final-heaps-floor-26.json",
        "abc-def-ghi-final-heaps-floor-26.json",
    ] {
        let path = fixture_paths()
            .into_iter()
            .find(|path| {
                path.file_name()
                    .is_some_and(|file| file == OsStr::new(name))
            })
            .unwrap_or_else(|| panic!("missing depth-26 fixture {name}"));
        let fixture = read_fixture(&path);
        let expected = fixture.floors.first().expect("depth-26 oracle facts");

        let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
        let mut actual = None;
        for depth in 1_i32..=26 {
            dungeon.depth = depth;
            actual = Some(create_level_partial(&mut dungeon));
        }
        let actual = actual.expect("depth-26 replay");
        let map = actual.layout_map.as_ref().expect("LastLevel map");

        assert!(
            actual.pre_items_rng_probe.is_empty(),
            "{name} pre-items RNG"
        );
        assert_eq!(actual.pre_items_rng_probe, expected.pre_items_rng, "{name}");
        let expected_terrain = expected.terrain.as_ref().expect("terrain");
        let mismatches = map
            .tiles
            .iter()
            .zip(expected_terrain)
            .enumerate()
            .filter_map(|(cell, (&actual, &expected))| {
                (actual != expected).then_some((cell, actual, expected))
            })
            .collect::<Vec<_>>();
        assert!(mismatches.is_empty(), "{name} terrain: {mismatches:?}");
        assert_eq!(
            map.tile_variance,
            *expected.tile_variance.as_ref().expect("tile variance"),
            "{name}"
        );
        assert_eq!(
            map.discoverable,
            *expected.discoverable.as_ref().expect("discoverability"),
            "{name}"
        );
        let transitions = map
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
            .collect::<Vec<_>>();
        assert_eq!(
            transitions,
            *expected.transitions.as_ref().expect("transitions"),
            "{name}"
        );
        assert_eq!(
            map.custom_tiles.len(),
            expected.custom_tiles.len(),
            "{name}"
        );
        assert_eq!(
            map.custom_walls.len(),
            expected.custom_walls.len(),
            "{name}"
        );
        assert!(map.traps.is_empty(), "{name} traps");
        assert!(map.plants.is_empty(), "{name} plants");
        assert!(map.blobs.is_empty(), "{name} blobs");
        assert_eq!(expected.traps.as_ref(), Some(&Vec::new()), "{name}");
        assert_eq!(expected.plants.as_ref(), Some(&Vec::new()), "{name}");
        assert_eq!(expected.blobs.as_ref(), Some(&Vec::new()), "{name}");
        for (actual, expected) in map.custom_tiles.iter().zip(&expected.custom_tiles) {
            assert_eq!(actual.class_name, expected.class_name, "{name}");
            assert_eq!(
                actual.texture,
                expected.texture.as_deref().unwrap_or("halls_special"),
                "{name}"
            );
            assert_eq!(
                (actual.x, actual.y, actual.width, actual.height),
                (expected.x, expected.y, expected.width, expected.height),
                "{name}"
            );
            // The oracle dumps `CustomFloor.CANDLES`; `create()` paints them on
            // rows 9..16, bumps bright candles by tile variance, and marks
            // floor-over-chasm edges with 6.
            let actual_data: Vec<i16> = if actual.class_name == "CustomFloor" {
                actual.static_data[9 * 7..16 * 7]
                    .iter()
                    .zip(&expected.static_data)
                    .map(|(&visual, &candle)| match visual {
                        47 => 46,
                        6 => candle,
                        visual => visual,
                    })
                    .collect()
            } else {
                actual.static_data.clone()
            };
            let mismatches = actual_data
                .iter()
                .zip(&expected.static_data)
                .enumerate()
                .filter_map(|(index, (&actual, &expected))| {
                    (actual != expected).then_some((index, actual, expected))
                })
                .collect::<Vec<_>>();
            assert_eq!(actual_data.len(), expected.static_data.len(), "{name}");
            assert!(mismatches.is_empty(), "{name} custom tile: {mismatches:?}");
        }
        for (actual, expected) in map.custom_walls.iter().zip(&expected.custom_walls) {
            assert_eq!(actual.class_name, expected.class_name, "{name}");
            assert_eq!(
                actual.texture,
                expected.texture.as_deref().unwrap_or("halls_special"),
                "{name}"
            );
            assert_eq!(
                (actual.x, actual.y, actual.width, actual.height),
                (expected.x, expected.y, expected.width, expected.height),
                "{name}"
            );
            assert_eq!(actual.static_data, expected.static_data, "{name}");
        }
    }
}
