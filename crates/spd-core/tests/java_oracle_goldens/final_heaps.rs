use super::*;

#[path = "final_heaps/floor_eighteen.rs"]
mod floor_eighteen;
#[path = "final_heaps/floor_nineteen.rs"]
mod floor_nineteen;
#[path = "final_heaps/floor_seventeen.rs"]
mod floor_seventeen;
#[path = "final_heaps/floor_sixteen.rs"]
mod floor_sixteen;
#[path = "final_heaps/floor_three.rs"]
mod floor_three;
#[path = "final_heaps/floor_twenty_four.rs"]
mod floor_twenty_four;
#[path = "final_heaps/floor_twenty_one.rs"]
mod floor_twenty_one;
#[path = "final_heaps/floor_twenty_six.rs"]
mod floor_twenty_six;
#[path = "final_heaps/floor_twenty_three.rs"]
mod floor_twenty_three;
#[path = "final_heaps/floor_twenty_two.rs"]
mod floor_twenty_two;
#[path = "final_heaps/replay_aaa.rs"]
mod replay_aaa;
#[path = "final_heaps/replay_aaa_caves_visual.rs"]
mod replay_aaa_caves_visual;
#[path = "final_heaps/replay_aaa_sewers.rs"]
mod replay_aaa_sewers;

fn report_blacklists(class_name: &str) -> bool {
    matches!(
        class_name,
        "Gold"
            | "Dewdrop"
            | "IronKey"
            | "GoldenKey"
            | "CrystalKey"
            | "EnergyCrystal"
            | "CorpseDust"
            | "Embers"
            | "CeremonialCandle"
            | "Pickaxe"
    )
}

fn signatures(items: &[ComparableItem]) -> Vec<String> {
    items
        .iter()
        .map(|item| {
            if item.cursed {
                format!("cursed {}", item.class_name)
            } else {
                item.class_name.clone()
            }
        })
        .collect()
}

fn normalize_oracle_class(class_name: &str) -> &str {
    if class_name.ends_with("Seed") {
        "Seed"
    } else {
        class_name
    }
}

fn assert_main_loop_heaps_match(
    actual: &spd_core::level::LevelState,
    expected: &OracleFloor,
    context: &impl std::fmt::Display,
) -> usize {
    let map = actual.map.as_ref().expect("regular floor map");
    let mut compared = 0;
    let mut expected_items: Vec<_> = expected
        .final_heaps
        .iter()
        .flat_map(|heap| heap.items.iter())
        .collect();

    for heap in &map.heaps {
        for item in &heap.items {
            if item.source.as_deref() != Some("heap") {
                continue;
            }
            compared += 1;
            let expected_index = expected_items
                .iter()
                .position(|expected_item| {
                    expected_item.class_name == normalize_oracle_class(item.class_name())
                        && expected_item.quantity == item.quantity
                        && expected_item.level == item.level
                        && expected_item.cursed == item.cursed
                })
                .unwrap_or_else(|| {
                    panic!(
                        "main-loop item has no Java final-heap match in {context}: item={item:?}"
                    )
                });
            expected_items.remove(expected_index);
        }
    }

    compared
}

#[test]
fn every_covered_main_loop_spawn_matches_java_final_heaps() {
    let _journal = crate::fresh_journal();
    let mut compared_floors = 0;
    let mut compared_items = 0;

    for path in fixture_paths() {
        let fixture = read_fixture(&path);
        if !matches!(
            fixture.schema_version,
            FINAL_HEAPS_SCHEMA_VERSION | EXTENDED_FINAL_HEAPS_SCHEMA_VERSION
        ) || fixture.contract.as_deref() != Some("final_placed_heaps")
            || fixture.input.depths.len() != 1
        {
            continue;
        }
        if fixture.input.challenge.is_some() || fixture.input.trinket.is_some() {
            continue;
        }
        let depth = fixture.input.depths[0];
        if matches!(depth, 5 | 10 | 15 | 20 | 25 | 26) {
            continue;
        }

        let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
        let mut actual = None;
        for replay_depth in 1..=depth {
            dungeon.depth = replay_depth as i32;
            actual = Some(create_level_partial(&mut dungeon));
        }
        let actual = actual.expect("positive fixture depth");
        let expected = fixture.floors.first().expect("single oracle floor");
        let context = path.display();
        assert_eq!(expected.depth, depth, "fixture floor in {context}");
        let item_count = assert_main_loop_heaps_match(&actual, expected, &context);
        assert!(item_count > 0, "plain main-loop heap count in {context}");
        compared_floors += 1;
        compared_items += item_count;
    }

    assert!(
        compared_floors >= 51,
        "covered regular floors: {compared_floors}"
    );
    assert!(
        compared_items >= 160,
        "covered main-loop items: {compared_items}"
    );
}

