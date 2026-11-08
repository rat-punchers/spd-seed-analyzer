fn category_for(class_name: &str) -> &'static str {
    if class_name == "Food" || class_name == "Pasty" || class_name == "Meat" {
        "food"
    } else if class_name.starts_with("Ring") {
        "ring"
    } else if crate::generator::weapon_tier_for_class(class_name).is_some() {
        "weapon"
    } else if class_name == "Tomahawk"
        || class_name == "Dart"
        || class_name == "Boomerang"
        || class_name == "Javelin"
        || class_name == "HeavyBoomerang"
        || class_name == "Trident"
        || class_name == "ThrowingHammer"
        || class_name == "ForceCube"
    {
        "missile"
    } else if class_name.ends_with("Armor") {
        "armor"
    } else if class_name.starts_with("StoneOf") {
        "stone"
    } else if class_name.starts_with("Potion") {
        "potion"
    } else if class_name.starts_with("Scroll") {
        "scroll"
    } else if class_name.starts_with("Wand") {
        "wand"
    } else if [
        "AlchemistsToolkit",
        "ChaliceOfBlood",
        "CloakOfShadows",
        "DriedRose",
        "EtherealChains",
        "HolyTome",
        "HornOfPlenty",
        "MasterThievesArmband",
        "SandalsOfNature",
        "SkeletonKey",
        "TalismanOfForesight",
        "TimekeepersHourglass",
        "UnstableSpellbook",
    ]
    .contains(&class_name)
    {
        "artifact"
    } else if class_name.ends_with("Seed") {
        "seed"
    } else {
        "other"
    }
}
use super::*;

mod matching;

#[test]
fn depth_four_search_completes_with_minimum_size_secret_larder() {
    let result = search_seeds(&SeedSearchRequest {
        start_seed: 2_718_251_756_419,
        candidate_count: 10,
        floors: 4,
        constraints: vec![ItemConstraint {
            item_group: "ring".into(),
            item: Some(spd_objects::ItemId::RingOfWealth),
            min_level: None,
            min_depth: 1,
            max_depth: 4,
        }],
        match_mode: MatchMode::Any,
        include_variable: false,
        deep_search: false,
        max_matches: 10,
    })
    .expect("the bounded search should complete");

    assert_eq!(result.candidates_scanned, 10);
    assert_eq!(result.next_seed, Some(2_718_251_756_429));
}

fn constraint(class_name: &str, min_depth: u32, max_depth: u32) -> ItemConstraint {
    ItemConstraint {
        item_group: category_for(class_name).into(),
        item: Some(spd_objects::ItemId::from_class_name_lossy(class_name)),
        min_level: None,
        min_depth,
        max_depth,
    }
}

#[test]
fn generation_floors_uses_the_deepest_constraint() {
    let mut value = request(
        vec![
            constraint("RingOfWealth", 1, 5),
            constraint("WandOfLightning", 1, 17),
        ],
        MatchMode::All,
    );
    value.floors = 4;
    assert_eq!(value.generation_floors(), 17);
    assert!(value.validate().is_ok());
}

fn request(constraints: Vec<ItemConstraint>, match_mode: MatchMode) -> SeedSearchRequest {
    SeedSearchRequest {
        start_seed: 0,
        candidate_count: 2,
        floors: 1,
        constraints,
        match_mode,
        include_variable: false,
        deep_search: false,
        max_matches: 10,
    }
}

fn exact_floor(depth: u32, classes: &[(&str, i32)]) -> crate::FloorReport {
    crate::FloorReport {
        depth,
        feeling: None,
        builder: None,
        rooms: vec![],
        possible_rooms: vec![],
        guaranteed_appearances: vec![],
        initial_encounters: vec![],
        items: classes
            .iter()
            .map(|(class_name, level)| {
                crate::report::ItemEntry {
                    name: (*class_name).into(),
                    quantity: 1,
                    quantity_range: None,
                    label: None,
                    item: Some(spd_objects::ItemId::from_class_name_lossy(class_name)),
                    candidates: Vec::new(),
                    category: category_for(class_name).into(),
                    tier: None,
                    tier_range: None,
                    level: Some(*level),
                    level_range: None,
                    cursed: Some(false),
                    enchantment: None,
                    prediction: ItemPredictionKind::Exact,
                    spawn_conditions: Vec::new(),
                    conditions: Vec::new(),
                    notes: vec![],
                    source: Some("test".into()),
                }
                .into()
            })
            .collect(),
        quests: vec![],
        map: None,
        assumed_map: None,
        branches: vec![],
    }
}

