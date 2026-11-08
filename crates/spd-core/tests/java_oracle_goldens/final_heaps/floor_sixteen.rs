use super::*;

use std::ffi::OsStr;

#[test]
fn aaa_floor_sixteen_matches_room_classes_and_normalized_bounds() {
    let _journal = crate::fresh_journal();
    assert_floor_matches_oracle("aaa-aaa-aaa-final-heaps-floor-16.json");
}

/// FVX-YTN-SLC's floor-13 exit room connects to the smithy, which Java allows
/// (`BlacksmithRoom.canConnect` only runs on the smithy's own `connect`). A
/// rejected pair there rebuilt floor 13 and shifted every later floor's
/// Generator decks. Floors 12 and 14 also cover a Suspicious Chest prize chest.
#[test]
fn fvx_caves_and_city_floors_match_java_final_state() {
    let _journal = crate::fresh_journal();
    for depth in [12, 13, 14, 16] {
        assert_floor_matches_oracle(&format!("fvx-ytn-slc-final-heaps-floor-{depth}.json"));
    }
}

fn assert_floor_matches_oracle(fixture_name: &str) {
    let name = OsStr::new(fixture_name);
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .unwrap_or_else(|| panic!("missing {fixture_name}"));
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("oracle floor facts");
    let depth = expected.depth as i32;

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for replay_depth in 1_i32..=depth {
        dungeon.depth = replay_depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor replay");
    let mut rooms = actual.rooms.clone();
    rooms.sort();
    assert_eq!(rooms, expected.rooms, "{fixture_name} room classes");

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
    assert_eq!(
        bounds, expected.room_bounds,
        "{fixture_name} normalized bounds"
    );
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "{fixture_name} pre-paint RNG boundary"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "{fixture_name} pre-mobs RNG boundary"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "{fixture_name} pre-items RNG boundary"
    );
    let map = actual.map.as_ref().expect("regular floor map");
    let mobs: Vec<_> = map
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(mobs, expected.final_mobs, "{fixture_name} final mobs");
    let heaps: Vec<_> = map
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
    assert_eq!(heaps, expected.final_heaps, "{fixture_name} final heaps");
}

#[derive(Debug, serde::Deserialize)]
struct CityShopTrace {
    depth: i32,
    builder_kind: String,
    build_attempts: Vec<CityShopAttempt>,
    pre_shuffle_rooms: Vec<CityShopRoom>,
    checkpoints: Vec<CityShopCheckpoint>,
}

#[derive(Debug, serde::Deserialize)]
struct CityShopAttempt {
    start_rng: Vec<i32>,
    success: bool,
}

#[derive(Debug, serde::Deserialize)]
struct CityShopRoom {
    #[serde(rename = "class")]
    class_name: String,
    bounds: [i32; 4],
}

#[derive(Debug, serde::Deserialize)]
struct CityShopCheckpoint {
    stage: String,
    room: String,
    rng: Vec<i32>,
}

/// `LoopBuilder` collides its shop against the loop only. This AFU floor is the
/// pinned case where the wider room list burns an extra `findFreeSpace`
/// tie-break, which would shift the whole shop stock and every later floor.
#[test]
fn afu_floor_sixteen_loop_builder_shop_matches_oracle() {
    let _journal = crate::fresh_journal();
    let actual =
        assert_loop_builder_shop_trace("AAA-AAA-AFU", "aaa-aaa-afu-floor-16-city-paint.json");

    let stock = actual
        .placed_items
        .iter()
        .filter(|item| item.source.as_deref() == Some("ShopRoom"))
        .map(|item| item.class_name.as_str())
        .collect::<Vec<_>>();
    assert!(
        stock.contains(&"Stylus"),
        "Java stocks the floor-16 rare slot with a Stylus, not an artifact: {stock:?}"
    );
    assert!(
        stock.contains(&"HealingDart"),
        "tipped dart stock: {stock:?}"
    );
}

/// A second seed guards the narrower LoopBuilder shop collision list against
/// accidentally fitting only the original AFU placement.
#[test]
fn gfx_floor_sixteen_loop_builder_shop_matches_oracle() {
    let _journal = crate::fresh_journal();
    assert_loop_builder_shop_trace("GFX-PZH-DCH", "gfx-pzh-dch-floor-16-city-paint.json");
}

fn assert_loop_builder_shop_trace(
    seed_text: &str,
    fixture_name: &str,
) -> spd_core::level::LevelState {
    let trace_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/java-oracle/fixtures/traces")
        .join(fixture_name);
    let trace: CityShopTrace = serde_json::from_str(
        &std::fs::read_to_string(&trace_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", trace_path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", trace_path.display()));
    assert_eq!(trace.depth, 16);
    assert_eq!(trace.builder_kind, "LoopBuilder");
    assert_eq!(trace.build_attempts.len(), 1);
    assert!(trace.build_attempts[0].success);
    assert_eq!(
        trace.build_attempts[0].start_rng.len(),
        8,
        "fixed-width Java RNG probe"
    );

    let seed = parse_seed(seed_text).expect("valid oracle seed");
    let mut dungeon = dungeon_from_run(init_run(seed.numeric));
    let mut actual = None;
    for depth in 1_i32..=16 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-16 replay");

    let expected_rooms = trace
        .pre_shuffle_rooms
        .iter()
        .map(|room| (room.class_name.clone(), room.bounds))
        .collect::<Vec<_>>();
    let actual_rooms = actual
        .pre_shuffle_room_bounds
        .iter()
        .map(|room| {
            (
                room.class_name.clone(),
                [room.left, room.top, room.right, room.bottom],
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_rooms, expected_rooms, "pre-shuffle room bounds");

    let expected_callbacks = trace
        .checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.stage == "room")
        .map(|checkpoint| (checkpoint.room.clone(), checkpoint.rng.clone()))
        .collect::<Vec<_>>();
    let actual_callbacks = actual
        .room_paint_rng_checkpoints
        .iter()
        .map(|checkpoint| (checkpoint.room.clone(), checkpoint.rng.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        actual_callbacks, expected_callbacks,
        "room paint RNG callbacks"
    );

    let doors = trace.checkpoints.last().expect("paintDoors checkpoint");
    assert_eq!((&doors.stage[..], &doors.room[..]), ("doors", "paintDoors"));
    assert_eq!(
        actual.post_doors_rng_probe, doors.rng,
        "post-door RNG boundary"
    );

    actual
}
