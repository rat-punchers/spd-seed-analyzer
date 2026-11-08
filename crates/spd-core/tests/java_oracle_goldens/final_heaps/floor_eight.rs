use super::*;

#[path = "floor_eight/parity.rs"]
mod parity;

#[test]
fn hkt_floor_eight_lifecycle_matches_oracle() {
    let _journal = crate::fresh_journal();
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "hkt-jzn-xqq-final-heaps-floor-8.json")
        })
        .expect("HKT floor-8 schema-v3 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-8 oracle facts");
    assert_eq!(fixture.schema_version, EXTENDED_FINAL_HEAPS_SCHEMA_VERSION);
    assert_eq!(fixture.contract.as_deref(), Some("final_placed_heaps"));
    assert_eq!(fixture.input.depths, [8]);
    assert_eq!(fixture.floors.len(), 1);
    assert_eq!(expected.depth, 8);
    assert_eq!((expected.width, expected.height), (33, 47));
    assert_eq!(
        expected.rooms,
        [
            "LaboratoryRoom",
            "MazeConnectionRoom",
            "PerimeterRoom",
            "PerimeterRoom",
            "PerimeterRoom",
            "PerimeterRoom",
            "PerimeterRoom",
            "PerimeterRoom",
            "PerimeterRoom",
            "PillarsRoom",
            "RegionDecoLineEntranceRoom",
            "RegionDecoLineExitRoom",
            "RegionDecoLineRoom",
            "SecretSummoningRoom",
            "SegmentedRoom",
            "SegmentedRoom",
            "SegmentedRoom",
        ]
    );
    assert_eq!(
        expected.pre_paint_rng,
        [
            -801_880_017,
            1_279_339_595,
            -1_981_559_586,
            1_886_819_143,
            -1_351_769_589,
            -171_529_757,
            907_911_222,
            807_719_556,
        ]
    );
    assert_eq!(
        expected.pre_mobs_rng,
        [
            -1_556_813_937,
            476_307_848,
            -108_868_643,
            -63_914_992,
            -992_236_198,
            -93_947_353,
            432_800_423,
            -1_972_956_674,
        ]
    );
    assert_eq!(
        expected.pre_items_rng,
        [
            -701_498_439,
            1_236_075_099,
            1_145_481_979,
            -700_827_306,
            -903_100_383,
            -100_902_382,
            -2_078_450_022,
            321_399_050,
        ]
    );
    assert_eq!(expected.terrain.as_ref().map(Vec::len), Some(33 * 47));
    assert_eq!(expected.discoverable.as_ref().map(Vec::len), Some(33 * 47));
    assert_eq!(expected.tile_variance.as_ref().map(Vec::len), Some(33 * 47));
    assert!(expected.forced_items.is_empty());
    assert_eq!(expected.final_heaps.len(), 14);
    assert!(expected
        .final_heaps
        .windows(2)
        .all(|pair| pair[0].cell < pair[1].cell));
    assert!(expected
        .final_heaps
        .iter()
        .all(|heap| heap.items.len() == 1));
    let heap_facts: Vec<_> = expected
        .final_heaps
        .iter()
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
    assert_eq!(
        heap_facts,
        [
            (367, "heap", "ScrollOfTerror", 1, 0, false),
            (382, "heap", "GuidePage", 1, 0, false),
            (431, "heap", "Gold", 210, 0, false),
            (877, "heap", "StoneOfFlock", 1, 0, false),
            (901, "heap", "Food", 1, 0, false),
            (908, "heap", "PotionOfStrength", 1, 0, false),
            (910, "heap", "AlchemyPage", 1, 0, false),
            (937, "heap", "IronKey", 1, 0, false),
            (938, "heap", "Gold", 219, 0, false),
            (975, "heap", "AlchemyPage", 1, 0, false),
            (1007, "heap", "EnergyCrystal", 5, 0, false),
            (1038, "locked_chest", "WandOfCorrosion", 1, 2, false),
            (1070, "heap", "GoldenKey", 1, 0, false),
            (1443, "skeleton", "Gold", 172, 0, false),
        ]
    );
    assert_eq!(expected.final_mobs.len(), 6);
    assert!(expected
        .final_mobs
        .windows(2)
        .all(|pair| pair[0].cell < pair[1].cell));
    let mob_facts: Vec<_> = expected
        .final_mobs
        .iter()
        .map(|mob| (mob.cell, mob.class_name.as_str()))
        .collect();
    assert_eq!(
        mob_facts,
        [
            (181, "DM100"),
            (336, "DM100"),
            (745, "Thief"),
            (868, "Skeleton"),
            (1080, "Guard"),
            (1178, "Necromancer"),
        ]
    );
    assert_eq!(expected.transitions.as_ref().map(Vec::len), Some(2));
    assert_eq!(expected.traps.as_ref().map(Vec::len), Some(17));
    assert_eq!(expected.plants.as_ref().map(Vec::len), Some(0));
    assert_eq!(expected.blobs.as_ref().map(Vec::len), Some(1));
    for cells in [
        expected
            .transitions
            .as_ref()
            .expect("floor-8 transitions")
            .iter()
            .map(|transition| transition.cell)
            .collect::<Vec<_>>(),
        expected
            .traps
            .as_ref()
            .expect("floor-8 traps")
            .iter()
            .map(|trap| trap.cell)
            .collect(),
    ] {
        assert!(cells.windows(2).all(|pair| pair[0] < pair[1]));
    }

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1..=8 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-8 Rust facts");
    let map = actual.map.as_ref().expect("floor-8 regular map");
    let mut actual_rooms = actual.rooms.clone();
    actual_rooms.sort();

    assert_eq!(actual.feeling.as_str(), "none", "HKT floor-8 feeling");
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "HKT floor-8 pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "HKT floor-8 pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "HKT floor-8 pre-items RNG boundary"
    );
    assert_eq!(
        (map.width, map.height),
        (expected.width, expected.height),
        "HKT floor-8 map bounds"
    );
    assert_eq!(actual_rooms, expected.rooms, "HKT floor-8 room classes");
    parity::assert_map(map, expected);
}