#[test]
fn validation_rejects_unbounded_and_malformed_requests() {
    let mut value = request(vec![constraint("Food", 1, 1)], MatchMode::Any);
    value.start_seed = TOTAL_SEEDS;
    assert!(matches!(
        value.validate(),
        Err(SearchError::StartSeedOutOfRange)
    ));

    value.start_seed = 0;
    value.candidate_count = MAX_SEARCH_CANDIDATES + 1;
    assert!(matches!(
        value.validate(),
        Err(SearchError::CandidateCountOutOfRange)
    ));

    value.candidate_count = 1;
    value.constraints.clear();
    assert!(matches!(
        value.validate(),
        Err(SearchError::EmptyConstraints)
    ));

    value.constraints = vec![ItemConstraint {
        item_group: " \t ".into(),
        item: None,
        min_level: None,
        min_depth: 1,
        max_depth: 1,
    }];
    assert!(matches!(
        value.validate(),
        Err(SearchError::EmptyItemGroup { index: 0 })
    ));

    value.constraints = vec![constraint("Food", 2, 1)];
    assert!(matches!(
        value.validate(),
        Err(SearchError::InvalidDepthRange { index: 0 })
    ));

    value.constraints = vec![constraint("Food", 1, MAX_SEARCH_FLOORS + 1)];
    assert!(matches!(
        value.validate(),
        Err(SearchError::InvalidDepthRange { index: 0 })
    ));

    value.constraints = vec![constraint("Food", 1, 1)];
    value.constraints[0].min_level = Some(5);
    assert!(matches!(
        value.validate(),
        Err(SearchError::InvalidLevel { index: 0 })
    ));

    value.constraints = vec![constraint("Food", 1, 1)];
    value.max_matches = 0;
    assert!(matches!(
        value.validate(),
        Err(SearchError::MaxMatchesOutOfRange)
    ));
}

#[test]
fn variable_search_defaults_to_included_when_omitted() {
    let request: SeedSearchRequest = serde_json::from_value(serde_json::json!({
        "startSeed": 0,
        "candidateCount": 1,
        "floors": 1,
        "constraints": [{
            "itemGroup": "food",
            "item": spd_objects::ItemId::Food.id(),
            "minDepth": 1,
            "maxDepth": 1
        }],
        "matchMode": "any",
        "maxMatches": 1
    }))
    .expect("request without includeVariable");

    assert!(request.include_variable);
}

#[test]
fn any_and_all_modes_use_each_constraint_independently() {
    let floor = exact_floor(3, &[("Honeypot", 0), ("ShatteredPot", 0)]);
    let any = matching_evidence(
        std::slice::from_ref(&floor),
        &[constraint("Honeypot", 3, 3), constraint("Missing", 3, 3)],
        false,
    );
    assert_eq!(any.len(), 1);
    let all = matching_evidence(
        &[floor],
        &[
            constraint("ShatteredPot", 3, 3),
            constraint("Honeypot", 3, 3),
        ],
        false,
    );
    assert_eq!(all.len(), 2);
}

#[test]
fn depth_ranges_are_inclusive_and_scoped() {
    let floor = exact_floor(3, &[("Honeypot", 0)]);
    assert_eq!(
        matching_evidence(
            std::slice::from_ref(&floor),
            &[constraint("Honeypot", 3, 3)],
            false,
        )[0]
        .depth,
        3
    );
    assert!(matching_evidence(&[floor], &[constraint("Honeypot", 2, 2)], false).is_empty());
}

#[test]
fn minimum_upgrade_levels_are_optional_and_inclusive() {
    let floor = exact_floor(3, &[("Sword", 2)]);
    assert_eq!(
        matching_evidence(
            std::slice::from_ref(&floor),
            &[constraint("Sword", 3, 3)],
            false
        )
        .len(),
        1
    );
    let mut upgraded = constraint("Sword", 3, 3);
    upgraded.min_level = Some(3);
    assert!(matching_evidence(&[floor], &[upgraded], false).is_empty());
}