fn assert_aaa_regression_facts(
    fixture: &OracleFixture,
    floor: &OracleFloor,
    projection: &[ComparableItem],
    context: &impl std::fmt::Display,
) {
    if fixture.input.seed != "AAA-AAA-AAA" {
        return;
    }

    assert_eq!(
        (floor.width, floor.height),
        (37, 43),
        "pinned AAA Java map bounds in {context}"
    );
    assert_eq!(
        floor.final_mobs,
        [
            OracleMob {
                cell: 289,
                class_name: "Rat".into()
            },
            OracleMob {
                cell: 394,
                class_name: "Piranha".into()
            },
            OracleMob {
                cell: 465,
                class_name: "Rat".into()
            },
            OracleMob {
                cell: 505,
                class_name: "Piranha".into()
            },
            OracleMob {
                cell: 507,
                class_name: "Piranha".into()
            },
            OracleMob {
                cell: 534,
                class_name: "Rat".into()
            },
            OracleMob {
                cell: 574,
                class_name: "Albino".into()
            },
            OracleMob {
                cell: 818,
                class_name: "Snake".into()
            },
            OracleMob {
                cell: 834,
                class_name: "Rat".into()
            },
            OracleMob {
                cell: 1090,
                class_name: "Snake".into()
            },
            OracleMob {
                cell: 1376,
                class_name: "Rat".into()
            },
        ],
        "pinned AAA Java final mobs in {context}"
    );
    assert_eq!(
        signatures(projection),
        [
            "Food",
            "PotionOfInvisibility",
            "ScrollOfRage",
            "Shuriken",
            "StoneOfAggression",
            "StoneOfBlink",
            "StoneOfDeepSleep",
            "ThrowingSpear",
            "WandOfFrost",
        ],
        "pinned AAA Java report-visible projection in {context}"
    );
}

fn assert_aaa_aad_public_report_facts(
    fixture: &OracleFixture,
    expected: &OracleFloor,
    floor: &spd_core::FloorReport,
    context: &impl std::fmt::Display,
) {
    if fixture.input.seed != "AAA-AAA-AAD" {
        return;
    }

    assert_eq!(
        floor
            .items
            .iter()
            .any(|item| item.source.as_deref() == Some("SacrificeRoom")),
        floor.rooms.iter().any(|room| room == "SacrificeRoom"),
        "Sacrifice contract is public only when room selection is seed-safe in {context}"
    );
    let oracle_spawns_seed = expected
        .final_heaps
        .iter()
        .flat_map(|heap| &heap.items)
        .any(|item| item.class_name == "Seed");
    let report_spawns_seed = floor.items.iter().any(|item| item.category == "seed");
    assert_eq!(
        report_spawns_seed, oracle_spawns_seed,
        "public seed presence matches the pinned Java floor in {context}"
    );
}

#[test]
fn aaa_aad_public_seed_presence_matches_v4_oracle() {
    let _journal = crate::fresh_journal();
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "aaa-aaa-aad-final-heaps-floor-1.json")
        })
        .expect("AAA-AAD floor-1 fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-1 oracle facts");
    let report = analyze_seed(&fixture.input.seed, 1).expect("analyze AAA-AAD");
    assert_aaa_aad_public_report_facts(&fixture, expected, &report.floors[0], &path.display());
}

