use super::*;
use crate::items::model::{ItemCategory, ItemProvenance, QuestRewardRole};
use crate::quests::imp::ImpSlotKind;
use crate::run::{dungeon_from_run, init_run};
use crate::{MapProfile, TrinketEvent, TrinketEventAction, TrinketKind};
use serde::Deserialize;
use std::collections::BTreeSet;

const RING_DECK_FIXTURES: [&str; 5] = [
    include_str!(
        "../../../../../tools/java-oracle/fixtures/generator/aaa-aaa-aaa-imp-ring-deck.json"
    ),
    include_str!(
        "../../../../../tools/java-oracle/fixtures/generator/abc-def-ghi-imp-ring-deck.json"
    ),
    include_str!(
        "../../../../../tools/java-oracle/fixtures/generator/gfx-pzh-dch-imp-ring-deck.json"
    ),
    include_str!(
        "../../../../../tools/java-oracle/fixtures/generator/hkt-jzn-xqq-imp-ring-deck.json"
    ),
    include_str!(
        "../../../../../tools/java-oracle/fixtures/generator/zzz-zzz-zzz-imp-ring-deck.json"
    ),
];

#[derive(Deserialize)]
struct RingDeckFixture {
    schema_version: u32,
    contract: String,
    spd: FixturePin,
    input: FixtureInput,
    spawn: FixtureSpawn,
}

#[derive(Deserialize)]
struct FixturePin {
    version: String,
    commit: String,
}

#[derive(Deserialize)]
struct FixtureInput {
    seed: String,
    numeric: i64,
}

#[derive(Deserialize)]
struct FixtureSpawn {
    depth: i32,
    ring_dropped_before: i32,
    ring_dropped_after: i32,
    artifact_dropped_before: i32,
    artifact_dropped_after: i32,
    wand_dropped_before: i32,
    wand_dropped_after: i32,
    wep_t4_dropped_before: i32,
    wep_t4_dropped_after: i32,
    wep_t5_dropped_before: i32,
    wep_t5_dropped_after: i32,
    mis_t4_dropped_before: i32,
    mis_t4_dropped_after: i32,
    mis_t5_dropped_before: i32,
    mis_t5_dropped_after: i32,
    reward_options: Vec<RewardOption>,
}

#[derive(Deserialize)]
struct RewardOption {
    class: String,
    level: i32,
    cursed: bool,
}

fn replay_through(seed: i64, max_depth: i32) -> crate::dungeon::DungeonState {
    let mut dungeon = dungeon_from_run(init_run(seed));
    for depth in 1..=max_depth {
        dungeon.depth = depth;
        let _ = crate::level::create_level_partial(&mut dungeon);
    }
    dungeon
}

fn profiled_imp_outcome(seed: i64, trinket: Option<TrinketKind>) -> (i32, Vec<String>) {
    let mut dungeon = dungeon_from_run(init_run(seed));
    let profile = MapProfile {
        trinket_events: trinket
            .into_iter()
            .flat_map(|trinket| {
                [
                    TrinketEvent {
                        before_depth: 17,
                        action: TrinketEventAction::Acquired {
                            trinket,
                            min_upgrades: None,
                        },
                    },
                    TrinketEvent {
                        before_depth: 17,
                        action: TrinketEventAction::Upgraded,
                    },
                    TrinketEvent {
                        before_depth: 17,
                        action: TrinketEventAction::Upgraded,
                    },
                    TrinketEvent {
                        before_depth: 17,
                        action: TrinketEventAction::Upgraded,
                    },
                ]
            })
            .collect(),
        ..MapProfile::default()
    };
    crate::level::analyze_floors_with_profile(&mut dungeon, 19, Some(&profile));
    (
        dungeon.imp.depth,
        dungeon
            .imp
            .reward_options
            .iter()
            .map(|item| item.class_name.clone())
            .collect(),
    )
}

const V4_WEAPON_ENCHANTS: [&str; 4] = ["Venomous", "Eldritch", "Vorpal", "Crystal"];
const ARMOR_GLYPHS: [&str; 13] = [
    "Obfuscation",
    "Swiftness",
    "Viscosity",
    "Potential",
    "Brimstone",
    "Stone",
    "Entanglement",
    "Repulsion",
    "Camouflage",
    "Flow",
    "Affection",
    "AntiMagic",
    "Thorns",
];