#[test]
fn constrained_runtime_sensitive_items_never_match_exact_searches() {
    let floor = crate::FloorReport {
        depth: 13,
        feeling: None,
        builder: None,
        rooms: vec![spd_objects::RoomId::SacrificeRoom],
        possible_rooms: vec![],
        guaranteed_appearances: vec![],
        initial_encounters: vec![],
        items: vec![crate::report::ItemEntry {
            name: "weapon reward".into(),
            quantity: 1,
            quantity_range: None,
            label: None,
            item: None,
            candidates: Vec::new(),
            category: "weapon".into(),
            tier: Some(3),
            tier_range: None,
            level: None,
            level_range: None,
            cursed: Some(true),
            enchantment: None,
            prediction: crate::report::ItemPredictionKind::Constrained,
            spawn_conditions: Vec::new(),
            conditions: Vec::new(),
            notes: vec!["Parchment Scrap may alter enchantment chance.".into()],
            source: Some("SacrificeRoom".into()),
        }
        .into()],
        quests: Vec::new(),
        map: None,
        assumed_map: None,
        branches: vec![],
    };
    let constraints = [ItemConstraint {
        item_group: "weapon".into(),
        item: Some(spd_objects::ItemId::Sword),
        min_level: None,
        min_depth: 13,
        max_depth: 13,
    }];
    assert!(matching_evidence(&[floor], &constraints, false).is_empty());
}

#[test]
fn category_only_imp_ring_never_matches_an_exact_ring_search() {
    let mut floor = exact_floor(18, &[]);
    floor.items.push(
        crate::report::ItemEntry {
            name: "ring reward".into(),
            quantity: 1,
            quantity_range: None,
            label: None,
            item: None,
            candidates: Vec::new(),
            category: "ring".into(),
            tier: None,
            tier_range: None,
            level: None,
            level_range: Some(crate::report::NumericRange { min: 2, max: 4 }),
            cursed: Some(false),
            enchantment: None,
            prediction: ItemPredictionKind::Constrained,
            spawn_conditions: Vec::new(),
            conditions: Vec::new(),
            notes: vec!["Quest completion is required.".into()],
            source: Some("Imp.Quest".into()),
        }
        .into(),
    );
    assert!(matching_evidence(
        &[floor.clone()],
        &[constraint("RingOfHaste", 18, 18)],
        false
    )
    .is_empty());
    assert!(matching_evidence(&[floor], &[constraint("RingOfWealth", 18, 18)], false).is_empty());
}

#[test]
fn conditional_floor_loot_candidates_keep_levels_for_search() {
    let mut floor = exact_floor(8, &[]);
    floor.items.push(
        crate::report::ItemEntry {
            name: "long sword +2".into(),
            quantity: 1,
            quantity_range: None,
            label: None,
            item: None,
            candidates: vec![spd_objects::ItemId::Longsword],
            category: "weapon".into(),
            tier: None,
            tier_range: None,
            level: Some(2),
            level_range: None,
            cursed: Some(false),
            enchantment: None,
            prediction: ItemPredictionKind::Constrained,
            spawn_conditions: Vec::new(),
            conditions: Vec::new(),
            notes: vec!["Assumes no external artifact acquisition".into()],
            source: Some("heap".into()),
        }
        .into(),
    );
    let mut wanted = constraint("Longsword", 8, 8);
    wanted.min_level = Some(2);

    let evidence = matching_evidence(&[floor], &[wanted], false);
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].level, 2);
}

#[test]
fn later_identity_unknown_food_spawns_never_match_exact_item_searches() {
    for class_name in ["Food", "Pasty"] {
        let result = search_seeds(&SeedSearchRequest {
            start_seed: 0,
            candidate_count: 4,
            floors: 2,
            constraints: vec![constraint(class_name, 2, 2)],
            match_mode: MatchMode::Any,
            include_variable: false,
            deep_search: false,
            max_matches: 4,
        })
        .expect("forced queue search");
        assert!(
            result.matches.is_empty(),
            "{class_name} identity constraint leaked into exact search"
        );
    }
}

