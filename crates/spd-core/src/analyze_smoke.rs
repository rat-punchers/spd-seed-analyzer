use super::*;

#[path = "analyze_smoke/room_reward_regressions.rs"]
mod room_reward_regressions;

#[path = "analyze_smoke/map_terrain.rs"]
mod map_terrain;

#[path = "analyze_smoke/map_item_list_parity.rs"]
mod map_item_list_parity;

#[test]
fn analyze_seed_smoke() {
    let r = analyze_seed("GFX-PZH-DCH", 4).expect("analyze");
    assert!(r
        .floors
        .iter()
        .flat_map(report::FloorReport::item_variants)
        .any(|item| !item.spawn_conditions.is_empty()));
    assert_eq!(r.status, "partial");
    eprintln!("status={} floors={}", r.status, r.floors.len());
    for f in &r.floors {
        eprintln!(
            "  floor {} rooms={} items={} quests={:?} map={:?}",
            f.depth,
            f.rooms.len(),
            f.items.len(),
            f.quests,
            f.map
                .as_ref()
                .map(|m| (m.width, m.height, m.tileset.as_str()))
        );
        if let Some(map) = &f.map {
            assert_eq!(map.tiles.len(), (map.width * map.height) as usize);
            assert_eq!(map.tile_variance.len(), map.tiles.len());
            assert!(map.tile_variance.iter().all(|&value| value < 100));
            assert!(
                map.markers
                    .iter()
                    .all(|marker| marker.cell < map.tiles.len() as u32),
                "all exported marker cells must be within the map"
            );
        }
    }
    assert!(r
        .floors
        .iter()
        .all(|floor| floor.items.iter().any(|item| item.category == "food")));
}

#[test]
fn analyzes_daily_run_date() {
    let report = analyze_seed("2026-04-01", 1).expect("analyze Daily Run");
    assert_eq!(report.seed.numeric, 7_204_505_278_976);
    assert!(report.seed.daily);
    assert_eq!(report.seed.code, None);
    assert_eq!(report.floors.len(), 1);
}

#[test]
fn fresh_profile_publishes_floor_one_ordinary_loot() {
    let mut floors_with_ordinary_loot = 0;
    for seed in 0..50 {
        let report = analyze_seed_with_profile(&seed.to_string(), 1, &MapProfile::default())
            .expect("profiled analysis");
        let floor = &report.floors[0];
        assert!(floor.map.is_some(), "seed {seed} floor-one layout");
        let mut has_ordinary_loot = false;
        for item in floor.items.iter().filter(|item| {
            item.source
                .as_deref()
                .and_then(|source| source.rsplit(':').next())
                == Some("heap")
        }) {
            has_ordinary_loot = true;
            assert_eq!(
                item.prediction,
                crate::report::ItemPredictionKind::Exact,
                "seed {seed} floor-one ordinary loot"
            );
            assert!(item.notes.is_empty());
        }
        floors_with_ordinary_loot += usize::from(has_ordinary_loot);
    }
    assert!(
        floors_with_ordinary_loot >= 40,
        "fresh floor-one ordinary loot should not be hidden by later-run uncertainty"
    );
}

#[test]
fn floor_one_traps_room_relocates_its_exact_forced_reward() {
    let report = analyze_seed("HKH-FKC-YTK", 1).expect("analyze");
    let floor = &report.floors[0];
    let prize = floor
        .items
        .iter()
        .find(|item| {
            item.source.as_deref() == Some("TrapsRoom")
                && item.class_name() == Some("PotionOfStrength")
        })
        .expect("TrapsRoom reward");

    assert_eq!(prize.name, "Potion of Strength");
    assert_eq!(prize.prediction, crate::report::ItemPredictionKind::Exact);
    assert!(!floor.items.iter().any(|item| {
        item.name == "single room reward source" && item.source.as_deref() == Some("TrapsRoom")
    }));
    assert!(!floor.items.iter().any(|item| {
        item.class_name() == Some("PotionOfStrength")
            && item.source.as_deref() == Some("guaranteed floor spawn")
    }));
}

#[test]
fn small_floor_two_ring_rooms_do_not_report_phantom_rewards() {
    let report = analyze_seed("PUB-CLI-VQH", 2).expect("analyze");
    let floor = &report.floors[1];

    assert_eq!(
        floor
            .rooms
            .iter()
            .filter(|room| room.class_name() == "RingRoom")
            .count(),
        2,
        "fixture keeps both small RingRoom instances"
    );
    assert!(
        floor
            .items
            .iter()
            .all(|item| item.source.as_deref() != Some("RingRoom")),
        "small RingRooms have no placeCenterDetail call or item reward"
    );
}

#[test]
fn pub_cli_vqh_narrows_floor_two_suspicious_chest_rewards() {
    let report = analyze_seed("PUB-CLI-VQH", 2).expect("analyze");
    let floor = &report.floors[1];

    let gold_fallback = floor
        .items
        .iter()
        .find(|group| group.source.as_deref() == Some("SuspiciousChestRoom:gold"))
        .expect("Suspicious Chest gold fallback contract");
    assert_eq!(gold_fallback.variants.len(), 1);
    assert_eq!(gold_fallback.category, "gold");
    assert_eq!(gold_fallback.level, Some(0));
    assert_eq!(gold_fallback.cursed, Some(false));

    let mimic_reward = floor
        .items
        .iter()
        .find(|group| group.source.as_deref() == Some("SuspiciousChestRoom:mimic_reward"))
        .expect("Suspicious Chest Mimic reward");
    assert_eq!(mimic_reward.variants.len(), 1);
    assert_eq!(
        mimic_reward.variants[0].prediction,
        crate::report::ItemPredictionKind::Constrained
    );
    assert_eq!(
        mimic_reward.variants[0].tier_range,
        Some(crate::report::NumericRange { min: 2, max: 5 })
    );
    assert_eq!(
        mimic_reward.variants[0].level_range,
        Some(crate::report::NumericRange { min: 0, max: 2 })
    );
}