#[test]
fn depth_one_final_heaps_match_report_projection() {
    let _journal = crate::fresh_journal();
    let mut compared = 0;
    for path in fixture_paths() {
        let fixture = read_fixture(&path);
        if !matches!(
            fixture.schema_version,
            FINAL_HEAPS_SCHEMA_VERSION | EXTENDED_FINAL_HEAPS_SCHEMA_VERSION
        ) {
            continue;
        }
        if fixture.input.challenge.is_some() {
            continue;
        }
        if fixture.input.depths != [1] {
            continue;
        }
        let context = path.display();
        assert_eq!(
            fixture.contract.as_deref(),
            Some("final_placed_heaps"),
            "schema v3 contract in {context}"
        );
        assert_eq!(fixture.input.depths, [1], "requested depths in {context}");
        assert_eq!(fixture.floors.len(), 1, "floor count in {context}");
        let expected_floor = &fixture.floors[0];
        assert_eq!(expected_floor.depth, 1, "floor depth in {context}");
        assert!(
            expected_floor.width > 0 && expected_floor.height > 0,
            "pinned Java map has positive bounds in {context}"
        );
        assert!(
            !expected_floor.rooms.is_empty()
                && expected_floor
                    .rooms
                    .windows(2)
                    .all(|pair| pair[0] <= pair[1]),
            "pinned Java rooms are non-empty and sorted in {context}"
        );
        assert_eq!(
            expected_floor.pre_paint_rng.len(),
            8,
            "pre-paint RNG probe length in {context}"
        );
        assert_eq!(
            expected_floor.pre_mobs_rng.len(),
            8,
            "pre-mobs RNG probe length in {context}"
        );
        assert_eq!(
            expected_floor.pre_items_rng.len(),
            8,
            "RNG probe length in {context}"
        );
        assert!(
            expected_floor.forced_items.is_empty(),
            "schema v3 does not reuse the schema v2 forced_items field in {context}"
        );
        assert!(
            !expected_floor.final_heaps.is_empty(),
            "final heap snapshot must not be empty in {context}"
        );
        assert!(
            expected_floor
                .final_heaps
                .windows(2)
                .all(|pair| pair[0].cell < pair[1].cell),
            "final heaps must be strictly ordered by cell in {context}"
        );
        for heap in &expected_floor.final_heaps {
            assert!(
                matches!(
                    heap.heap_type.as_str(),
                    "heap"
                        | "for_sale"
                        | "chest"
                        | "locked_chest"
                        | "crystal_chest"
                        | "tomb"
                        | "skeleton"
                        | "remains"
                ),
                "pinned Heap.Type in {context}: {}",
                heap.heap_type
            );
            assert!(!heap.items.is_empty(), "empty final heap in {context}");
            assert!(
                heap.items
                    .iter()
                    .all(|item| !item.class_name.is_empty() && item.quantity > 0),
                "valid final heap item facts in {context}"
            );
        }

        // The current public analyzer report does not retain quantity, level,
        // heap type, or an item-to-cell relation. Compare its strongest honest
        // projection (report-visible class + curse), while the fixture above
        // still validates and preserves every v3 placement fact.
        let mut expected_projection: Vec<_> = expected_floor
            .final_heaps
            .iter()
            .flat_map(|heap| &heap.items)
            .filter(|item| !report_blacklists(&item.class_name))
            .map(|item| ComparableItem {
                class_name: item.class_name.clone(),
                cursed: item.cursed,
            })
            .collect();
        expected_projection.sort();

        let report = analyze_seed(&fixture.input.seed, 1)
            .unwrap_or_else(|error| panic!("failed to analyze seed in {context}: {error}"));
        let mut actual_rooms = report.floors[0].rooms.clone();
        actual_rooms.sort();
        assert_eq!(
            actual_rooms, expected_floor.rooms,
            "depth-one room classes in {context}"
        );
        let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
        dungeon.depth = 1;
        let level = create_level_partial(&mut dungeon);
        // Oracle parity inspects the exact internal map. The public report
        // intentionally omits runtime-history-sensitive createItems cells.
        let map = level.map.as_ref().expect("depth-one internal map");
        magical_fire::assert_abc_magical_fire_facts(&fixture, map, &context);
        runestone::assert_aaa_afu_facts(&fixture, map, &context);
        traps::assert_hello_traps_facts(&fixture, map, &context);
        sentry::assert_aaz_sentry_facts(&fixture, map, &context);
        assert_eq!(
            (map.width, map.height),
            (expected_floor.width, expected_floor.height),
            "Rust map bounds in {context}"
        );
        assert_eq!(
            level.pre_paint_rng_probe, expected_floor.pre_paint_rng,
            "pre-painter RNG boundary in {context}"
        );
        assert_eq!(
            level.pre_mobs_rng_probe, expected_floor.pre_mobs_rng,
            "pre-createMobs RNG boundary in {context}"
        );
        assert_eq!(
            level.pre_items_rng_probe, expected_floor.pre_items_rng,
            "pre-createItems RNG boundary in {context}"
        );
        if let Some(terrain) = &expected_floor.terrain {
            let mismatches: Vec<_> = map
                .tiles
                .iter()
                .zip(terrain)
                .enumerate()
                .filter_map(|(cell, (&actual, &expected))| {
                    (actual != expected).then_some((cell, actual, expected))
                })
                .collect();
            assert!(
                mismatches.is_empty(),
                "depth-one final terrain mismatches (cell, actual, expected) in {context}: {mismatches:?}"
            );
        }
        if let Some(discoverable) = &expected_floor.discoverable {
            assert_eq!(
                map.discoverable, *discoverable,
                "depth-one discoverable mask cell-for-cell in {context}"
            );
        }
        if let Some(tile_variance) = &expected_floor.tile_variance {
            assert_eq!(
                map.tile_variance, *tile_variance,
                "depth-one tile variance cell-for-cell in {context}"
            );
        }
        if expected_floor.terrain.is_some() {
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
                            // Java reports nested plant seed classes by their
                            // ambiguous simple name; preserve exact item
                            // identity in the analyzer and normalize only at
                            // this oracle boundary.
                            class_name: if item.class_name().ends_with("Seed") {
                                "Seed".into()
                            } else {
                                item.class_name().to_string()
                            },
                            quantity: item.quantity,
                            level: item.level,
                            cursed: item.cursed,
                        })
                        .collect(),
                })
                .collect();
            assert_eq!(
                actual_heaps, expected_floor.final_heaps,
                "depth-one structured heaps in {context}"
            );
            let actual_mobs: Vec<_> = map
                .mobs
                .iter()
                .map(|mob| OracleMob {
                    cell: mob.cell,
                    class_name: mob.class_name().to_string(),
                })
                .collect();
            assert_eq!(
                actual_mobs, expected_floor.final_mobs,
                "depth-one structured mobs in {context}"
            );
        }
        if let Some(transitions) = &expected_floor.transitions {
            let actual: Vec<_> = map
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
            assert_eq!(actual, *transitions, "depth-one transitions in {context}");
        }
        if let Some(traps) = &expected_floor.traps {
            let actual: Vec<_> = map
                .traps
                .iter()
                .map(|trap| OracleTrap {
                    cell: trap.cell,
                    class_name: trap.class_name().to_string(),
                    visible: trap.visible,
                    active: trap.active,
                    color: trap.color,
                    shape: trap.shape,
                })
                .collect();
            assert_eq!(actual, *traps, "depth-one traps in {context}");
        }
        if let Some(plants) = &expected_floor.plants {
            let actual: Vec<_> = map
                .plants
                .iter()
                .map(|plant| OraclePlant {
                    cell: plant.cell,
                    class_name: plant.class_name().to_string(),
                    image: plant.image,
                })
                .collect();
            assert_eq!(actual, *plants, "depth-one plants in {context}");
        }
        if let Some(blobs) = &expected_floor.blobs {
            let actual: Vec<_> = map
                .blobs
                .iter()
                .map(|blob| OracleBlob {
                    class_name: blob.class_name().to_string(),
                    volume: blob.volume,
                    always_visible: blob.always_visible,
                    cells: blob
                        .cells
                        .iter()
                        .map(|cell| OracleBlobCell {
                            cell: cell.cell,
                            value: cell.value,
                        })
                        .collect(),
                })
                .collect();
            assert_eq!(actual, *blobs, "depth-one active blobs in {context}");
        }
        let actual_item_cells: Vec<_> = map
            .markers
            .iter()
            .filter(|marker| marker.kind == spd_core::report::MapMarkerKind::Item)
            .map(|marker| marker.cell)
            .collect();
        let expected_item_cells: Vec<_> = expected_floor
            .final_heaps
            .iter()
            .map(|heap| heap.cell)
            .collect();
        assert_eq!(
            actual_item_cells, expected_item_cells,
            "depth-one final heap cells in {context}"
        );
        if let Some(public_map) = &report.floors[0].map {
            assert!(
                public_map.markers == map.markers
                    && public_map.heaps == map.heaps
                    && public_map.mobs == map.mobs,
                "public depth-one map must publish the populated entities in {context}"
            );
        }
        assert_aaa_aad_public_report_facts(&fixture, expected_floor, &report.floors[0], &context);

        // FloorOracle's final-heaps contract cannot see the prize held by the
        // SacrificialFire blob. Exclude that reward only at this heap-only
        // comparison boundary; the public report assertion above keeps it.
        // Java seed implementations are nested `*.Seed` classes, so its
        // `getSimpleName()` observation is canonicalized here without
        // flattening the analyzer's precise item class names.
        let mut actual_projection: Vec<_> = level
            .forced_items
            .iter()
            .chain(&level.placed_items)
            .filter(|item| !report_blacklists(&item.class_name))
            .filter(|item| item.source.as_deref() != Some("SacrificeRoom"))
            .map(|item| ComparableItem {
                class_name: if item.class_name.ends_with("Seed") {
                    "Seed".to_string()
                } else {
                    item.class_name.clone()
                },
                cursed: item.cursed,
            })
            .collect();
        actual_projection.sort();

        assert_aaa_regression_facts(&fixture, expected_floor, &expected_projection, &context);
        assert_eq!(
            actual_projection, expected_projection,
            "internal exact item projection in {context}"
        );
        assert_eq!(report.status, "partial", "accuracy status in {context}");
        compared += 1;
    }
    assert!(
        compared >= 8,
        "expected at least eight depth-one final-heaps fixtures, compared {compared}"
    );
}