fn generate_reward_options_at(seed: i64) -> Vec<GeneratedItem> {
    let mut generator = init_run(seed).generator;
    Random::reset_generators();
    Random::push_generator_seeded(seed);
    let (options, _draws) = rewards::generate_reward_options(&mut generator, 18);
    Random::pop_generator();
    options
}

#[test]
fn imp_ring_deck_fixture_pins_v4_spawn_draw_sites() {
    // Pinned to a fresh-journal Java oracle run.
    let _journal = crate::journal::scope_lore_complete(false);
    for fixture_json in RING_DECK_FIXTURES {
        let fixture: RingDeckFixture =
            serde_json::from_str(fixture_json).expect("Imp ring deck fixture");
        assert_eq!(fixture.schema_version, 1);
        assert_eq!(fixture.contract, "imp_ring_deck");
        assert_eq!(fixture.spd.version, crate::SPD_VERSION);
        assert_eq!(fixture.spd.commit, crate::SPD_COMMIT);
        assert_eq!(
            fixture.input.numeric,
            crate::parse_seed(&fixture.input.seed)
                .expect("oracle seed")
                .numeric
        );
        assert!(
            (17..=19).contains(&fixture.spawn.depth),
            "{} spawn depth",
            fixture.input.seed
        );

        let dungeon = replay_through(fixture.input.numeric, fixture.spawn.depth);
        let spawn = &fixture.spawn;
        assert_eq!(
            dungeon.imp.depth, spawn.depth,
            "{} spawn depth",
            fixture.input.seed
        );
        let expected_delta = ImpSpawnDecks {
            ring: spawn.ring_dropped_after - spawn.ring_dropped_before,
            artifact: spawn.artifact_dropped_after - spawn.artifact_dropped_before,
            wand: spawn.wand_dropped_after - spawn.wand_dropped_before,
            wep_t4: spawn.wep_t4_dropped_after - spawn.wep_t4_dropped_before,
            wep_t5: spawn.wep_t5_dropped_after - spawn.wep_t5_dropped_before,
            mis_t4: spawn.mis_t4_dropped_after - spawn.mis_t4_dropped_before,
            mis_t5: spawn.mis_t5_dropped_after - spawn.mis_t5_dropped_before,
        };
        let actual_delta = ImpSpawnDecks {
            ring: dungeon.imp.dropped_after.ring - dungeon.imp.dropped_before.ring,
            artifact: dungeon.imp.dropped_after.artifact - dungeon.imp.dropped_before.artifact,
            wand: dungeon.imp.dropped_after.wand - dungeon.imp.dropped_before.wand,
            wep_t4: dungeon.imp.dropped_after.wep_t4 - dungeon.imp.dropped_before.wep_t4,
            wep_t5: dungeon.imp.dropped_after.wep_t5 - dungeon.imp.dropped_before.wep_t5,
            mis_t4: dungeon.imp.dropped_after.mis_t4 - dungeon.imp.dropped_before.mis_t4,
            mis_t5: dungeon.imp.dropped_after.mis_t5 - dungeon.imp.dropped_before.mis_t5,
        };
        assert_eq!(
            actual_delta, expected_delta,
            "{} dropped deltas before={:?} after={:?}",
            fixture.input.seed, dungeon.imp.dropped_before, dungeon.imp.dropped_after
        );

        assert_eq!(
            dungeon.imp.reward_options.len(),
            6,
            "{} rewardOptions after initRooms",
            fixture.input.seed
        );
        let options = &dungeon.imp.reward_options;
        assert_eq!(options[4].class_name, "PlateArmor");
        assert_eq!(options[1].category, ItemCategory::Ring);
        assert_eq!(options[5].category, ItemCategory::Wand);
        assert!(matches!(
            options[0].category,
            ItemCategory::Artifact | ItemCategory::Ring
        ));
        assert!(options.iter().all(|item| !item.cursed));
        assert!(spawn.reward_options.iter().all(|item| !item.cursed));
        // Class identity still follows persistent decks from prior floors
        // (unported city layout). Ambient IntRange/coin-flip overwrites do not.
        for (index, (actual, expected)) in options.iter().zip(&spawn.reward_options).enumerate() {
            if index == 0 && actual.class_name != expected.class {
                continue;
            }
            assert_eq!(
                actual.level, expected.level,
                "{} reward[{index}] {} level (class {})",
                fixture.input.seed, expected.class, actual.class_name
            );
        }
    }
}