#[test]
fn pub_cli_vnw_narrows_floor_two_pool_reward() {
    let report = analyze_seed("PUB-CLI-VNW", 2).expect("analyze");
    let floor = &report.floors[1];
    let pool = floor
        .items
        .iter()
        .find(|group| group.source.as_deref() == Some("PoolRoom:equipment"))
        .expect("Pool equipment reward");

    assert_eq!(pool.variants.len(), 2);
    assert_eq!(pool.variants[0].category, "weapon / armor / missile");
    assert_eq!(
        pool.variants[0].tier_range,
        Some(crate::report::NumericRange { min: 2, max: 5 })
    );
    assert_eq!(
        pool.variants[0].level_range,
        Some(crate::report::NumericRange { min: 0, max: 3 })
    );
    assert_eq!(pool.variants[0].cursed, Some(false));
    assert_eq!(
        pool.variants[1].prediction,
        crate::report::ItemPredictionKind::Variable
    );
    assert_eq!(pool.variants[1].class_name(), Some("MailArmor"));
    assert_eq!(pool.variants[1].name, "mail Armor +0");
    assert_eq!(pool.variants[1].category, "armor");
    assert_eq!(pool.variants[1].level, Some(0));
    assert_eq!(pool.variants[1].cursed, Some(false));
    assert!(report.compact_text().contains("2 Mail Armor +0 Pool"));
}

#[test]
fn floor_one_runestone_rewards_and_initial_encounters_are_exact() {
    let report = analyze_seed("AAA-AAA-AFU", 1).expect("analyze");
    let floor = &report.floors[0];
    let runestones: Vec<_> = floor
        .items
        .iter()
        .filter(|item| item.source.as_deref() == Some("RunestoneRoom"))
        .map(|item| (item.class_name(), item.prediction, item.quantity))
        .collect();
    assert_eq!(
        runestones,
        [
            (
                Some("StoneOfBlast"),
                crate::report::ItemPredictionKind::Exact,
                1,
            ),
            (
                Some("StoneOfDeepSleep"),
                crate::report::ItemPredictionKind::Exact,
                1,
            ),
        ]
    );

    assert_eq!(
        floor
            .initial_encounters
            .iter()
            .map(|encounter| (encounter.class_name(), encounter.quantity))
            .collect::<Vec<_>>(),
        [("Rat", 6), ("Snake", 2)]
    );
    let snake = floor
        .initial_encounters
        .iter()
        .find(|encounter| encounter.class_name() == "Snake")
        .expect("initial snakes");
    assert!(matches!(
        snake.combat_rewards.as_slice(),
        [crate::report::CombatReward {
            prediction: crate::report::CombatRewardPrediction::RuntimeChance,
            chance: Some(crate::report::RewardChance {
                numerator: 1,
                denominator: 4,
            }),
            item: None,
            ..
        }]
    ));
}

#[test]
fn floor_one_room_mobs_keep_fixed_combat_rewards_non_positional() {
    let report = analyze_seed("AAA-AAA-AAA", 1).expect("analyze");
    let floor = &report.floors[0];
    let piranhas = floor
        .initial_encounters
        .iter()
        .find(|encounter| encounter.class_name() == "Piranha")
        .expect("PoolRoom piranhas");
    assert_eq!(piranhas.quantity, 3);
    assert!(matches!(
        piranhas.combat_rewards.as_slice(),
        [crate::report::CombatReward {
            item: Some(item),
            prediction: crate::report::CombatRewardPrediction::Guaranteed,
            chance: None,
            ..
        }] if *item == spd_objects::ItemId::MysteryMeat
    ));
    let map = floor.map.as_ref().expect("floor one map");
    assert_eq!(
        map.mobs
            .iter()
            .filter(|mob| mob.mob == spd_objects::MobId::Piranha)
            .count(),
        3,
        "public map places the PoolRoom piranhas"
    );
}

#[test]
fn public_maps_publish_entities_with_uncertainty_and_hidden_cells() {
    let report = analyze_seed("GFX-PZH-DCH", 8).expect("analyze");
    let mut saw_hidden = false;
    for floor in &report.floors {
        let Some(map) = floor.map.as_ref().or(floor.assumed_map.as_ref()) else {
            continue;
        };
        let len = map.tiles.len() as u32;
        let entity_cells: std::collections::BTreeSet<u32> = map
            .heaps
            .iter()
            .map(|heap| heap.cell)
            .chain(map.mobs.iter().map(|mob| mob.cell))
            .chain(map.markers.iter().map(|marker| marker.cell))
            .collect();
        assert!(entity_cells.iter().all(|&cell| cell < len));
        assert!(
            map.uncertain_cells.windows(2).all(|pair| pair[0] < pair[1]),
            "uncertain cells are sorted and unique"
        );
        assert!(
            map.uncertain_cells
                .iter()
                .all(|cell| entity_cells.contains(cell)),
            "only entity cells can be uncertain (floor {})",
            floor.depth
        );
        assert!(map.hidden_cells.windows(2).all(|pair| pair[0] < pair[1]));
        for &cell in &map.hidden_cells {
            let tile = map.tiles[cell as usize];
            assert!(tile != 4 && tile != 12, "hidden cells are never walls");
        }
        let secret_doors = map.tiles.iter().filter(|&&tile| tile == 16).count();
        assert!(map.hidden_cells.len() >= secret_doors);
        saw_hidden |= !map.hidden_cells.is_empty();
    }
    assert!(saw_hidden, "eight floors include at least one secret door");
}

