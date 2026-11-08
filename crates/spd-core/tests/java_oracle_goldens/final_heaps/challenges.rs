use super::*;

use std::ffi::OsStr;

use spd_core::Challenge;

/// Replays FVX-YTN-SLC with all nine challenges through `fixture`'s floor.
fn assert_all_challenge_floor(fixture_name: &str) {
    assert_challenge_floor(fixture_name, "all", &Challenge::ALL);
}

/// Replays `fixture`'s seed with `challenges` through its floor and compares
/// painted terrain, mobs, and seed-determined heaps with the oracle.
fn assert_challenge_floor(fixture_name: &str, oracle_flag: &str, challenges: &[Challenge]) {
    let name = OsStr::new(fixture_name);
    let path = fixture_paths()
        .into_iter()
        .find(|path| path.file_name().is_some_and(|file| file == name))
        .unwrap_or_else(|| panic!("missing {fixture_name}"));
    let fixture = read_fixture(&path);
    assert_eq!(fixture.input.challenge.as_deref(), Some(oracle_flag));
    let expected = fixture.floors.first().expect("oracle floor facts");
    let context = format!("{fixture_name} floor {}", expected.depth);

    let profile = spd_core::MapProfile {
        challenges: challenges.to_vec(),
        ..spd_core::MapProfile::default()
    };
    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    for depth in 1..=expected.depth {
        dungeon.depth = depth as i32;
        actual = Some(spd_core::level::create_level_partial_for_profile(
            &mut dungeon,
            &profile,
        ));
    }
    let actual = actual.expect("profiled replay");
    assert_eq!(
        actual.pre_paint_rng_probe, expected.pre_paint_rng,
        "{context}"
    );
    assert_eq!(
        actual.pre_mobs_rng_probe, expected.pre_mobs_rng,
        "{context}"
    );
    assert_eq!(
        actual.pre_items_rng_probe, expected.pre_items_rng,
        "{context}"
    );

    let map = actual.map.as_ref().expect("floor map");
    // The oracle has not read the searching guide page, so floor 2 hides its
    // entrance doors (`RegularPainter.java:268-275`); the public map shows them.
    let terrain: Vec<u16> = expected
        .terrain
        .as_ref()
        .expect("terrain")
        .iter()
        .zip(&map.tiles)
        .map(|(&java, &rust)| if (java, rust) == (16, 5) { 5 } else { java })
        .collect();
    assert_eq!(map.tiles, terrain, "{context} terrain");
    if challenges.contains(&Challenge::BarrenLand) {
        assert!(
            map.plants.is_empty(),
            "{context}: Barren Land couches no plants"
        );
    }

    let actual_mobs: Vec<_> = map
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(actual_mobs, expected.final_mobs, "{context} mobs");
    // Guide pages depend on the player's journal, not the seed. Equal-score
    // `ShopRoom.ChooseBag` ties follow JVM identity-hash iteration.
    let seed_determined = |heap: &OracleHeap| {
        heap.items.iter().all(|item| {
            !matches!(
                item.class_name.as_str(),
                "GuidePage" | "MagicalHolster" | "ScrollHolder" | "PotionBandolier" | "VelvetPouch"
            )
        })
    };
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
                    // The oracle names nested `Plant.Seed` classes "Seed".
                    class_name: normalize_oracle_class(item.class_name()).to_string(),
                    quantity: item.quantity,
                    level: item.level,
                    cursed: item.cursed,
                })
                .collect(),
        })
        .filter(seed_determined)
        .map(merge_gold)
        .collect();
    // A Suspicious Chest prize drawn from the spawn queue stays unpublished;
    // its chest sits on the room's center pedestal.
    const PEDESTAL: u16 = 11;
    let unpublished_suspicious_chest = |heap: &OracleHeap| {
        actual
            .rooms
            .iter()
            .any(|room| room == "SuspiciousChestRoom")
            && map.tiles[heap.cell as usize] == PEDESTAL
            && heap.heap_type == "chest"
            && !map.heaps.iter().any(|actual| actual.cell == heap.cell)
    };
    let expected_heaps: Vec<_> = expected
        .final_heaps
        .iter()
        .filter(|heap| seed_determined(heap) && !unpublished_suspicious_chest(heap))
        .cloned()
        .collect();
    assert_eq!(actual_heaps, expected_heaps, "{context} heaps");
    if challenges.contains(&Challenge::IntoDarkness) {
        assert!(
            actual_heaps
                .iter()
                .any(|heap| heap.items.iter().any(|item| item.class_name == "Torch")),
            "{context}: Into Darkness places a torch"
        );
    }
}

/// `Heap.drop` merges gold piles dropped on one cell; the map lists each pile.
fn merge_gold(mut heap: OracleHeap) -> OracleHeap {
    let mut items: Vec<OracleItem> = Vec::new();
    for item in heap.items {
        match items
            .iter_mut()
            .find(|kept| item.class_name == "Gold" && kept.class_name == "Gold")
        {
            Some(gold) => gold.quantity += item.quantity,
            None => items.push(item),
        }
    }
    heap.items = items;
    heap
}

/// Forbidden Runes drops floor 2's queued Scroll of Upgrade, shifting every
/// later queued drop (keys included); Into Darkness adds the floor torch.
#[test]
fn fvx_floor_two_all_challenges_matches_oracle() {
    let _journal = crate::fresh_journal();
    assert_all_challenge_floor("fvx-ytn-slc-final-heaps-floor-2-all-challenges.json");
}

/// The oracle's champion counter follows an unplayed run. These floors roll
/// no possible Giant outside open space, so the replay matches it anyway.
#[test]
fn fvx_later_floors_all_challenges_match_oracle() {
    let _journal = crate::fresh_journal();
    for depth in [4, 8, 12] {
        assert_all_challenge_floor(&format!(
            "fvx-ytn-slc-final-heaps-floor-{depth}-all-challenges.json"
        ));
    }
}

/// Floor 1's first mob is a champion (`Dungeon.init` sets the counter to 1).
/// On these seeds it is a LARGE Giant that has to move to open space.
#[test]
fn floor_one_giant_champion_matches_oracle() {
    let _journal = crate::fresh_journal();
    for seed in ["aaa-aaa-acv", "aaa-aaa-aes"] {
        assert_challenge_floor(
            &format!("{seed}-final-heaps-floor-1-champion-enemies.json"),
            "champion-enemies",
            &[Challenge::ChampionEnemies],
        );
    }
}

/// Barren Land on garden-heavy floors: Plants and Garden rooms (floor 2),
/// a Rot Garden (floor 8), and a Caves Secret Garden (floor 11).
#[test]
fn garden_floors_with_barren_land_match_oracle() {
    let _journal = crate::fresh_journal();
    for fixture in [
        "aaa-aaa-aan-final-heaps-floor-2-barren-land.json",
        "aaa-aaa-aag-final-heaps-floor-8-barren-land.json",
        "aaa-aaa-aac-final-heaps-floor-11-barren-land.json",
    ] {
        assert_challenge_floor(fixture, "barren-land", &[Challenge::BarrenLand]);
    }
}