#[test]
fn transfer_upgrade_uses_artifact_level_cap() {
    assert_eq!(rewards::transfer_upgrade_level("SandalsOfNature", 5), 2);
    assert_eq!(
        rewards::transfer_upgrade_level("TimekeepersHourglass", 5),
        3
    );
    assert_eq!(rewards::transfer_upgrade_level("EtherealChains", 5), 3);
    assert_eq!(rewards::transfer_upgrade_level("TalismanOfForesight", 5), 5);
    assert_eq!(
        rewards::transfer_upgrade_level("MasterThievesArmband", 5),
        5
    );
}

#[test]
fn reward_options_are_deterministic_uncursed_and_take_one() {
    let gen_template = init_run(42).generator;

    Random::reset_generators();
    Random::push_generator_seeded(777);
    let (first, first_draws) = rewards::generate_reward_options(&mut gen_template.clone(), 18);
    Random::pop_generator();

    Random::reset_generators();
    Random::push_generator_seeded(777);
    let (second, _) = rewards::generate_reward_options(&mut gen_template.clone(), 18);
    Random::pop_generator();

    assert_eq!(first.len(), 6);
    assert_eq!(
        first
            .iter()
            .map(|item| (&item.class_name, item.level, item.cursed, &item.enchantment))
            .collect::<Vec<_>>(),
        second
            .iter()
            .map(|item| (&item.class_name, item.level, item.cursed, &item.enchantment))
            .collect::<Vec<_>>()
    );
    assert!(first.iter().all(|item| !item.cursed));
    assert!(first
        .iter()
        .all(|item| item.source.as_deref() == Some("Imp.Quest")));
    assert_eq!(first[4].class_name, "PlateArmor");
    assert_eq!(first[4].category, ItemCategory::Armor);
    assert!(first[1].category == ItemCategory::Ring);
    assert!(first[5].category == ItemCategory::Wand);
    for (slot, item) in first.iter().enumerate() {
        assert_eq!(
            item.provenance,
            ItemProvenance::Quest(QuestRewardRole::ImpVaultOption { slot: slot as u8 })
        );
    }

    // Draw sites are recorded once per slot, and only the plate slot skips a deck.
    assert_eq!(first_draws.len(), 6);
    assert_eq!(first_draws[4].category, None);
    assert!(first_draws
        .iter()
        .enumerate()
        .all(|(slot, draw)| slot == 4 || draw.category.is_some()));
}

#[test]
fn exhausted_artifact_deck_falls_back_to_a_distinct_ring() {
    let mut generator = init_run(42).generator;
    Random::reset_generators();
    Random::push_generator_seeded(777);
    while generator.random_artifact(18).is_some() {}
    let (options, draws) = rewards::generate_reward_options(&mut generator, 18);
    Random::pop_generator();

    // Both slot 0 and slot 1 are RING-deck draws once artifacts run out.
    assert_eq!(draws[0].kind, ImpSlotKind::Ring);
    assert_eq!(draws[0].category, Some(Category::Ring));
    assert_eq!(options[0].category, ItemCategory::Ring);
    assert_eq!(options[1].category, ItemCategory::Ring);
    assert_ne!(options[0].class_name, options[1].class_name);
}

#[test]
fn depth19_always_spawns_when_not_spawned() {
    Random::reset_generators();
    let mut imp = ImpQuestState::default();
    let mut generator = init_run(1).generator;
    Random::push_generator_seeded(1);
    let mut specs = Vec::new();
    assert!(try_spawn(&mut imp, &mut generator, 19, &mut specs));
    assert_eq!(specs[0].name, "AmbitiousImpRoom");
    assert_eq!(imp.reward_options.len(), 6);
    let result = take_pending(&mut imp).expect("spawn rewards");
    assert_eq!(result.options.len(), 6);
    assert!(result.options.iter().all(|item| !item.cursed));
    Random::pop_generator();
}