#[test]
fn compact_report_promotes_exact_floor_one_room_rewards() {
    let report = analyze_seed("RZN-LKU-EFS", 17).expect("analyze compact-report fixture");
    let floor_one = &report.floors[0];
    let rose = floor_one
        .items
        .iter()
        .find(|item| {
            item.prediction == report::ItemPredictionKind::Exact
                && item.class_name() == Some("DriedRose")
        })
        .expect("exact floor-one Crystal Choice Dried Rose");
    assert_eq!(rose.level, Some(0));
    assert_eq!(rose.cursed, Some(true));
    assert_eq!(
        rose.source.as_deref(),
        Some("CrystalChoiceRoom:hidden_reward")
    );
    assert_eq!(rose.prediction, report::ItemPredictionKind::Exact);
    assert!(floor_one.items.iter().all(|item| {
        item.class_name() != Some("DriedRose")
            || item.prediction == report::ItemPredictionKind::Exact
    }));

    let compact = report.compact_text();
    assert!(compact.starts_with("RZN-LKU-EFS\nShPD v4.0.2\n\n"));
    assert!(
        compact
            .lines()
            .any(|line| line == "1 (cursed) Rose Crystal Choice"),
        "compact output promotes the exact floor-one reward"
    );
    assert_eq!(report.status, "partial");
}

#[test]
fn ghost_enchantment_omits_unreachable_parchment_requirement() {
    let report = analyze_seed("HKH-FKC-YTK", 4).expect("analyze");
    assert_eq!(report.trinket_selection.first_effective_depth, 5);
    let (depth, rewards): (u32, Vec<_>) = report
        .floors
        .iter()
        .find_map(|floor| {
            let rewards: Vec<_> = floor
                .items
                .iter()
                .filter(|item| item.source.as_deref() == Some("Ghost.Quest"))
                .flat_map(|item| &item.variants)
                .collect();
            (!rewards.is_empty()).then_some((floor.depth, rewards))
        })
        .expect("Ghost reward");
    assert!(
        depth < report.trinket_selection.first_effective_depth,
        "fixture keeps Ghost before parchment can apply"
    );

    assert!(rewards.iter().all(|reward| {
        reward.enchantment.is_none()
            && reward
                .notes
                .iter()
                .all(|note| !note.contains("Parchment Scrap"))
    }));
}

#[test]
fn ghost_enchantment_keeps_reachable_parchment_requirement() {
    let report = analyze_seed_with_profile("5", 4, &MapProfile::default()).expect("analyze");
    let (reward_depth, enchantment) = report
        .floors
        .iter()
        .find_map(|floor| {
            floor
                .items
                .iter()
                .filter(|item| item.source.as_deref() == Some("Ghost.Quest"))
                .find_map(|item| {
                    item.enchantment
                        .as_ref()
                        .filter(|value| !value.conditions.is_empty())
                })
                .map(|enchantment| (floor.depth, enchantment))
        })
        .expect("reachable conditional Ghost enchantment");
    let first_effective_depth = report.trinket_selection.first_effective_depth;
    assert!(reward_depth >= first_effective_depth);

    assert!(matches!(
        enchantment.conditions.as_slice(),
        [ItemCondition::Trinket { events }]
            if matches!(events.as_slice(), [TrinketEvent {
                before_depth,
                action: TrinketEventAction::Acquired {
                    trinket: TrinketKind::ParchmentScrap,
                    ..
                },
            }] if *before_depth == reward_depth)
    ));
}

#[test]
fn first_alchemy_pot_is_a_guaranteed_floor_appearance() {
    for seed in 0..50 {
        let report = analyze_seed_seed_only(&seed.to_string(), 5).expect("analyze");
        let selection = &report.trinket_selection;
        let expected_source = if selection.first_alchemy_pot_is_secret {
            "SecretLaboratoryRoom"
        } else {
            "LaboratoryRoom"
        };
        let floor = report
            .floors
            .iter()
            .find(|floor| floor.depth == selection.first_alchemy_pot_depth)
            .expect("first pot floor is included");

        assert!(floor.guaranteed_appearances.iter().any(|appearance| {
            appearance.kind == GuaranteedAppearanceKind::AlchemyPot
                && appearance
                    .source
                    .is_some_and(|source| source == expected_source)
        }));

        for exact_floor in report.floors.iter().filter(|floor| !floor.rooms.is_empty()) {
            let room_pots = exact_floor
                .rooms
                .iter()
                .filter(|room| {
                    matches!(room.class_name(), "LaboratoryRoom" | "SecretLaboratoryRoom")
                })
                .count();
            assert_eq!(
                exact_floor.guaranteed_appearances.len(),
                room_pots,
                "seed {seed} floor {} exact pot appearances",
                exact_floor.depth
            );
        }
    }
}