#[test]
fn floor_one_exact_food_identity_matches_exact_item_search() {
    let report = crate::analyze_seed("0", 1).expect("floor-one report");
    let class_name = report.floors[0]
        .items
        .iter()
        .find(|item| item.category == "food")
        .and_then(|item| item.class_name().map(str::to_string))
        .expect("exact floor-one food class");
    let result = search_seeds(&SeedSearchRequest {
        start_seed: 0,
        candidate_count: 1,
        floors: 1,
        constraints: vec![constraint(&class_name, 1, 1)],
        match_mode: MatchMode::Any,
        include_variable: false,
        deep_search: false,
        max_matches: 1,
    })
    .expect("exact floor-one food search");
    assert_eq!(result.matches.len(), 1);
}

#[test]
fn finder_variable_highlights_exclude_promoted_floor_one_rewards() {
    let numeric = crate::parse_seed("RZN-LKU-EFS")
        .expect("fixture seed")
        .numeric;
    let result = search_seeds(&SeedSearchRequest {
        start_seed: numeric,
        candidate_count: 1,
        floors: 17,
        constraints: vec![constraint("Food", 1, 1)],
        match_mode: MatchMode::Any,
        include_variable: false,
        deep_search: false,
        max_matches: 1,
    })
    .expect("variable-highlight finder result");

    let matched = &result.matches[0];
    assert_eq!(matched.evidence.len(), 1);
    assert_eq!(matched.evidence[0].class_name(), Some("Food"));
    assert!(matched
        .variable_items
        .iter()
        .all(|item| { item.depth != 1 || item.item.class_name() != Some("RingOfWealth") }));
    assert!(matched.variable_items.iter().all(|item| {
        item.item.prediction == ItemPredictionKind::Variable
            && item.item.class_name() != Some("Food")
    }));
}

#[test]
fn sacrifice_sickle_requires_variable_opt_in_and_is_labeled() {
    // Pinned to a fresh-journal Java oracle run.
    let _journal = crate::journal::scope_lore_complete(false);
    let numeric = crate::parse_seed("PUB-CLI-VNW")
        .expect("fixture seed")
        .numeric;
    let mut wanted = constraint("Mace", 4, 4);
    wanted.min_level = Some(1);
    let mut request = SeedSearchRequest {
        start_seed: numeric,
        candidate_count: 1,
        floors: 4,
        constraints: vec![wanted],
        match_mode: MatchMode::All,
        include_variable: false,
        deep_search: false,
        max_matches: 1,
    };

    let guaranteed_only = search_seeds(&request).expect("guaranteed-only search");
    assert!(guaranteed_only.matches.is_empty());

    request.include_variable = true;
    let with_variable = search_seeds(&request).expect("variable-inclusive search");
    let evidence = &with_variable.matches[0].evidence[0];
    assert_eq!(evidence.class_name(), Some("Mace"));
    assert_eq!(evidence.depth, 4);
    assert_eq!(evidence.level, 1);
    assert_eq!(evidence.prediction, ItemPredictionKind::Variable);
    assert_eq!(evidence.source.as_deref(), Some("SacrificeRoom"));
}

