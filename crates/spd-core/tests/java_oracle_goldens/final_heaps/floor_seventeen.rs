use super::*;

use std::ffi::OsStr;

#[test]
fn aaa_floor_seventeen_matches_the_pinned_final_facts() {
    let _journal = crate::fresh_journal();
    let name = OsStr::new("aaa-aaa-aaa-final-heaps-floor-17.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .expect("missing AAA floor-17 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-17 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1_i32..=17 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-17 replay");
    let mut rooms = actual.rooms.clone();
    rooms.sort();
    assert_eq!(rooms, expected.rooms, "floor-17 room classes");

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
    assert_eq!(bounds, expected.room_bounds, "floor-17 normalized bounds");
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "floor-17 pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "floor-17 pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "floor-17 pre-items RNG boundary"
    );
    let mobs: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-17 map")
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(mobs, expected.final_mobs, "floor-17 final mobs");
    let heaps: Vec<_> = actual
        .map
        .as_ref()
        .expect("floor-17 map")
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
    assert_eq!(heaps, expected.final_heaps, "floor-17 normalized heaps");
}