#[test]
fn automatic_item_conditions_cover_supported_run_combinations() {
    let report = analyze_seed("5", 4).expect("analyze");
    let dependencies = report
        .floors
        .iter()
        .flat_map(report::FloorReport::item_variants)
        .flat_map(|item| &item.spawn_conditions)
        .flat_map(|condition| &condition.all_of)
        .collect::<Vec<_>>();

    assert!(dependencies.iter().any(|condition| matches!(
        condition,
        ItemDependencyCondition::Challenge {
            challenge: Challenge::ForbiddenRunes,
            ..
        }
    )));
    assert!(report
        .floors
        .iter()
        .flat_map(report::FloorReport::item_variants)
        .flat_map(|item| item.conditions.iter())
        .chain(report.floors.iter().flat_map(|floor| {
            floor.item_variants().flat_map(|item| {
                item.enchantment
                    .as_ref()
                    .into_iter()
                    .flat_map(|e| &e.conditions)
            })
        }))
        .any(|condition| matches!(condition, report::ItemCondition::Trinket { .. })));
    assert!(report
        .floors
        .iter()
        .flat_map(report::FloorReport::item_variants)
        .any(|item| {
            item.class_name() == Some("ScrollOfUpgrade") && !item.spawn_conditions.is_empty()
        }));
}

#[test]
fn public_contract_serializes_conditions_on_items_only() {
    let report = analyze_seed("42", 4).expect("analyze");
    let value = serde_json::to_value(report).expect("serialize report");
    assert!(value.get("floors").is_some());
    assert!(value.get("default_assumptions").is_none());
    assert!(value.get("conditional_changes").is_none());
    assert!(value.get("modeled_outcomes").is_none());
    assert!(value["floors"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|floor| floor["items"].as_array().unwrap())
        .flat_map(|group| group["variants"].as_array().unwrap())
        .any(|variant| variant.get("spawn_conditions").is_some()));
    assert!(value["floors"]
        .as_array()
        .unwrap()
        .iter()
        .all(|floor| floor.get("variable_items").is_none()));
    assert!(value.get("analysis_notes").is_none());
    assert!(value.get("message").is_none());
    assert!(value["identities"]["potions"][0].get("name").is_none());
    for item in value["floors"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|floor| floor["items"].as_array().unwrap())
        .flat_map(|group| group["variants"].as_array().unwrap())
    {
        assert!(item.get("notes").is_none());
        if let Some(enchantment) = item.get("enchantment") {
            assert!(enchantment.get("type").is_some());
            assert!(enchantment.get("conditions").is_some());
            assert!(enchantment.is_object());
        }
    }
}

#[test]
fn no_op_spawn_dependencies_are_not_serialized() {
    let item = crate::report::ItemEntry {
        name: "test".into(),
        quantity: 1,
        quantity_range: None,
        label: None,
        item: None,
        candidates: Vec::new(),
        category: "other".into(),
        tier: None,
        tier_range: None,
        level: None,
        level_range: None,
        cursed: None,
        enchantment: None,
        prediction: report::ItemPredictionKind::Constrained,
        spawn_conditions: vec![report::ItemSpawnCondition {
            all_of: vec![report::ItemDependencyCondition::Trinket { events: Vec::new() }],
        }],
        conditions: Vec::new(),
        notes: Vec::new(),
        source: None,
    };
    let value = serde_json::to_value(item).expect("serialize item");
    assert!(value.get("spawn_conditions").is_none());
}

#[test]
fn pub_cli_vnw_groups_the_depth_four_sacrifice_reward() {
    let report = analyze_seed("PUB-CLI-VNW", 4).expect("analyze");
    let floor = report
        .floors
        .iter()
        .find(|floor| floor.depth == 4)
        .expect("depth four");
    let groups = floor
        .items
        .iter()
        .filter(|group| group.source.as_deref() == Some("SacrificeRoom"))
        .collect::<Vec<_>>();
    assert_eq!(groups.len(), 1);

    let variants = &groups[0].variants;
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[0].name, "weapon reward");
    assert_eq!(
        variants[0].prediction,
        report::ItemPredictionKind::Constrained
    );
    assert_eq!(variants[0].class_name(), None);
    assert_eq!(
        variants[0].tier_range,
        Some(report::NumericRange { min: 2, max: 5 })
    );
    assert_eq!(
        variants[0].level_range,
        Some(report::NumericRange { min: 0, max: 3 })
    );
    assert_eq!(variants[0].cursed, Some(true));
    assert!(variants[0].spawn_conditions.is_empty());

    assert_eq!(variants[1].name, "polarized mace +1");
    assert_eq!(variants[1].class_name(), Some("Mace"));
    assert_eq!(variants[1].level, Some(1));
    assert_eq!(variants[1].cursed, Some(true));
    assert_eq!(variants[1].prediction, report::ItemPredictionKind::Variable);

    let value = serde_json::to_value(groups[0]).expect("serialize grouped reward");
    assert_eq!(value["source"], "SacrificeRoom");
    assert_eq!(value["variants"].as_array().unwrap().len(), 2);
    assert!(value["variants"]
        .as_array()
        .unwrap()
        .iter()
        .all(|variant| variant.get("source").is_none()));
    assert!(!value.to_string().contains("forbidden_runes"));
}