#[test]
fn depth16_never_spawns() {
    Random::reset_generators();
    let mut imp = ImpQuestState::default();
    let mut generator = init_run(1).generator;
    let mut specs = Vec::new();
    assert!(!try_spawn(&mut imp, &mut generator, 16, &mut specs));
    assert!(specs.is_empty());
}

#[test]
fn held_mossy_clump_does_not_shift_imp_spawn_depth() {
    // Both default-branch floats are consumed; GRASS/WATER do not change initRooms.
    assert_eq!(profiled_imp_outcome(5, None).0, 17);
    assert_eq!(profiled_imp_outcome(5, Some(TrinketKind::MossyClump)).0, 17);
}

#[test]
fn held_mossy_clump_does_not_shift_imp_reward_identities() {
    let baseline = profiled_imp_outcome(0, None);
    let mossy = profiled_imp_outcome(0, Some(TrinketKind::MossyClump));
    assert_eq!(baseline.0, 19);
    assert_eq!(mossy, baseline);
}

#[test]
fn imp_weapon_enchants_intersect_v4_names() {
    let mut weapons = BTreeSet::new();
    let mut glyphs = BTreeSet::new();
    for seed in 0..200 {
        let options = generate_reward_options_at(seed);
        for item in &options[2..=3] {
            weapons.insert(
                item.enchantment
                    .clone()
                    .expect("Imp weapon slots always overwrite enchantment"),
            );
        }
        glyphs.insert(
            options[4]
                .enchantment
                .clone()
                .expect("Imp plate always overwrites glyph"),
        );
    }
    assert!(
        V4_WEAPON_ENCHANTS
            .iter()
            .copied()
            .any(|name| weapons.contains(name)),
        "expected a v4 weapon enchant in Imp slots 2–3, got {weapons:?}"
    );
    assert!(
        glyphs
            .iter()
            .all(|name| ARMOR_GLYPHS.contains(&name.as_str())),
        "Imp plate glyphs should stay on the v3 set, got {glyphs:?}"
    );
}

#[test]
fn imp_reward_enchant_pin() {
    let options = generate_reward_options_at(2);
    assert_eq!(options[2].enchantment.as_deref(), Some("Unstable"));
    assert_eq!(options[3].enchantment.as_deref(), Some("Eldritch"));
    assert_eq!(options[4].enchantment.as_deref(), Some("Swiftness"));
}

/// The load-bearing claim behind reward distributions: a category's draw
/// sequence is a pure function of the seed, so deck index N in a from-zero
/// replay equals the draw that actually happened at index N. Pinned against
/// java-oracle ground truth.
#[test]
fn deck_index_matches_real_draw() {
    for fixture_json in RING_DECK_FIXTURES {
        let fixture: RingDeckFixture = serde_json::from_str(fixture_json).unwrap();
        let seed = fixture.input.numeric;
        let dungeon = replay_through(seed, fixture.spawn.depth);
        let idx = fixture.spawn.ring_dropped_before as usize;
        let seq = dungeon
            .generator
            .category_class_history(Category::Ring, fixture.spawn.depth);
        // reward_options[1] is always the ring slot.
        let actual_ring = &fixture.spawn.reward_options[1].class;
        eprintln!(
            "seed={} idx={} seq={:?} actual={}",
            fixture.input.seed, idx, seq, actual_ring
        );
        assert_eq!(
            seq.get(idx).map(String::as_str),
            Some(actual_ring.as_str()),
            "seed {} ring index {}",
            fixture.input.seed,
            idx
        );
    }
}

