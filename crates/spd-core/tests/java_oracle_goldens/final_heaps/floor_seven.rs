use super::*;

#[path = "floor_seven/parity.rs"]
mod parity;

#[test]
fn hkt_floor_seven_lifecycle_matches_oracle() {
    let _journal = crate::fresh_journal();
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "hkt-jzn-xqq-final-heaps-floor-7.json")
        })
        .expect("HKT floor-7 schema-v3 fixture");
    let fixture = read_fixture(&path);
    assert_eq!(fixture.floors.len(), 1);
    let expected = fixture.floors.first().expect("floor-7 oracle facts");

    assert_eq!(fixture.schema_version, EXTENDED_FINAL_HEAPS_SCHEMA_VERSION);
    assert_eq!(fixture.contract.as_deref(), Some("final_placed_heaps"));
    assert_eq!(fixture.input.depths, [7]);
    assert_eq!(expected.depth, 7);
    assert_eq!((expected.width, expected.height), (36, 48));
    assert_eq!(expected.pre_paint_rng.len(), 8);
    assert_eq!(expected.pre_mobs_rng.len(), 8);
    assert_eq!(expected.pre_items_rng.len(), 8);
    assert_eq!(expected.terrain.as_ref().map(Vec::len), Some(36 * 48));
    assert_eq!(expected.discoverable.as_ref().map(Vec::len), Some(36 * 48));
    assert_eq!(expected.tile_variance.as_ref().map(Vec::len), Some(36 * 48));
    assert!(expected.forced_items.is_empty());
    assert_eq!(expected.final_heaps.len(), 14);
    assert_eq!(expected.final_mobs.len(), 7);
    assert_eq!(expected.transitions.as_ref().map(Vec::len), Some(2));
    assert_eq!(expected.traps.as_ref().map(Vec::len), Some(2));
    assert_eq!(expected.plants.as_ref().map(Vec::len), Some(0));
    assert_eq!(expected.blobs.as_ref().map(Vec::len), Some(0));
    assert_eq!(
        expected.quest_rewards,
        [
            OracleItem {
                class_name: "WandOfLivingEarth".into(),
                quantity: 1,
                level: 1,
                cursed: false,
            },
            OracleItem {
                class_name: "WandOfPrismaticLight".into(),
                quantity: 1,
                level: 3,
                cursed: false,
            },
        ],
        "pinned Java Wandmaker rewards"
    );

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1..=7 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-7 Rust facts");
    let map = actual.map.as_ref().expect("floor-7 regular map");
    let mut actual_rooms = actual.rooms.clone();
    actual_rooms.sort();
    assert_eq!(actual.feeling.as_str(), "none", "HKT floor-7 feeling");
    assert_eq!(actual_rooms, expected.rooms, "HKT floor-7 room classes");
    assert_eq!(
        (map.width, map.height),
        (expected.width, expected.height),
        "HKT floor-7 map bounds"
    );
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "HKT floor-7 pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "HKT floor-7 pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "HKT floor-7 pre-items RNG boundary"
    );
    parity::assert_map(map, expected);
    let actual_rewards: Vec<_> = actual
        .placed_items
        .iter()
        .filter(|item| item.source.as_deref() == Some("Wandmaker.Quest"))
        .map(|item| OracleItem {
            class_name: item.class_name.clone(),
            quantity: item.quantity,
            level: item.level,
            cursed: item.cursed,
        })
        .collect();

    assert_eq!(
        actual_rewards, expected.quest_rewards,
        "HKT floor-7 Wandmaker rewards"
    );
    assert!(actual
        .quests
        .iter()
        .any(|quest| matches!(quest, spd_core::QuestReport::OldWandmaker { .. })));
}