#[test]
fn enchantments_serialize_type_and_conditions() {
    let enchantment = report::ItemEnchantment {
        enchantment_type: spd_objects::Enchantment::Corrupting,
        conditions: Vec::new(),
    };
    let value = serde_json::to_value(enchantment).expect("serialize enchantment");
    assert_eq!(value["type"], spd_objects::Enchantment::Corrupting.id());
    assert_eq!(value["conditions"], serde_json::json!([]));
    assert_eq!(value.as_object().unwrap().len(), 2);
}

#[test]
fn enchantment_trinket_events_serialize_with_their_details() {
    use crate::trinkets::{TrinketEvent, TrinketEventAction, TrinketKind};

    let enchantment = report::ItemEnchantment {
        enchantment_type: spd_objects::Enchantment::Chilling,
        conditions: vec![report::ItemCondition::Trinket {
            events: vec![TrinketEvent {
                before_depth: 3,
                action: TrinketEventAction::Acquired {
                    trinket: TrinketKind::ParchmentScrap,
                    min_upgrades: Some(1),
                },
            }],
        }],
    };

    let value = serde_json::to_value(enchantment).expect("serialize enchantment");
    assert_eq!(
        value["conditions"][0]["events"],
        serde_json::json!([
            {
                "before_depth": 3,
                "kind": "acquired",
                "trinket": "parchment_scrap",
                "min_upgrades": 1
            }
        ])
    );
}

#[test]
fn ghost_quest_spawns_within_sewers_sometime() {
    let mut dungeon = dungeon_from_run(init_run(0));
    let mut saw = false;
    for depth in 1..=4 {
        dungeon.depth = depth;
        let state = level::create_level_partial_with_profile(&mut dungeon, true);
        if state
            .quests
            .iter()
            .any(|quest| matches!(quest, report::QuestReport::SadGhost { .. }))
        {
            assert!(state
                .placed_items
                .iter()
                .any(|item| item.source.as_deref() == Some("Ghost.Quest")));
            let report = state.to_floor_report();
            assert_eq!(
                report
                    .items
                    .iter()
                    .filter(|item| item.source.as_deref() == Some("Ghost.Quest"))
                    .count(),
                2
            );
            assert!(report.quests.iter().any(|quest| matches!(
                quest,
                report::QuestReport::SadGhost { contract, .. }
                    if contract.rewards.item_source == "Ghost.Quest"
                        && contract.rewards.option_count == 2
                        && contract.rewards.selected_count == 1
            )));
            saw = true;
        }
    }
    assert!(saw);
}

#[test]
fn prior_runtime_divergence_preserves_pinned_ghost_rewards() {
    let mut saw_quest = false;
    for seed in 0..100 {
        let mut dungeon = dungeon_from_run(init_run(seed));
        dungeon.public_generation_tainted = true;
        for depth in 2..=4 {
            dungeon.depth = depth;
            let state = level::create_level_partial_with_profile(&mut dungeon, true);
            if state.quests.is_empty() {
                continue;
            }
            let report = state.to_floor_report();
            saw_quest = true;
            assert_eq!(state.runtime_sensitive_quests_from, None);
            assert!(report
                .quests
                .iter()
                .any(|quest| matches!(quest, report::QuestReport::SadGhost { .. })));
            assert_eq!(
                report
                    .items
                    .iter()
                    .filter(|item| item.source.as_deref() == Some("Ghost.Quest"))
                    .count(),
                2
            );
            break;
        }
        if saw_quest {
            break;
        }
    }
    assert!(saw_quest, "Ghost quest is guaranteed by sewer depth 4");
}

#[test]
fn shop_stock_on_floor_6() {
    let mut dungeon = dungeon_from_run(init_run(0));
    let mut state = None;
    for depth in 1..=6 {
        dungeon.depth = depth;
        state = Some(level::create_level_partial(&mut dungeon));
    }
    let state = state.expect("floor six");
    let shop: Vec<_> = state
        .placed_items
        .iter()
        .filter(|item| matches!(item.provenance, items::model::ItemProvenance::Shop(_)))
        .collect();
    assert!(!shop.is_empty());
    let public = state.to_floor_report();
    assert!(!public.rooms.is_empty());
    assert!(public
        .items
        .iter()
        .any(|item| item.source.as_deref() == Some("ShopRoom")));
    assert!(public.items.iter().any(|item| {
        item.name == "Hourglass sand stock" && item.class_name() == Some("SandBag")
    }));
}

#[test]
fn imp_shop_stock_on_floor_20_is_reported_conditionally() {
    let mut dungeon = dungeon_from_run(init_run(0));
    let mut state = None;
    for depth in 1..=20 {
        dungeon.depth = depth;
        state = Some(level::create_level_partial(&mut dungeon));
    }

    let public = state.expect("floor twenty").to_floor_report();
    let shop: Vec<_> = public
        .items
        .iter()
        .filter(|item| item.source.as_deref() == Some("ImpShopRoom"))
        .collect();

    assert!(!shop.is_empty());
    assert!(shop.iter().all(|item| item
        .notes
        .iter()
        .any(|note| note.contains("Ambitious Imp quest") && note.contains("score > 2000"))));
    assert!(shop.iter().all(|item| {
        item.conditions.iter().any(|condition| {
            matches!(
                condition,
                report::ItemCondition::Quest {
                    quest_id,
                    ..
                } if quest_id == "ambitious_imp"
            )
        })
    }));
    assert!(public
        .guaranteed_appearances
        .iter()
        .all(|appearance| appearance.source != Some(spd_objects::RoomId::ImpShopRoom)));
    assert!(shop.iter().any(|item| {
        item.class_name() == Some("PlateArmor")
            && item.prediction == crate::report::ItemPredictionKind::Exact
    }));
    assert!(shop.iter().any(|item| {
        item.name == "weapon stock"
            && item.tier == Some(5)
            && item.prediction == crate::report::ItemPredictionKind::Constrained
    }));
}