#[test]
fn barren_land_fixture_declares_its_challenge_profile() {
    let _journal = crate::fresh_journal();
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "aaa-aaa-aaa-final-heaps-floor-4-barren-land.json")
        })
        .expect("AAA floor-4 Barren Land fixture");
    let fixture = read_fixture(&path);
    assert_eq!(fixture.input.depths, [4]);
    assert_eq!(fixture.input.challenge.as_deref(), Some("barren-land"));
}

#[test]
fn rat_skull_fixtures_declare_profile_state() {
    let _journal = crate::fresh_journal();
    let mut found = 0;
    for path in fixture_paths() {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.contains("rat-skull") {
            continue;
        }
        let fixture = read_fixture(&path);
        assert_eq!(fixture.input.trinket.as_deref(), Some("rat-skull"));
        assert_eq!(fixture.input.trinket_level, Some(3));
        assert_eq!(fixture.input.trinket_before_depth, Some(6));
        assert_eq!(fixture.input.challenge, None);
        found += 1;
    }
    assert_eq!(found, 2, "Rat Skull CrystalVault and Statue fixtures");
}

#[test]
fn rat_skull_fixtures_cover_crystal_vault_and_statue_boundaries() {
    let _journal = crate::fresh_journal();
    let mut depths = Vec::new();
    for path in fixture_paths() {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.contains("rat-skull") {
            continue;
        }
        let fixture = read_fixture(&path);
        let floor = fixture.floors.first().expect("Rat Skull floor");
        assert_eq!(floor.pre_paint_rng.len(), 8);
        assert_eq!(floor.pre_mobs_rng.len(), 8);
        assert_eq!(floor.pre_items_rng.len(), 8);
        assert!(!floor.final_heaps.is_empty());
        assert!(!floor.final_mobs.is_empty());
        assert!(floor.rooms.iter().any(|room| {
            (floor.depth == 6 && room == "CrystalVaultRoom")
                || (floor.depth == 7 && room == "StatueRoom")
        }));
        depths.push(floor.depth);
    }
    depths.sort_unstable();
    assert_eq!(depths, [6, 7]);
}

