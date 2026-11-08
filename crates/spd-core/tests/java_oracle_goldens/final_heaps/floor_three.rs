use super::*;

use std::ffi::OsStr;

#[test]
fn gfx_floor_three_lifecycle_matches_v4_oracle() {
    let _journal = crate::fresh_journal();
    let name = OsStr::new("gfx-pzh-dch-final-heaps-floor-3.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .expect("missing GFX floor-3 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-3 oracle facts");
    assert_eq!(fixture.spd.version, SPD_VERSION);
    assert_eq!(fixture.spd.commit, SPD_COMMIT);

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1..=3 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-3 replay");
    assert_eq!(actual.pre_mobs_rng_probe, expected.pre_mobs_rng);
    assert_eq!(actual.pre_items_rng_probe, expected.pre_items_rng);

    let map = actual.map.as_ref().expect("floor-3 map");
    let actual_mobs = map
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_mobs, expected.final_mobs, "pinned v4 floor-3 mobs");

    let actual_heaps = map
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
        .collect::<Vec<_>>();
    assert_eq!(
        actual_heaps, expected.final_heaps,
        "pinned v4 floor-3 heaps"
    );
}