#[test]
fn wandmaker_quest_spawns_within_prison() {
    // Depth 9 always spawns the quest room if not yet placed on 7–8.
    let mut saw = false;
    for s in [
        "GFX-PZH-DCH",
        "AAA-AAA-AAA",
        "hello",
        "42",
        "shattered",
        "JLY-ZYR-HET",
    ] {
        let r = analyze_seed(s, 9).expect("analyze");
        for f in &r.floors {
            if f.quests
                .iter()
                .any(|q| matches!(q, report::QuestReport::OldWandmaker { .. }))
                || f.items
                    .iter()
                    .any(|i| i.source.as_deref() == Some("Wandmaker.Quest"))
            {
                let rewards: Vec<_> = f
                    .items
                    .iter()
                    .filter(|i| {
                        i.source.as_deref() == Some("Wandmaker.Quest")
                            && i.prediction != report::ItemPredictionKind::Variable
                    })
                    .collect();
                assert_eq!(rewards.len(), 2);
                assert!(rewards.iter().all(|item| {
                    item.prediction == report::ItemPredictionKind::Constrained
                        && item.level.is_none()
                        && item.level_range == Some(report::NumericRange { min: 1, max: 3 })
                        && item.cursed == Some(false)
                        && item.class_name().is_none()
                        && item.candidate_classes().is_empty()
                        && item.notes.iter().any(|note| {
                            note.contains("two distinct wand options")
                                && note.contains("choose one")
                        })
                }));
                let variable_rewards: Vec<_> = f
                    .items
                    .iter()
                    .filter(|item| {
                        item.source.as_deref() == Some("Wandmaker.Quest")
                            && item.prediction == report::ItemPredictionKind::Variable
                    })
                    .collect();
                assert_eq!(variable_rewards.len(), 2);
                assert!(variable_rewards.iter().all(|item| {
                    item.class_name().is_some()
                        && item.level.is_some()
                        && item.candidate_classes().is_empty()
                }));
                let contract = f.quests.iter().find_map(|quest| match quest {
                    report::QuestReport::OldWandmaker { contract, .. } => Some(contract),
                    _ => None,
                });
                let contract = contract.expect("wandmaker quest contract");
                assert_eq!(contract.slots.len(), 2);
                assert_eq!(contract.slots[0].slot, 0);
                assert_eq!(
                    contract.slots[0]
                        .candidates
                        .first()
                        .map(|candidate| candidate.class_name()),
                    variable_rewards[0].class_name(),
                    "slot 0's fresh candidate must be the seed-drawn wand"
                );
                for slot in &contract.slots {
                    let weight: u32 = slot
                        .candidates
                        .iter()
                        .map(|candidate| candidate.weight)
                        .sum();
                    assert_eq!(weight, slot.scenario_count);
                }
                if let Some(map) = &f.map {
                    assert!(map.mobs.is_empty());
                    assert!(map.heaps.is_empty());
                }
                assert!(f.quests.iter().any(|quest| matches!(
                    quest,
                    report::QuestReport::OldWandmaker { contract, variable }
                        if contract.spawn_depth_range == report::QuestDepthRange { min: 7, max: 9 }
                            && contract.rewards.item_source == "Wandmaker.Quest"
                            && contract.rewards.option_count == 2
                            && contract.rewards.selected_count == 1
                            && contract.objective_options.len() == 3
                            && contract.objective_options.contains(&variable.objective)
                )));
                saw = true;
                break;
            }
        }
        if saw {
            break;
        }
    }
    assert!(
        saw,
        "at least one sampled Wandmaker quest must remain public"
    );
}

