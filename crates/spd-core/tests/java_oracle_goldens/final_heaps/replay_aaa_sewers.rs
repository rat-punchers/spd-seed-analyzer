use super::*;

use std::ffi::OsStr;

#[test]
fn aaa_replay_pins_floors_two_through_four() {
    let _journal = crate::fresh_journal();
    let fixtures: Vec<_> = (2..=4)
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
    for depth in 1_u32..=4 {
        dungeon.depth = depth as i32;
        let actual = create_level_partial(&mut dungeon);
        if depth == 1 {
            continue;
        }
        let expected = fixture_and_floor(&fixtures[(depth - 2) as usize], depth);
        let context = format!("AAA floor-{depth}");
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
            actual.pre_paint_rng_probe, expected.pre_paint_rng,
            "{context} pre-paint RNG"
        );
        assert_eq!(
            actual
                .room_paint_rng_checkpoints
                .last()
                .map(|checkpoint| checkpoint.rng.as_slice()),
            Some(expected.pre_doors_rng.as_slice()),
            "{context} pre-doors RNG"
        );
        assert_eq!(
            actual.post_doors_rng_probe, expected.post_doors_rng,
            "{context} post-doors RNG"
        );
        if depth == 2 {
            let map = actual.map.as_ref().expect("regular floor map");
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
                            class_name: if item.class_name().ends_with("Seed") {
                                "Seed".into()
                            } else {
                                item.class_name().to_string()
                            },
                            quantity: item.quantity,
                            level: item.level,
                            cursed: item.cursed,
                        })
                        .collect(),
                })
                .collect();
            assert_eq!(
                seeded_heaps(&actual_heaps),
                seeded_heaps(&expected.final_heaps),
                "{context} exact seeded heaps"
            );
            assert_eq!(
                actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
                "{context} pre-mobs RNG"
            );
            assert_eq!(
                actual.pre_items_rng_probe, expected.pre_items_rng,
                "{context} pre-items RNG"
            );
        } else if depth == 3 {
            let map = actual.map.as_ref().expect("regular floor map");
            let actual_exit = map
                .transitions
                .iter()
                .find(|transition| transition.transition_type == "REGULAR_EXIT")
                .expect("AAA floor-3 regular exit");
            let java_exit = expected
                .transitions
                .as_ref()
                .expect("AAA floor-3 Java transitions")
                .iter()
                .find(|transition| transition.transition_type == "REGULAR_EXIT")
                .expect("AAA floor-3 Java regular exit");
            assert_eq!(
                actual_exit.cell, java_exit.cell,
                "{context} exact regular exit"
            );
            assert_eq!(
                actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
                "{context} pre-mobs RNG"
            );
            assert_eq!(
                actual.pre_items_rng_probe, expected.pre_items_rng,
                "{context} pre-items RNG"
            );
        } else if depth == 4 {
            assert_eq!(
                actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
                "{context} pre-mobs RNG"
            );
            assert_eq!(
                actual.pre_items_rng_probe, expected.pre_items_rng,
                "{context} pre-items RNG"
            );
            let map = actual.map.as_ref().expect("regular floor map");
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
                            class_name: if item.class_name().ends_with("Seed") {
                                "Seed".into()
                            } else {
                                item.class_name().to_string()
                            },
                            quantity: item.quantity,
                            level: item.level,
                            cursed: item.cursed,
                        })
                        .collect(),
                })
                .collect();
            assert_eq!(
                seeded_heaps(&actual_heaps),
                seeded_heaps(&expected.final_heaps),
                "{context} exact seeded heaps"
            );
            assert_eq!(
                map.tiles,
                *expected.terrain.as_ref().expect("AAA floor-4 terrain"),
                "{context} exact terrain"
            );
            let actual_plants: Vec<_> = map
                .plants
                .iter()
                .map(|plant| OraclePlant {
                    cell: plant.cell,
                    class_name: plant.class_name().to_string(),
                    image: plant.image,
                })
                .collect();
            assert_eq!(
                actual_plants,
                *expected.plants.as_ref().expect("AAA floor-4 plants"),
                "{context} exact plants"
            );
            let actual_blobs: Vec<_> = map
                .blobs
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
                .collect();
            assert_eq!(
                actual_blobs,
                *expected.blobs.as_ref().expect("AAA floor-4 blobs"),
                "{context} exact blobs"
            );
        }
    }
}

fn seeded_heaps(heaps: &[OracleHeap]) -> Vec<&OracleHeap> {
    heaps
        .iter()
        .filter(|heap| heap.items.iter().all(|item| item.class_name != "GuidePage"))
        .collect()
}

fn fixture_and_floor(fixture: &OracleFixture, depth: u32) -> &OracleFloor {
    assert_eq!(fixture.schema_version, EXTENDED_FINAL_HEAPS_SCHEMA_VERSION);
    assert_eq!(fixture.contract.as_deref(), Some("final_placed_heaps"));
    assert_eq!(fixture.input.depths, [depth]);
    fixture.floors.first().expect("replay oracle floor")
}