#[test]
fn trinket_fixtures_replay_exact_profiled_floor_state() {
    let _journal = crate::fresh_journal();
    let mut failures = Vec::new();
    fn normalize(mut heaps: Vec<OracleHeap>) -> Vec<OracleHeap> {
        for heap in &mut heaps {
            heap.items.sort_by_key(|item| {
                (
                    item.class_name.clone(),
                    item.level,
                    item.cursed,
                    item.quantity,
                )
            });
            let mut merged: Vec<OracleItem> = Vec::new();
            for item in heap.items.drain(..) {
                if let Some(last) = merged.last_mut() {
                    if last.class_name == item.class_name
                        && last.level == item.level
                        && last.cursed == item.cursed
                    {
                        last.quantity += item.quantity;
                        continue;
                    }
                }
                merged.push(item);
            }
            heap.items = merged;
        }
        heaps
    }

    for path in fixture_paths() {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let fixture = read_fixture(&path);
        let trinket = match fixture.input.trinket.as_deref() {
            Some("rat-skull") => spd_core::TrinketKind::RatSkull,
            Some("mimic-tooth") => spd_core::TrinketKind::MimicTooth,
            Some("cracked-spyglass") => spd_core::TrinketKind::CrackedSpyglass,
            Some("parchment-scrap") => spd_core::TrinketKind::ParchmentScrap,
            Some(other) => panic!("{name}: unsupported oracle trinket {other}"),
            None => continue,
        };
        let expected = fixture.floors.first().expect("trinket floor");
        let profile = spd_core::MapProfile {
            trinket_events: vec![spd_core::TrinketEvent {
                before_depth: fixture.input.trinket_before_depth.expect("before depth"),
                action: spd_core::TrinketEventAction::Acquired {
                    trinket,
                    min_upgrades: fixture.input.trinket_level,
                },
            }],
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
        let actual = actual.expect("profiled floor");
        let mut check = |what: &str, same: bool, detail: String| {
            if !same {
                failures.push(format!("{name} {what}: {detail}"));
            }
        };
        check(
            "pre-paint",
            actual.pre_paint_rng_probe == expected.pre_paint_rng,
            String::new(),
        );
        check(
            "pre-mobs",
            actual.pre_mobs_rng_probe == expected.pre_mobs_rng,
            String::new(),
        );
        check(
            "pre-items",
            actual.pre_items_rng_probe == expected.pre_items_rng,
            String::new(),
        );
        let map = actual.map.as_ref().expect("profiled map");
        let mobs: Vec<_> = map
            .mobs
            .iter()
            .map(|mob| OracleMob {
                cell: mob.cell,
                class_name: mob.class_name().to_string(),
            })
            .collect();
        check(
            "mobs",
            mobs == expected.final_mobs,
            format!("\n  rust {mobs:?}\n  java {:?}", expected.final_mobs),
        );
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
                        class_name: normalize_oracle_class(item.class_name()).to_string(),
                        quantity: item.quantity,
                        level: item.level,
                        cursed: item.cursed,
                    })
                    .collect(),
            })
            .collect();
        let mut java_heaps = expected.final_heaps.clone();
        if expected.depth == 2 {
            // `EntranceRoom.placeEarlyGuidePages` drops floor 2's guide page
            // under an unseeded generator, so its cell differs per run.
            let page = |heap: &OracleHeap| {
                heap.items.len() == 1 && heap.items[0].class_name == "GuidePage"
            };
            let before = java_heaps.len();
            java_heaps.retain(|heap| {
                !page(heap)
                    || heaps
                        .iter()
                        .any(|rust| rust.cell == heap.cell && page(rust))
            });
            check(
                "entrance guide page",
                before == java_heaps.len() + 1,
                format!("removed {}", before - java_heaps.len()),
            );
        }
        let (heaps, java_heaps) = (normalize(heaps), normalize(java_heaps));
        let only_rust: Vec<_> = heaps.iter().filter(|h| !java_heaps.contains(h)).collect();
        let only_java: Vec<_> = java_heaps.iter().filter(|h| !heaps.contains(h)).collect();
        check(
            "heaps",
            only_rust.is_empty() && only_java.is_empty(),
            format!("\n  only rust {only_rust:?}\n  only java {only_java:?}"),
        );
        let mut rust_enchantments: Vec<_> = map
            .heaps
            .iter()
            .flat_map(|heap| {
                heap.items.iter().filter_map(move |item| {
                    let enchantment = item.enchantment?;
                    Some((
                        heap.cell,
                        item.class_name().to_string(),
                        enchantment.class_name().to_string(),
                    ))
                })
            })
            .collect();
        let mut java_enchantments = oracle_heap_enchantments(&path);
        rust_enchantments.sort();
        java_enchantments.sort();
        check(
            "enchantments",
            rust_enchantments == java_enchantments,
            format!("\n  rust {rust_enchantments:?}\n  java {java_enchantments:?}"),
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `(cell, item class, enchantment class)` for every enchanted final heap
/// item. Trinket fixtures record weapon enchantments and armor glyphs.
fn oracle_heap_enchantments(path: &std::path::Path) -> Vec<(u32, String, String)> {
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("fixture")).expect("json");
    let mut out = Vec::new();
    for heap in json["floors"][0]["final_heaps"].as_array().expect("heaps") {
        let cell = heap["cell"].as_u64().expect("cell") as u32;
        for item in heap["items"].as_array().expect("items") {
            if let Some(enchantment) = item["enchantment"].as_str() {
                let class_name = item["class"].as_str().expect("class");
                out.push((cell, class_name.to_string(), enchantment.to_string()));
            }
        }
    }
    out
}