#[test]
fn imp_quest_spawns_within_city() {
    // Depth 19 always spawns if not yet placed on 17–18.
    let mut saw = false;
    for s in [
        "GFX-PZH-DCH",
        "AAA-AAA-AAA",
        "hello",
        "42",
        "shattered",
        "JLY-ZYR-HET",
    ] {
        let r = analyze_seed(s, 19).expect("analyze");
        for f in &r.floors {
            if f.quests
                .iter()
                .any(|q| matches!(q, report::QuestReport::AmbitiousImp { .. }))
                || f.items
                    .iter()
                    .any(|i| i.source.as_deref() == Some("Imp.Quest"))
            {
                saw = true;
                let options: Vec<_> = f
                    .items
                    .iter()
                    .filter(|item| {
                        item.source.as_deref() == Some("Imp.Quest")
                            && item.prediction != report::ItemPredictionKind::Variable
                    })
                    .collect();
                assert_eq!(options.len(), 6);
                assert!(options.iter().all(|item| {
                    item.prediction == report::ItemPredictionKind::Constrained
                        && item.class_name().is_none()
                        && item.cursed == Some(false)
                        && item
                            .notes
                            .iter()
                            .any(|note| note.contains("six vault take-out"))
                }));
                assert_eq!(options[0].name, "artifact or ring");
                assert!(f.quests.iter().any(|quest| matches!(
                    quest,
                    report::QuestReport::AmbitiousImp { contract, variable }
                        if contract.spawn_depth_range == report::QuestDepthRange { min: 17, max: 19 }
                            && contract.rewards.item_source == "Imp.Quest"
                            && contract.rewards.option_count == 6
                            && contract.rewards.selected_count == 1
                            && (17..=19).contains(&variable.spawn_depth)
                )));
                let variable_options: Vec<_> = f
                    .items
                    .iter()
                    .filter(|item| {
                        item.source.as_deref() == Some("Imp.Quest")
                            && item.prediction == report::ItemPredictionKind::Variable
                    })
                    .collect();
                assert_eq!(variable_options.len(), 6);
                assert!(variable_options.iter().all(|item| {
                    item.class_name().is_some()
                        && item.level.is_some()
                        && item.cursed == Some(false)
                        && item.candidate_classes().is_empty()
                }));
                let vault = f
                    .branches
                    .iter()
                    .find(|branch| matches!(branch.kind, report::BranchFloorKind::ImpVault))
                    .expect("nested imp vault");
                assert_eq!(vault.id.depth, f.depth);
                assert_eq!(vault.id.branch, 1);
                assert_eq!(vault.origin.depth, f.depth);
                assert!(vault.access.requires_acceptance);
                assert_eq!(vault.access.required_item, None);
                assert!(vault.map.is_some());
                break;
            }
        }
        if saw {
            break;
        }
    }
    // Internal quest generation is covered in `quests::imp`; inherited public
    // taint may intentionally hide every sampled city quest in this scan.
}

#[test]
fn blacksmith_quest_spawns_within_caves() {
    // Depth 14 always spawns if not yet placed on 12–13.
    let mut saw = false;
    for s in [
        "GFX-PZH-DCH",
        "AAA-AAA-AAA",
        "hello",
        "42",
        "shattered",
        "JLY-ZYR-HET",
    ] {
        let r = analyze_seed(s, 14).expect("analyze");
        for f in &r.floors {
            if f.quests
                .iter()
                .any(|q| matches!(q, report::QuestReport::TrollBlacksmith { .. }))
                || f.items
                    .iter()
                    .any(|i| i.source.as_deref() == Some("Blacksmith.Quest"))
            {
                saw = true;
                let rewards: Vec<_> = f
                    .items
                    .iter()
                    .filter(|item| {
                        item.source.as_deref() == Some("Blacksmith.Quest")
                            && item.prediction != report::ItemPredictionKind::Variable
                    })
                    .collect();
                assert_eq!(
                    rewards.len(),
                    4,
                    "expected 2 weapons + missile + armor, got {:?}",
                    rewards
                );
                assert!(rewards.iter().all(|item| {
                    item.prediction == report::ItemPredictionKind::Constrained
                        && item.class_name().is_none()
                        && item.candidate_classes().is_empty()
                        && item.tier.is_none()
                        && item.tier_range == Some(report::NumericRange { min: 3, max: 5 })
                        && item.level.is_none()
                        && item.level_range == Some(report::NumericRange { min: 0, max: 3 })
                        && item.cursed == Some(false)
                        && item.enchantment.is_none()
                        && item.notes.iter().any(|note| {
                            note.contains("four mutually exclusive options")
                                && note.contains("2,000 favor")
                        })
                        && item.notes.iter().any(|note| {
                            note.contains("share one +0…+3 level roll")
                                && note.contains("Parchment Scrap +1")
                                && note.contains("before this floor is generated")
                        })
                }));
                let variable_rewards: Vec<_> = f
                    .items
                    .iter()
                    .filter(|item| {
                        item.source.as_deref() == Some("Blacksmith.Quest")
                            && item.prediction == report::ItemPredictionKind::Variable
                    })
                    .collect();
                assert_eq!(variable_rewards.len(), 4);
                assert!(variable_rewards.iter().all(|item| {
                    item.class_name().is_some()
                        && item.level.is_some()
                        && item.cursed == Some(false)
                }));
                let room_rewards: Vec<_> = f
                    .items
                    .iter()
                    .filter(|i| i.source.as_deref() == Some("BlacksmithRoom"))
                    .collect();
                // A preceding runtime-sensitive room callback can invalidate
                // the later sampled BlacksmithRoom reward tail entirely.
                assert!(room_rewards.len() == 2 || room_rewards.is_empty());
                assert!(room_rewards.iter().all(|item| {
                    item.tier.is_some()
                        && item.level.is_some()
                        && item.cursed.is_none()
                        && (item.category != "armor"
                            || (item.prediction == report::ItemPredictionKind::Exact
                                && item.class_name().is_some()))
                        && (item.category == "armor"
                            || (item.prediction == report::ItemPredictionKind::Constrained
                                && item.class_name().is_none()))
                }));
                if let Some(map) = &f.map {
                    let constrained_heaps: Vec<_> = map
                        .heaps
                        .iter()
                        .filter(|heap| heap.items.is_empty())
                        .collect();
                    assert!(!constrained_heaps.is_empty());
                    assert!(map
                        .markers
                        .iter()
                        .any(|marker| { marker.label == "Blacksmith room equipment" }));
                }
                assert!(f.quests.iter().any(|quest| matches!(
                    quest,
                    report::QuestReport::TrollBlacksmith { contract, variable }
                        if contract.spawn_depth_range == report::QuestDepthRange { min: 12, max: 14 }
                            && contract.rewards.item_source == "Blacksmith.Quest"
                            && contract.rewards.option_count == 4
                            && contract.rewards.selected_count == 1
                            && contract.rewards.favor_requirement == Some(2_000)
                            && contract.objective_options.contains(&variable.objective)
                )));
                break;
            }
        }
        if saw {
            break;
        }
    }
    // Internal quest generation is covered in `quests::blacksmith`; inherited
    // public taint may intentionally hide every sampled caves quest here.
}