#[test]
fn every_quest_exposes_searchable_rewards_including_variable_items() {
    let report = crate::analyze_seed_seed_only("0", 20).expect("seed-zero quest report");
    let quest_sources = [
        "Ghost.Quest",
        "Wandmaker.Quest",
        "Blacksmith.Quest",
        "Imp.Quest",
    ];
    let searchable_rewards = report
        .floors
        .iter()
        .flat_map(|floor| {
            floor
                .items
                .iter()
                .flat_map(|group| &group.variants)
                .filter(|item| {
                    item.source
                        .as_deref()
                        .is_some_and(|source| quest_sources.contains(&source))
                })
                .flat_map(|item| {
                    let concrete = match item.prediction {
                        ItemPredictionKind::Exact | ItemPredictionKind::Variable => {
                            item.class_name().into_iter().collect::<Vec<_>>()
                        }
                        ItemPredictionKind::Constrained => item.candidate_classes(),
                    };
                    concrete.into_iter().map(|class_name| {
                        (
                            class_name,
                            item.category.clone(),
                            floor.depth,
                            item.source.clone().expect("quest reward source"),
                        )
                    })
                })
        })
        .collect::<Vec<_>>();

    // A quest class that also exists as ordinary floor loot at the same depth
    // (e.g. BlacksmithRoom ScaleArmor) can steal All-mode evidence. Keep classes
    // that uniquely identify the quest reward on this seed.
    let searchable_rewards = searchable_rewards
        .into_iter()
        .filter(|(class_name, _, depth, _)| {
            report
                .floors
                .iter()
                .filter(|floor| floor.depth == *depth)
                .flat_map(crate::FloorReport::item_variants)
                .filter(|item| {
                    item.class_name() == Some(*class_name)
                        || item
                            .candidate_classes()
                            .iter()
                            .any(|candidate| candidate == class_name)
                })
                .all(|item| {
                    item.source
                        .as_deref()
                        .is_some_and(|source| quest_sources.contains(&source))
                })
        })
        .collect::<Vec<_>>();

    for source in quest_sources {
        assert!(
            searchable_rewards
                .iter()
                .any(|(_, _, _, reward_source)| reward_source == source),
            "missing searchable rewards from {source}"
        );
    }
    assert!(report.floors.iter().any(|floor| {
        floor
            .items
            .iter()
            .flat_map(|group| &group.variants)
            .any(|item| {
                item.prediction == ItemPredictionKind::Variable
                    && item
                        .source
                        .as_deref()
                        .is_some_and(|source| quest_sources.contains(&source))
            })
    }));

    let result = search_seeds(&SeedSearchRequest {
        start_seed: 0,
        candidate_count: 1,
        floors: 20,
        constraints: searchable_rewards
            .iter()
            .map(|(class_name, category, depth, _)| ItemConstraint {
                item_group: category.clone(),
                item: Some(spd_objects::ItemId::from_class_name_lossy(class_name)),
                min_level: None,
                min_depth: *depth,
                max_depth: *depth,
            })
            .collect(),
        match_mode: MatchMode::All,
        include_variable: true,
        deep_search: false,
        max_matches: 1,
    })
    .expect("quest reward search");

    assert_eq!(result.matches.len(), 1);
    let evidence = &result.matches[0].evidence;
    assert_eq!(evidence.len(), searchable_rewards.len());
    assert!(evidence.iter().all(|item| item
        .source
        .as_deref()
        .is_some_and(|source| quest_sources.contains(&source))));
}

#[test]
fn search_generates_floors_for_the_deepest_constraint() {
    let result = search_seeds(&SeedSearchRequest {
        start_seed: 0,
        candidate_count: 64,
        floors: 1,
        constraints: vec![constraint("PotionOfStrength", 1, 4)],
        match_mode: MatchMode::Any,
        include_variable: false,
        deep_search: false,
        max_matches: 1,
    })
    .expect("constraint depth should extend generation");
    assert!(
        !result.matches.is_empty(),
        "missing PotionOfStrength spawn match when floors is shallower than the constraint"
    );
}

#[test]
fn guaranteed_limited_drop_spawns_match_exact_item_searches() {
    for class_name in [
        "PotionOfStrength",
        "ScrollOfUpgrade",
        "Stylus",
        "StoneOfIntuition",
        "TrinketCatalyst",
    ] {
        let result = search_seeds(&SeedSearchRequest {
            start_seed: 0,
            candidate_count: 64,
            floors: 4,
            constraints: vec![constraint(class_name, 1, 4)],
            match_mode: MatchMode::Any,
            include_variable: false,
            deep_search: false,
            max_matches: 1,
        })
        .expect("guaranteed limited drop search");
        assert!(
            !result.matches.is_empty(),
            "missing {class_name} spawn match"
        );
    }

    let torches = search_seeds(&SeedSearchRequest {
        start_seed: 0,
        candidate_count: 1,
        floors: 24,
        constraints: vec![constraint("Torch", 21, 24)],
        match_mode: MatchMode::Any,
        include_variable: false,
        deep_search: false,
        max_matches: 1,
    })
    .expect("guaranteed Torch search");
    assert_eq!(torches.matches.len(), 1);
}