#[test]
fn aaa_floor_four_barren_land_matches_secret_garden_oracle() {
    let _journal = crate::fresh_journal();
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "aaa-aaa-aaa-final-heaps-floor-4-barren-land.json")
        })
        .expect("AAA floor-4 Barren Land fixture");
    let fixture = read_fixture(&path);
    let expected = fixture.floors.first().expect("floor-4 oracle facts");

    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    dungeon.challenges = spd_core::Challenge::BarrenLand.mask();
    let mut actual = None;
    for depth in 1..=4 {
        dungeon.depth = depth;
        actual = Some(create_level_partial(&mut dungeon));
    }
    let actual = actual.expect("barren floor");
    let context = path.display();

    let mut rooms = actual.rooms.clone();
    rooms.sort();
    assert_eq!(
        rooms, expected.rooms,
        "Barren Land room classes in {context}"
    );
    assert_eq!(actual.pre_paint_rng_probe, expected.pre_paint_rng);
    assert_eq!(actual.pre_mobs_rng_probe, expected.pre_mobs_rng);
    assert_eq!(actual.pre_items_rng_probe, expected.pre_items_rng);

    let map = actual.map.as_ref().expect("barren floor map");
    assert_eq!((map.width, map.height), (expected.width, expected.height));
    assert_eq!(map.tiles, *expected.terrain.as_ref().expect("terrain"));
    assert!(map.plants.is_empty(), "Barren Land must not couch plants");
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
                    class_name: item.class_name().to_string(),
                    quantity: item.quantity,
                    level: item.level,
                    cursed: item.cursed,
                })
                .collect(),
        })
        .collect();
    assert_eq!(
        actual_heaps, expected.final_heaps,
        "Barren Land heaps in {context}"
    );
    let actual_mobs: Vec<_> = map
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    assert_eq!(
        actual_mobs, expected.final_mobs,
        "Barren Land mobs in {context}"
    );
}

