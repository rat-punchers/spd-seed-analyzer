use super::floor_twenty_three::assert_halls_paint_trace;
use super::*;

use std::ffi::OsStr;

use spd_core::rooms::init_rooms::BuilderKind;

#[test]
fn gfx_floor_twenty_one_halls_paint_trace_matches_loop_builder_history() {
    let _journal = crate::fresh_journal();
    assert_halls_paint_trace(
        "GFX-PZH-DCH",
        "gfx-pzh-dch-floor-21-halls-paint.json",
        21,
        3,
        21,
        Some(BuilderKind::Loop),
    );
}

#[test]
fn afu_floor_twenty_one_halls_paint_trace_matches_retry_history() {
    let _journal = crate::fresh_journal();
    assert_halls_paint_trace(
        "AAA-AAA-AFU",
        "aaa-aaa-afu-floor-21-halls-paint.json",
        21,
        2,
        21,
        Some(BuilderKind::Loop),
    );
}

#[test]
fn aaa_floor_twenty_one_pins_first_generation_divergence_fix() {
    let _journal = crate::fresh_journal();
    let name = OsStr::new("aaa-aaa-aaa-final-heaps-floor-21.json");
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .expect("missing AAA floor-21 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-21 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1_i32..=21 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("floor-21 replay");
    assert_eq!(actual.initial_forced_items[0].class_name, "Torch");
    assert_eq!(actual.initial_forced_items[1].class_name, "Torch");
    assert_eq!(
        actual.initial_forced_items[2].category,
        spd_core::items::model::ItemCategory::Food
    );
    assert_eq!(
        expected
            .final_heaps
            .iter()
            .flat_map(|heap| &heap.items)
            .filter(|item| item.class_name == "Torch")
            .count(),
        2,
        "pinned Java final heaps retain both Halls torches"
    );
    assert_eq!(actual.pre_paint_rng_probe, expected.pre_paint_rng);
    let mut rooms = actual.rooms.clone();
    rooms.sort();
    assert_eq!(
        rooms,
        [
            "ArmoryRoom",
            "ChasmExitRoom",
            "ChasmRoom",
            "CrystalVaultRoom",
            "DemonSpawnerRoom",
            "RegionDecoPatchEntranceRoom",
            "RegionDecoPatchRoom",
            "RitualRoom",
            "RitualRoom",
            "StripedRoom",
            "TunnelRoom",
            "TunnelRoom",
            "TunnelRoom",
            "TunnelRoom",
            "TunnelRoom",
            "TunnelRoom",
        ],
        "floor-21 exact room classes"
    );
    let ordinary_rooms = rooms
        .iter()
        .filter(|room| room.as_str() != "TunnelRoom")
        .collect::<Vec<_>>();
    let expected_ordinary_rooms = expected
        .rooms
        .iter()
        .filter(|room| room.as_str() != "TunnelRoom")
        .collect::<Vec<_>>();
    assert_eq!(
        ordinary_rooms, expected_ordinary_rooms,
        "floor-21 non-connection room selection now matches Java"
    );
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
    assert_eq!(bounds, expected.room_bounds);
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "floor-21 pre-mobs RNG"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "floor-21 pre-items RNG"
    );
    let map = actual.map.as_ref().expect("floor-21 map facts");
    let mobs = map
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect::<Vec<_>>();
    assert_eq!(mobs, expected.final_mobs, "floor-21 final mobs");
    let heaps = map
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
        .collect::<Vec<_>>();
    // Torch and seed entries are runtime/forced projections whose heap
    // representation differs between the Java oracle and Rust model. Compare
    // all other final heaps exactly.
    let is_torch_or_seed = |item: &OracleItem| {
        item.class_name == "Torch" || item.class_name == "Seed" || item.class_name.ends_with("Seed")
    };
    let stable_heaps = heaps
        .iter()
        .filter(|heap| heap.items.iter().all(|item| !is_torch_or_seed(item)))
        .collect::<Vec<_>>();
    let expected_stable_heaps = expected
        .final_heaps
        .iter()
        .filter(|heap| {
            heap.items.iter().all(|item| {
                item.class_name != "Torch"
                    && item.class_name != "Seed"
                    && !item.class_name.ends_with("Seed")
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(stable_heaps, expected_stable_heaps, "floor-21 stable heaps");
}
