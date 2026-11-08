use super::floor_twenty_three::assert_halls_paint_trace;
use super::*;

use std::ffi::OsStr;
use std::fs;

use serde::Deserialize;
use spd_core::rooms::init_rooms::BuilderKind;

#[derive(Debug, Deserialize)]
struct PaintTrace {
    depth: i32,
    pre_shuffle_rooms: Vec<TraceRoom>,
    checkpoints: Vec<PaintCheckpoint>,
}

#[derive(Debug, Deserialize)]
struct TraceRoom {
    #[serde(rename = "class")]
    class_name: String,
    bounds: [i32; 4],
}

#[derive(Debug, Deserialize)]
struct PaintCheckpoint {
    stage: String,
    room: String,
    rng: Vec<i32>,
}

#[derive(Debug, Deserialize)]
struct DoorTrace {
    depth: i32,
    seed: String,
    post_doors_rng: Vec<i32>,
}

#[test]
fn gfx_floor_twenty_two_halls_paint_trace_matches_loop_builder_history() {
    let _journal = crate::fresh_journal();
    assert_halls_paint_trace(
        "GFX-PZH-DCH",
        "gfx-pzh-dch-floor-22-halls-paint.json",
        22,
        1,
        23,
        Some(BuilderKind::Loop),
    );
}

#[test]
fn afu_floor_twenty_two_halls_paint_trace_matches_retry_history() {
    let _journal = crate::fresh_journal();
    assert_halls_paint_trace(
        "AAA-AAA-AFU",
        "aaa-aaa-afu-floor-22-halls-paint.json",
        22,
        2,
        21,
        Some(BuilderKind::FigureEight),
    );
}

#[test]
fn aaa_floor_twenty_two_structural_lifecycle_matches_oracle() {
    let _journal = crate::fresh_journal();
    let name = OsStr::new("aaa-aaa-aaa-final-heaps-floor-22.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .expect("missing AAA floor-22 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-22 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1_i32..=22 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-22 replay");

    assert_eq!(actual.pre_paint_rng_probe, expected.pre_paint_rng);
    let mut rooms = actual.rooms.clone();
    rooms.sort();
    assert_eq!(rooms, expected.rooms, "floor-22 room selection");
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
    assert_eq!(bounds, expected.room_bounds, "floor-22 room bounds");

    let trace_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/java-oracle/fixtures/traces/aaa-aaa-aaa-floor-22-halls-paint.json");
    let trace: PaintTrace = serde_json::from_str(
        &fs::read_to_string(&trace_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", trace_path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", trace_path.display()));
    assert_eq!(trace.depth, 22);
    let pre_shuffle_rooms = trace
        .pre_shuffle_rooms
        .into_iter()
        .map(|room| (room.class_name, room.bounds))
        .collect::<Vec<_>>();
    assert_eq!(
        actual
            .pre_shuffle_room_bounds
            .iter()
            .map(|room| {
                (
                    room.class_name.clone(),
                    [room.left, room.top, room.right, room.bottom],
                )
            })
            .collect::<Vec<_>>(),
        pre_shuffle_rooms,
        "FigureEight returned room-list order before RegularPainter shuffle"
    );
    let expected_callbacks = trace
        .checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.stage == "room")
        .map(|checkpoint| (checkpoint.room.clone(), checkpoint.rng.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        actual
            .room_paint_rng_checkpoints
            .iter()
            .map(|checkpoint| (checkpoint.room.clone(), checkpoint.rng.clone()))
            .collect::<Vec<_>>(),
        expected_callbacks
    );
    let doors = trace.checkpoints.last().expect("paintDoors checkpoint");
    assert_eq!(doors.stage, "doors");
    assert_eq!(doors.room, "paintDoors");
    assert_eq!(doors.rng.len(), 8);
    assert_eq!(
        actual.post_doors_rng_probe, doors.rng,
        "paintDoors RNG boundary"
    );

    let map = actual.map.as_ref().expect("floor-22 internal map");
    let expected_terrain = expected.terrain.as_ref().expect("oracle terrain");
    let expected_discoverable = expected
        .discoverable
        .as_ref()
        .expect("oracle discoverability");
    // The two shared RuinsRoom edges that previously rolled doors must be
    // painter-open EMPTY terrain and retain their pinned discoverability.
    for (x, y) in [(7, 40), (25, 34)] {
        let cell = (y * map.width + x) as usize;
        assert_eq!(
            map.tiles[cell], expected_terrain[cell],
            "door terrain at {x},{y}"
        );
        assert_eq!(
            map.discoverable[cell], expected_discoverable[cell],
            "door discoverability at {x},{y}"
        );
    }
    let transitions: Vec<_> = map
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
        .collect();
    assert_eq!(
        transitions,
        *expected.transitions.as_ref().expect("oracle transitions")
    );
}

#[test]
fn abc_floor_twenty_two_paint_doors_matches_oracle() {
    let _journal = crate::fresh_journal();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/java-oracle/fixtures/traces/abc-def-ghi-floor-22-halls-doors.json");
    let trace: DoorTrace = serde_json::from_str(
        &fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
    )
    .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
    assert_eq!(trace.depth, 22);

    let seed = parse_seed(&trace.seed).expect("valid oracle seed");
    let mut dungeon = dungeon_from_run(init_run(seed.numeric));
    let mut actual = None;
    for depth in 1_i32..=trace.depth {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }

    assert_eq!(
        actual.expect("floor-22 replay").post_doors_rng_probe,
        trace.post_doors_rng,
        "contrasting depth-22 paintDoors RNG boundary"
    );
}