#[test]
fn aaa_floor_three_forbidden_runes_suppresses_even_upgrade_scroll() {
    let _journal = crate::fresh_journal();
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "aaa-aaa-aaa-final-heaps-floor-3-forbidden-runes.json")
        })
        .expect("AAA floor-3 Forbidden Runes fixture");
    let fixture = read_fixture(&path);
    assert_eq!(fixture.input.challenge.as_deref(), Some("forbidden-runes"));
    let expected = fixture.floors.first().expect("Forbidden Runes floor");
    let profile = spd_core::MapProfile {
        challenges: vec![spd_core::Challenge::ForbiddenRunes],
        ..spd_core::MapProfile::default()
    };
    let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
    let mut actual = None;
    let mut upgrade_counts = Vec::new();
    for depth in 1..=expected.depth {
        dungeon.depth = depth as i32;
        let level = spd_core::level::create_level_partial_for_profile(&mut dungeon, &profile);
        upgrade_counts.push(
            level
                .initial_forced_items
                .iter()
                .filter(|item| item.class_name == "ScrollOfUpgrade")
                .count(),
        );
        actual = Some(level);
    }
    let actual = actual.expect("profiled floor");
    assert_eq!(upgrade_counts, [0, 1, 0], "forced Upgrade Scroll queue");
    assert_eq!(actual.pre_paint_rng_probe, expected.pre_paint_rng);
    assert_eq!(actual.pre_mobs_rng_probe, expected.pre_mobs_rng);
    assert_eq!(actual.pre_items_rng_probe, expected.pre_items_rng);
    assert!(actual.initial_forced_items.iter().all(|item| {
        item.class_name != "ScrollOfUpgrade"
            || item.provenance
                != spd_core::items::model::ItemProvenance::Forced(
                    spd_core::items::model::ForcedDropRole::UpgradeScroll {
                        forbidden_runes_sensitive: true,
                    },
                )
    }));
    let map = actual.map.as_ref().expect("internal profiled map");
    let mut rooms = actual.rooms.clone();
    rooms.sort();
    assert_eq!(rooms, expected.rooms, "room classes");
    assert_eq!((map.width, map.height), (expected.width, expected.height));
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
                    class_name: normalize_oracle_class(item.class_name()).to_string(),
                    quantity: item.quantity,
                    level: item.level,
                    cursed: item.cursed,
                })
                .collect(),
        })
        .collect();
    assert_eq!(heaps, expected.final_heaps);
}