#[test]
fn puzzle_solution_potions_match_exact_item_searches() {
    for (seed, class_name, room_source, evidence_source) in [
        (25, "PotionOfHaste", "SentryRoom", "SentryRoom"),
        (53, "PotionOfPurity", "ToxicGasRoom", "RingRoom"),
        (99_162_322, "PotionOfLevitation", "TrapsRoom", "TrapsRoom"),
        (
            8_687_205_886,
            "PotionOfFrost",
            "MagicalFireRoom",
            "MagicalFireRoom",
        ),
    ] {
        let report = crate::analyze_seed(&seed.to_string(), 1).expect("puzzle solution report");
        assert!(report.floors[0].items.iter().any(|item| {
            item.class_name() == Some(class_name) && item.source.as_deref() == Some(room_source)
        }));

        let result = search_seeds(&SeedSearchRequest {
            start_seed: seed,
            candidate_count: 1,
            floors: 1,
            constraints: vec![constraint(class_name, 1, 1)],
            match_mode: MatchMode::Any,
            include_variable: false,
            deep_search: false,
            max_matches: 1,
        })
        .expect("puzzle solution search");

        let evidence = &result.matches[0].evidence[0];
        assert_eq!(evidence.class_name(), Some(class_name));
        assert_eq!(evidence.depth, 1);
        assert_eq!(evidence.level, 0);
        // Seed 53 also relocates another guaranteed Purity potion into a
        // RingRoom, which is the first exact copy selected as search evidence.
        assert_eq!(evidence.source.as_deref(), Some(evidence_source));
    }
}

#[test]
fn secret_honeypot_fixed_items_match_exact_search_evidence() {
    let floor = exact_floor(12, &[("ShatteredPot", 0), ("Honeypot", 0)]);
    let constraints = [
        constraint("ShatteredPot", 12, 12),
        constraint("Honeypot", 12, 12),
        constraint("Bomb", 12, 12),
    ];

    let evidence = matching_evidence(&[floor], &constraints, false);
    assert_eq!(evidence.len(), 2);
    assert_eq!(evidence[0].class_name(), Some("ShatteredPot"));
    assert_eq!(evidence[1].class_name(), Some("Honeypot"));
}

#[test]
fn constrained_shop_stock_never_matches_its_internal_concrete_class() {
    let floor = crate::FloorReport {
        depth: 6,
        feeling: None,
        builder: None,
        rooms: vec![spd_objects::RoomId::ShopRoom],
        possible_rooms: vec![],
        guaranteed_appearances: vec![],
        initial_encounters: vec![],
        items: vec![crate::report::ItemEntry {
            name: "weapon stock".into(),
            quantity: 1,
            quantity_range: None,
            label: None,
            item: None,
            candidates: Vec::new(),
            category: "weapon".into(),
            tier: Some(2),
            tier_range: None,
            level: Some(0),
            level_range: None,
            cursed: Some(false),
            enchantment: None,
            prediction: crate::report::ItemPredictionKind::Constrained,
            spawn_conditions: Vec::new(),
            conditions: Vec::new(),
            notes: vec![],
            source: Some("ShopRoom".into()),
        }
        .into()],
        quests: vec![],
        map: None,
        assumed_map: None,
        branches: vec![],
    };
    let constraints = [ItemConstraint {
        item_group: "weapon".into(),
        item: Some(spd_objects::ItemId::Quarterstaff),
        min_level: None,
        min_depth: 6,
        max_depth: 6,
    }];
    assert!(matching_evidence(&[floor], &constraints, false).is_empty());
}

#[test]
fn real_constrained_quest_class_never_matches_exact_search() {
    use crate::items::model::{ItemProvenance, QuestRewardRole};

    let mut dungeon = crate::run::dungeon_from_run(crate::run::init_run(0));
    for depth in 1..=19 {
        dungeon.depth = depth;
        let state = crate::level::create_level_partial(&mut dungeon);
        let Some(internal) = state.placed_items.iter().find(|item| {
            matches!(
                item.provenance,
                ItemProvenance::Quest(
                    QuestRewardRole::WandmakerWand
                        | QuestRewardRole::BlacksmithRoomWeapon { .. }
                        | QuestRewardRole::BlacksmithRoomMissile { .. }
                )
            )
        }) else {
            continue;
        };
        let constraints = [ItemConstraint {
            item_group: format!("{:?}", internal.category).to_ascii_lowercase(),
            item: Some(spd_objects::ItemId::from_class_name_lossy(
                &internal.class_name,
            )),
            min_level: None,
            min_depth: depth as u32,
            max_depth: depth as u32,
        }];
        assert!(matching_evidence(&[state.to_floor_report()], &constraints, false).is_empty());
        return;
    }
    panic!("expected a constrained quest reward");
}

