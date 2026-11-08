use super::*;
use crate::report::{FloorMap, ItemDependencyCondition};
use crate::FloorReport;
use spd_objects::ItemId;

const NINE_CHALLENGE_SEED: &str = "FVX-YTN-SLC";

fn displayed_map(report: &SeedReport, depth: u32) -> &FloorMap {
    let floor: &FloorReport = report
        .floors
        .iter()
        .find(|floor| floor.depth == depth)
        .expect("floor in report");
    floor
        .map
        .as_ref()
        .or(floor.assumed_map.as_ref())
        .expect("displayed map")
}

fn heap_cells(map: &FloorMap, item: ItemId) -> Vec<(u32, u32)> {
    map.heaps
        .iter()
        .filter(|heap| heap.items.iter().any(|entry| entry.item == item))
        .map(|heap| (heap.cell % map.width, heap.cell / map.width))
        .collect()
}

fn has_challenge_condition(report: &SeedReport) -> bool {
    report
        .floors
        .iter()
        .flat_map(|floor| &floor.items)
        .flat_map(|group| &group.variants)
        .flat_map(|item| &item.spawn_conditions)
        .flat_map(|condition| &condition.all_of)
        .any(|dependency| matches!(dependency, ItemDependencyCondition::Challenge { .. }))
}

#[test]
fn no_challenges_is_the_trinket_report() {
    let declared = analyze_seed_for_run("7", 3, None, &[], true).unwrap();
    let plain = analyze_seed_with_trinket("7", 3, None).unwrap();
    assert_eq!(
        serde_json::to_value(declared).unwrap(),
        serde_json::to_value(plain).unwrap()
    );
}

#[test]
fn forbidden_runes_moves_floor_two_keys_without_challenge_conditions() {
    let plain = analyze_seed_with_trinket(NINE_CHALLENGE_SEED, 2, None).unwrap();
    let runes = analyze_seed_for_run(
        NINE_CHALLENGE_SEED,
        2,
        None,
        &[Challenge::ForbiddenRunes],
        true,
    )
    .unwrap();

    assert_eq!(
        heap_cells(displayed_map(&plain, 2), ItemId::IronKey),
        [(5, 20)]
    );
    assert_eq!(
        heap_cells(displayed_map(&runes, 2), ItemId::IronKey),
        [(6, 25)]
    );
    assert!(has_challenge_condition(&plain));
    assert!(!has_challenge_condition(&runes));
}

#[test]
fn into_darkness_places_a_torch_on_regular_floors() {
    let report = analyze_seed_for_run(
        NINE_CHALLENGE_SEED,
        2,
        None,
        &[Challenge::IntoDarkness],
        true,
    )
    .unwrap();
    assert_eq!(
        heap_cells(displayed_map(&report, 2), ItemId::Torch),
        [(15, 19)]
    );
}

#[test]
fn every_generation_challenge_is_modeled() {
    let modeled = [
        Challenge::ChampionEnemies,
        Challenge::ForbiddenRunes,
        Challenge::BarrenLand,
        Challenge::IntoDarkness,
        Challenge::BadderBosses,
    ];
    for challenge in Challenge::ALL {
        let profile = MapProfile {
            challenges: vec![challenge],
            ..MapProfile::default()
        };
        assert!(!profile.has_unmodeled_generation_inputs(), "{challenge:?}");
        assert_eq!(
            challenge.generation_effect() == crate::ChallengeEffect::Modeled,
            modeled.contains(&challenge),
            "{challenge:?}"
        );
    }
}

fn page_count(report: &SeedReport, page: ItemId) -> usize {
    report
        .floors
        .iter()
        .filter_map(|floor| floor.map.as_ref().or(floor.assumed_map.as_ref()))
        .flat_map(|map| &map.heaps)
        .flat_map(|heap| &heap.items)
        .filter(|item| item.item == page)
        .count()
}

#[test]
fn complete_journal_spawns_no_lore_pages() {
    let report = analyze_seed_for_run("7", 12, None, &[], true).unwrap();
    assert_eq!(page_count(&report, ItemId::GuidePage), 0);
    assert_eq!(page_count(&report, ItemId::AlchemyPage), 0);
    assert!(crate::journal::lore_complete());
}

#[test]
fn fresh_journal_spawns_guide_and_alchemy_pages() {
    let report = analyze_seed_for_run("7", 12, None, &[], false).unwrap();
    assert!(page_count(&report, ItemId::GuidePage) > 0);
    assert!(page_count(&report, ItemId::AlchemyPage) > 0);
    assert!(
        crate::journal::lore_complete(),
        "the journal state reverts after the run"
    );
}

#[test]
fn champion_runs_publish_every_floor_map() {
    let report = analyze_seed_for_run(
        NINE_CHALLENGE_SEED,
        6,
        None,
        &[Challenge::ChampionEnemies],
        true,
    )
    .unwrap();
    let first = &report.floors[0];
    let map = first.map.as_ref().expect("floor 1 champion is seeded");
    assert!(map.uncertain_cells.is_empty());
    for depth in 2..=6 {
        displayed_map(&report, depth);
    }
}