/// Drift is bidirectional: a trinket profile can move the Imp's deck index in
/// either direction, and the from-zero sequence predicts the new draw either
/// way. Seed 42 moves backward (5 -> 3), which is why the candidate window
/// cannot be forward-only.
#[test]
fn drift_matches_profile_replay_in_both_directions() {
    for seed in [0_i64, 1, 2, 7, 42] {
        // Baseline.
        let base = replay_through(seed, 19);
        if !base.imp.spawned {
            continue;
        }
        let base_idx = base.imp.dropped_before.ring as usize;
        let base_ring = base.imp.reward_options[1].class_name.clone();

        // Profiled run with a trinket that adds deck-using sites earlier.
        let mut dungeon = dungeon_from_run(init_run(seed));
        let profile = MapProfile {
            trinket_events: vec![TrinketEvent {
                before_depth: 17,
                action: TrinketEventAction::Acquired {
                    trinket: TrinketKind::MimicTooth,
                    min_upgrades: None,
                },
            }],
            ..MapProfile::default()
        };
        crate::level::analyze_floors_with_profile(&mut dungeon, 19, Some(&profile));
        if !dungeon.imp.spawned {
            continue;
        }
        let prof_idx = dungeon.imp.dropped_before.ring as usize;
        let prof_ring = dungeon.imp.reward_options[1].class_name.clone();

        // The from-zero sequence taken from the PROFILED generator.
        let seq = dungeon
            .generator
            .category_class_history(Category::Ring, dungeon.imp.depth);
        eprintln!(
            "seed={seed} base(idx={base_idx},{base_ring}) prof(idx={prof_idx},{prof_ring}) seq={seq:?}"
        );
        assert_eq!(
            seq.get(prof_idx).map(String::as_str),
            Some(prof_ring.as_str()),
            "seed {seed}: profiled ring index {prof_idx}"
        );

        // Cross-check: the BASELINE generator's sequence, extended, must predict
        // the profiled ring too -- that is the forward-drift claim.
        let base_seq = base
            .generator
            .category_class_history(Category::Ring, base.imp.depth);
        if prof_idx < base_seq.len() {
            assert_eq!(
                base_seq.get(prof_idx).map(String::as_str),
                Some(prof_ring.as_str()),
                "seed {seed}: baseline sequence must predict profiled ring at index {prof_idx}"
            );
        } else {
            eprintln!(
                "  (profiled index {prof_idx} beyond baseline window {})",
                base_seq.len()
            );
        }
    }
}

/// Upgrades and enchants survive a change of run profile.
///
/// Everything the city floor spends before `Imp.Quest.spawn` is seed-fixed:
/// the limited-drop checks gate `addItemToSpawn` without rolling
/// (`Level.java:224-247`), and the feeling switch is one `Random.Int(14)` whose
/// branches each spend a fixed amount (`Level.java:258-296`). So a held trinket
/// or an enabled challenge can move a deck index without moving a single `+N`.
#[test]
fn upgrades_survive_profile_changes() {
    let profiles: Vec<(&str, MapProfile)> = vec![
        ("baseline", MapProfile::default()),
        (
            "forbidden_runes",
            MapProfile {
                challenges: vec![crate::Challenge::ForbiddenRunes],
                ..MapProfile::default()
            },
        ),
        (
            "mimic_tooth",
            MapProfile {
                trinket_events: vec![TrinketEvent {
                    before_depth: 17,
                    action: TrinketEventAction::Acquired {
                        trinket: TrinketKind::MimicTooth,
                        min_upgrades: None,
                    },
                }],
                ..MapProfile::default()
            },
        ),
        (
            "rat_skull_and_forbidden_runes",
            MapProfile {
                challenges: vec![crate::Challenge::ForbiddenRunes],
                trinket_events: vec![TrinketEvent {
                    before_depth: 17,
                    action: TrinketEventAction::Acquired {
                        trinket: TrinketKind::RatSkull,
                        min_upgrades: None,
                    },
                }],
                ..MapProfile::default()
            },
        ),
    ];

    /// Spawn depth plus the `(level, enchant)` of each slot.
    type UpgradeFacts = (i32, Vec<(i32, Option<String>)>);

    let mut checked = 0;
    for seed in [0_i64, 7, 42, 99, 1234] {
        let mut expected: Option<UpgradeFacts> = None;
        for (label, profile) in &profiles {
            let mut dungeon = dungeon_from_run(init_run(seed));
            crate::level::analyze_floors_with_profile(&mut dungeon, 19, Some(profile));
            assert!(
                dungeon.imp.spawned,
                "seed {seed} {label}: Imp always spawns"
            );
            let actual = (
                dungeon.imp.depth,
                dungeon
                    .imp
                    .reward_options
                    .iter()
                    .map(|item| (item.level, item.enchantment.clone()))
                    .collect::<Vec<_>>(),
            );
            match &expected {
                None => expected = Some(actual),
                Some(first) => assert_eq!(
                    &actual, first,
                    "seed {seed}: profile {label} moved the spawn depth or an upgrade"
                ),
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 5);
}