#[test]
fn bounded_search_preserves_ascending_resume_position_without_matches() {
    let mut value = request(vec![constraint("Amulet", 1, 1)], MatchMode::Any);
    value.max_matches = 1;

    let result = search_seeds(&value).expect("bounded search");
    assert!(result.matches.is_empty());
    assert_eq!(result.candidates_scanned, 2);
    assert_eq!(result.next_seed, Some(2));
    assert!(!result.match_limit_reached);
    assert!(!result.exhausted);
    assert_eq!(result.status, "partial");
}

#[test]
fn search_does_not_wrap_at_total_seeds() {
    let value = SeedSearchRequest {
        start_seed: TOTAL_SEEDS - 1,
        candidate_count: 2,
        floors: 1,
        constraints: vec![constraint("Amulet", 1, 1)],
        match_mode: MatchMode::Any,
        include_variable: false,
        deep_search: false,
        max_matches: 1,
    };

    let result = search_seeds(&value).expect("last seed search");
    assert_eq!(result.candidates_scanned, 1);
    assert!(result.matches.is_empty());
    assert_eq!(result.next_seed, None);
    assert!(result.exhausted);
    assert!(!result.match_limit_reached);
}

#[test]
fn any_item_in_category_matches_exact_and_upgraded_items() {
    let mut floor = exact_floor(2, &[]);
    floor.items.push(
        crate::report::ItemEntry {
            name: "Ring of Wealth +1".into(),
            quantity: 1,
            quantity_range: None,
            label: None,
            item: Some(spd_objects::ItemId::RingOfWealth),
            candidates: Vec::new(),
            category: "ring".into(),
            tier: None,
            tier_range: None,
            level: Some(1),
            level_range: None,
            cursed: Some(false),
            enchantment: None,
            prediction: ItemPredictionKind::Exact,
            spawn_conditions: Vec::new(),
            conditions: Vec::new(),
            notes: Vec::new(),
            source: Some("CrystalVault".into()),
        }
        .into(),
    );

    // Matches any ring
    let any_ring = ItemConstraint {
        item_group: "ring".into(),
        item: None,
        min_level: None,
        min_depth: 1,
        max_depth: 5,
    };
    let evidence = matching_evidence(&[floor.clone()], &[any_ring], false);
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].class_name(), Some("RingOfWealth"));
    assert_eq!(evidence[0].level, 1);

    // Matches any ring with min_level <= 1
    let upgraded_ring = ItemConstraint {
        item_group: "ring".into(),
        item: None,
        min_level: Some(1),
        min_depth: 1,
        max_depth: 5,
    };
    let evidence = matching_evidence(&[floor.clone()], &[upgraded_ring], false);
    assert_eq!(evidence.len(), 1);

    // Does not match when min_level > 1
    let higher_level_ring = ItemConstraint {
        item_group: "ring".into(),
        item: None,
        min_level: Some(2),
        min_depth: 1,
        max_depth: 5,
    };
    assert!(matching_evidence(&[floor.clone()], &[higher_level_ring], false).is_empty());

    // Does not match another category
    let any_wand = ItemConstraint {
        item_group: "wand".into(),
        item: None,
        min_level: None,
        min_depth: 1,
        max_depth: 5,
    };
    assert!(matching_evidence(&[floor], &[any_wand], false).is_empty());
}

