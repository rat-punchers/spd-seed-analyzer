use super::*;

#[test]
fn toxic_gas_room_lifecycle_matches_oracle() {
    let _journal = crate::fresh_journal();
    let fixture = read_fixture(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tools/java-oracle/fixtures/aaa-aaa-acb-final-heaps-floor-1.json"),
    );
    let expected = &fixture.floors[0];
    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    dungeon.depth = 1;
    let level = create_level_partial(&mut dungeon);
    let actual = level.map.as_ref().expect("depth-one internal map");
    assert_eq!(
        (actual.width, actual.height),
        (expected.width, expected.height)
    );
    let toxic_heaps: Vec<_> = actual
        .heaps
        .iter()
        .filter(|heap| matches!(heap.cell, 947 | 998 | 1096))
        .map(|heap| {
            (
                heap.cell,
                heap.heap_type.as_str(),
                heap.items[0].class_name(),
                heap.items[0].quantity,
            )
        })
        .collect();
    assert_eq!(
        toxic_heaps,
        [
            (947, "chest", "Gold", 61),
            (998, "chest", "Gold", 71),
            (1096, "skeleton", "Gold", 92),
        ]
    );
    let actual_heaps: Vec<_> = actual
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
    assert_eq!(actual_heaps, expected.final_heaps);
    let actual_mobs: Vec<_> = actual
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(actual_mobs, expected.final_mobs);
    assert!(!actual.markers.iter().any(|marker| {
        marker.kind == spd_core::report::MapMarkerKind::Item && marker.label == "Room loot"
    }));
}
