use super::*;

use std::ffi::OsStr;

#[test]
fn aaa_floor_eighteen_matches_through_blobs() {
    let _journal = crate::fresh_journal();
    let name = OsStr::new("aaa-aaa-aaa-final-heaps-floor-18.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .expect("missing AAA floor-18 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-18 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1_i32..=18 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-18 replay");
    let mut rooms = actual.rooms.clone();
    rooms.sort();
    assert_eq!(rooms, expected.rooms, "floor-18 room classes");

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
    assert_eq!(bounds, expected.room_bounds, "floor-18 normalized bounds");
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "floor-18 pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "floor-18 pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "floor-18 pre-items RNG boundary"
    );
    let mobs: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-18 map facts")
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(mobs, expected.final_mobs, "floor-18 final mobs");
    let heaps: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-18 map facts")
        .heaps
        .iter()
        .map(|heap| OracleHeap {
            cell: heap.cell,
            heap_type: heap.heap_type.clone(),
            items: heap
                .items
                .iter()
                .map(|item| OracleItem {
                    // Java reports nested plant seed classes by their simple name.
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
    assert_eq!(heaps, expected.final_heaps, "floor-18 final heaps");
    let transitions = actual
        .map
        .as_ref()
        .expect("floor-18 map facts")
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
        &transitions,
        expected.transitions.as_ref().expect("floor-18 transitions"),
        "floor-18 transitions"
    );
    let traps = actual
        .map
        .as_ref()
        .expect("floor-18 map facts")
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
        .collect::<Vec<_>>();
    assert_eq!(
        &traps,
        expected.traps.as_ref().expect("floor-18 traps"),
        "floor-18 traps"
    );
    let plants = actual
        .map
        .as_ref()
        .expect("floor-18 map facts")
        .plants
        .iter()
        .map(|plant| OraclePlant {
            cell: plant.cell,
            class_name: plant.class_name().to_string(),
            image: plant.image,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        &plants,
        expected.plants.as_ref().expect("floor-18 plants"),
        "floor-18 plants"
    );
    let blobs = actual
        .map
        .as_ref()
        .expect("floor-18 map facts")
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
        .collect::<Vec<_>>();
    assert_eq!(
        &blobs,
        expected.blobs.as_ref().expect("floor-18 blobs"),
        "floor-18 blobs"
    );
    let expected_terrain = expected.terrain.as_ref().expect("floor-18 terrain");
    let actual_terrain = &actual.map.as_ref().expect("floor-18 map facts").tiles;
    assert_eq!(actual_terrain, expected_terrain, "floor-18 terrain");
    let actual_map = actual.map.as_ref().expect("floor-18 map facts");
    assert_eq!(
        &actual_map.discoverable,
        expected
            .discoverable
            .as_ref()
            .expect("floor-18 discoverable mask"),
        "floor-18 discoverable mask"
    );
    assert_eq!(
        &actual_map.tile_variance,
        expected
            .tile_variance
            .as_ref()
            .expect("floor-18 tile variance"),
        "floor-18 tile variance"
    );
}