#[test]
fn aaa_floor_fifteen_badder_bosses_matches_caves_layout_and_rng() {
    let _journal = crate::fresh_journal();
    let path = fixture_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .is_some_and(|name| name == "aaa-aaa-aaa-final-heaps-floor-15-badder-bosses.json")
        })
        .expect("AAA floor-15 Badder Bosses fixture");
    let fixture = read_fixture(&path);
    assert_eq!(fixture.input.challenge.as_deref(), Some("badder-bosses"));
    let expected = fixture.floors.first().expect("Badder Bosses floor");
    let profile = spd_core::MapProfile {
        challenges: vec![spd_core::Challenge::BadderBosses],
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
    let actual = actual.expect("profiled boss floor");
    assert!(
        !actual.runtime_sensitive_layout,
        "Badder Bosses layout is modeled"
    );
    assert_eq!(actual.pre_items_rng_probe, expected.pre_items_rng);
    let map = actual.layout_map.as_ref().expect("Caves layout map");
    let terrain = expected.terrain.as_ref().expect("Caves terrain");
    assert_eq!(&map.tiles, terrain, "Badder Bosses Caves terrain");
    assert_eq!(
        map.discoverable,
        *expected
            .discoverable
            .as_ref()
            .expect("Caves discoverability")
    );
    let actual_transitions: Vec<_> = map
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
        actual_transitions,
        *expected.transitions.as_ref().expect("Caves transitions")
    );
    let mut mobs: Vec<_> = map
        .mobs
        .iter()
        .map(|mob| OracleMob {
            cell: mob.cell,
            class_name: mob.class_name().to_string(),
        })
        .collect();
    mobs.sort_by_key(|mob| mob.cell);
    assert_eq!(mobs, expected.final_mobs, "Badder Bosses pylons");
}

#[path = "final_heaps/runestone.rs"]
mod runestone;

#[path = "final_heaps/magical_fire.rs"]
mod magical_fire;

#[path = "final_heaps/sentry.rs"]
mod sentry;
#[path = "final_heaps/traps.rs"]
mod traps;

#[path = "final_heaps/floor_six.rs"]
mod floor_six;

#[path = "final_heaps/floor_seven.rs"]
mod floor_seven;

#[path = "final_heaps/floor_eight.rs"]
mod floor_eight;

#[path = "final_heaps/challenges.rs"]
mod challenges;

#[path = "final_heaps/secret_well.rs"]
mod secret_well;