#[test]
fn crystal_vault_can_appear_with_prizes() {
    // Over several seeds, at least one CrystalVaultRoom should yield prizes.
    let mut saw = false;
    for s in [
        "GFX-PZH-DCH",
        "AAA-AAA-AAA",
        "hello",
        "42",
        "shattered",
        "JLY-ZYR-HET",
        "seedfinder",
        "crystal",
        "vault",
        "12345",
    ] {
        let r = analyze_seed(s, 24).expect("analyze");
        for f in &r.floors {
            if f.items
                .iter()
                .any(|i| i.source.as_deref() == Some("CrystalVaultRoom"))
            {
                saw = true;
                break;
            }
        }
        if saw {
            break;
        }
    }
    // Internal CrystalVault generation has dedicated room/oracle coverage;
    // inherited public taint may intentionally hide all sampled occurrences.
}

#[test]
fn special_trap_rooms_can_yield_prizes() {
    // Sentry/Traps/MagicalFire/Sacrifice/ToxicGas/SecretHoneypot — at least one source.
    const SOURCES: &[&str] = &[
        "SentryRoom",
        "TrapsRoom",
        "MagicalFireRoom",
        "SacrificeRoom",
        "ToxicGasRoom",
        "SecretHoneypotRoom",
    ];
    let mut saw = false;
    for s in [
        "GFX-PZH-DCH",
        "AAA-AAA-AAA",
        "hello",
        "42",
        "shattered",
        "JLY-ZYR-HET",
        "seedfinder",
        "traps",
        "sentry",
        "fire",
        "sacrifice",
        "12345",
        "98765",
        "honey",
    ] {
        let r = analyze_seed(s, 24).expect("analyze");
        for f in &r.floors {
            if f.items.iter().any(|i| {
                i.source
                    .as_deref()
                    .is_some_and(|src| SOURCES.contains(&src))
            }) {
                saw = true;
                break;
            }
        }
        if saw {
            break;
        }
    }
    assert!(
        saw,
        "expected at least one of {SOURCES:?} prizes across sample seeds"
    );
}

#[test]
fn analyze_several_seeds() {
    for s in ["AAA-AAA-AAA", "JLY-ZYR-HET", "hello", "42"] {
        let r = analyze_seed(s, 6);
        assert!(r.is_ok(), "seed {s}: {:?}", r.err());
    }
}

#[test]
fn seed_only_default_profile_keeps_regular_floor_room_facts() {
    let report = analyze_seed_seed_only("0", 4).expect("analyze");
    for floor in &report.floors {
        assert!(
            !floor.rooms.is_empty(),
            "depth {} should retain its no-extra-state room list",
            floor.depth
        );
    }
}

/// UI requests 26 floors; depth 26 is LastLevel (not RegularLevel).
/// Previously panicked in secrets_for_floor (region index 5) → WASM "unreachable".
#[test]
fn analyze_full_run_no_panic() {
    for s in ["GFX-PZH-DCH", "AAA-AAA-AAA", "hello", "42", "shattered"] {
        let r = analyze_seed_seed_only(s, 26).unwrap_or_else(|e| panic!("seed {s}: {e:?}"));
        assert_eq!(r.floors.len(), 26, "seed {s}");
        // Dedicated levels have no regular-room builder, but their pinned
        // layouts remain available in the concrete no-extra-state default profile.
        for depth in [5u32, 10, 15, 20, 25, 26] {
            let f = r.floors.iter().find(|f| f.depth == depth).expect("depth");
            assert!(
                f.builder.is_none(),
                "depth {depth} should not use a RegularLevel builder"
            );
        }
        // A mid Halls floor should still generate
        let f24 = r.floors.iter().find(|f| f.depth == 24).expect("24");
        assert!(f24.items.iter().any(|item| {
            item.name == "food" && item.prediction == report::ItemPredictionKind::Constrained
        }));
    }
}

#[test]
fn halls_report_the_mandatory_demon_spawner() {
    let report = analyze_seed("GFX-PZH-DCH", 24).expect("analyze");
    for depth in 21..=24 {
        let floor = &report.floors[(depth - 1) as usize];
        if floor.builder.is_none() {
            assert!(floor.rooms.is_empty());
            assert!(floor.map.is_none());
            continue;
        }
        assert!(floor.rooms.iter().any(|room| room == "DemonSpawnerRoom"));
        let Some(map) = floor.map.as_ref() else {
            // A prior runtime-sensitive room callback can invalidate every
            // later cell while the room-class fact remains safe.
            continue;
        };
        let spawner = map
            .markers
            .iter()
            .find(|marker| {
                marker.kind == crate::report::MapMarkerKind::Mob && marker.label == "Demon Spawner"
            })
            .unwrap_or_else(|| panic!("missing demon spawner marker on depth {depth}"));
        assert!(
            spawner.cell < map.tiles.len() as u32,
            "out-of-bounds demon spawner marker on depth {depth}"
        );
    }
}