/// The seed-only Imp entries carry no concrete class, so without widening they
/// can never satisfy a class constraint. Deep search fills them from the quest
/// contract's drift window, which always contains the fresh-run draw.
#[test]
fn deep_search_widens_seed_only_imp_entries() {
    let mut report = crate::analyze_seed_seed_only("0", 19).expect("analyze seed 0");

    let imp_constrained = |report: &crate::SeedReport| -> Vec<Vec<String>> {
        report
            .floors
            .iter()
            .flat_map(|floor| &floor.items)
            .filter(|group| group.source.as_deref() == Some("Imp.Quest"))
            .flat_map(|group| &group.variants)
            .filter(|item| item.prediction == crate::report::ItemPredictionKind::Constrained)
            .map(|item| {
                item.candidate_classes()
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    };

    let before = imp_constrained(&report);
    assert_eq!(before.len(), 6, "six seed-only Imp slots");
    assert!(
        before.iter().all(|classes| classes.is_empty()),
        "seed-only Imp entries start with no candidates: {before:?}"
    );

    super::quest_candidates::widen_quest_candidates(&mut report.floors);
    let after = imp_constrained(&report);
    assert_eq!(after.len(), 6);
    assert!(
        after.iter().all(|classes| !classes.is_empty()),
        "every slot gains candidates: {after:?}"
    );
    // The plate slot has no deck at all.
    assert_eq!(after[4], vec!["PlateArmor".to_string()]);

    // Each slot's window must contain the fresh-run draw for that slot.
    let seed_drawn: Vec<String> = report
        .floors
        .iter()
        .flat_map(|floor| &floor.items)
        .filter(|group| group.source.as_deref() == Some("Imp.Quest"))
        .flat_map(|group| &group.variants)
        .filter(|item| item.prediction == crate::report::ItemPredictionKind::Variable)
        .filter_map(|item| item.class_name().map(str::to_string))
        .collect();
    assert_eq!(seed_drawn.len(), 6);
    for (slot, (classes, concrete)) in after.iter().zip(&seed_drawn).enumerate() {
        assert!(
            classes.contains(concrete),
            "slot {slot}: window {classes:?} must contain the fresh-run draw {concrete}"
        );
    }
}

/// Deep search copies each Wandmaker slot's window onto the seed-only rows,
/// in slot order, and leaves the variable rows empty.
#[test]
fn deep_search_widens_seed_only_wandmaker_entries() {
    let mut report = crate::analyze_seed_seed_only("0", 9).expect("analyze seed 0");

    let wandmaker_classes = |report: &crate::SeedReport, prediction| -> Vec<Vec<String>> {
        report
            .floors
            .iter()
            .flat_map(|floor| &floor.items)
            .filter(|group| group.source.as_deref() == Some("Wandmaker.Quest"))
            .flat_map(|group| &group.variants)
            .filter(|item| item.prediction == prediction)
            .map(|item| {
                item.candidate_classes()
                    .into_iter()
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    };

    let constrained = crate::report::ItemPredictionKind::Constrained;
    let variable = crate::report::ItemPredictionKind::Variable;
    let before = wandmaker_classes(&report, constrained);
    assert_eq!(before.len(), 2, "two seed-only wand rows");
    assert!(
        before.iter().all(|classes| classes.is_empty()),
        "constrained wand rows start empty: {before:?}"
    );

    super::quest_candidates::widen_quest_candidates(&mut report.floors);
    let after = wandmaker_classes(&report, constrained);
    let slots: Vec<_> = report
        .floors
        .iter()
        .find_map(|floor| {
            floor.quests.iter().find_map(|quest| match quest {
                crate::report::QuestReport::OldWandmaker { contract, .. } => {
                    Some(contract.slots.clone())
                }
                _ => None,
            })
        })
        .expect("wandmaker slots");
    assert_eq!(slots.len(), 2);
    assert_eq!(after.len(), 2);
    for (slot, classes) in slots.iter().zip(&after) {
        let expected: Vec<_> = slot
            .candidates
            .iter()
            .map(|candidate| candidate.class_name().to_string())
            .collect();
        assert_eq!(classes, &expected, "slot {}", slot.slot);
        assert!(
            classes.contains(&slot.item.class_name().to_string()),
            "slot {} window {classes:?} must contain {}",
            slot.slot,
            slot.item
        );
    }

    let variable_rows = wandmaker_classes(&report, variable);
    assert_eq!(variable_rows.len(), 2);
    assert!(
        variable_rows.iter().all(|classes| classes.is_empty()),
        "variable wand rows stay empty: {variable_rows:?}"
    );
}
