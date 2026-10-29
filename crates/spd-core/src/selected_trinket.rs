//! Report for a seed whose Trinket Catalyst choice the player has made.

use crate::{
    analyze_route, analyze_run, analyze_seed, conditional, init_run, parse_seed, AnalyzeError,
    SeedReport, TrinketKind, TrinketSelectionReport,
};

/// Analyze a seed with `trinket` taken from the Catalyst and held at
/// [`TrinketSelectionReport::PICKED_LEVEL`] from the first floor it can
/// influence. `None` is [`analyze_seed`].
///
/// The held trinket is a player fact, so its replay publishes concrete
/// variable samples like the default report. Only the Forbidden Runes
/// challenge stays a spawn condition; upgrade and transmutation routes are
/// not enumerated.
pub fn analyze_seed_with_trinket(
    input: &str,
    floors: u32,
    trinket: Option<TrinketKind>,
) -> Result<SeedReport, AnalyzeError> {
    let Some(trinket) = trinket else {
        return analyze_seed(input, floors);
    };
    let info = parse_seed(input)?;
    let run = init_run(info.numeric);
    let selection = TrinketSelectionReport::for_run(&run);
    let profile = selection.catalyst_profile(trinket)?;
    let mut report = analyze_run(info, run, selection, floors, Some(&profile), true)?;

    let (base, routes) =
        conditional::pinned_profile_routes(&profile, report.floors_requested, false);
    let mut alternatives = Vec::new();
    for route in routes {
        let floors = analyze_route(input, &route, &report.trinket_selection)?;
        alternatives.push((route, floors));
    }
    conditional::merge_possible_rooms(&mut report.floors, base.clone(), &alternatives);
    conditional::merge_possible_items(&mut report.floors, base, &mut alternatives);
    Ok(report)
}

#[cfg(test)]
#[path = "selected_trinket/parchment_tests.rs"]
mod parchment_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProfileError, TrinketGenerationEffect};

    fn offered(report: &SeedReport, effect: TrinketGenerationEffect) -> Option<TrinketKind> {
        let selection = &report.trinket_selection;
        selection
            .catalyst_options
            .iter()
            .zip(&selection.catalyst_option_effects)
            .find(|(_, candidate)| **candidate == effect)
            .and_then(|(item, _)| TrinketKind::from_class_name(item.class_name()))
    }

    fn displayed_tiles(floor: &crate::FloorReport) -> Option<&Vec<u16>> {
        floor
            .map
            .as_ref()
            .or(floor.assumed_map.as_ref())
            .map(|map| &map.tiles)
    }

    #[test]
    fn mimic_tooth_reports_carried_drops_including_ebony() {
        use crate::report::FloorReport;

        fn carried<'a>(floor: &'a FloorReport, mob: &str) -> Option<&'a crate::report::MapMob> {
            floor
                .map
                .as_ref()
                .or(floor.assumed_map.as_ref())
                .and_then(|map| map.mobs.iter().find(|entry| entry.class_name() == mob))
        }

        let mut saw_ebony = false;
        for seed in 0..24 {
            let selection = TrinketSelectionReport::for_run(&crate::init_run(seed));
            let offered = selection
                .catalyst_options
                .iter()
                .any(|item| item.class_name() == "MimicTooth");
            if !offered {
                continue;
            }
            let report =
                analyze_seed_with_trinket(&seed.to_string(), 8, Some(TrinketKind::MimicTooth))
                    .unwrap();
            let tooth_from = report.trinket_selection.first_effective_depth;
            for floor in &report.floors {
                if let Some(ebony) = carried(floor, "EbonyMimic") {
                    assert_eq!(ebony.items.len(), 3, "seed {seed} floor {}", floor.depth);
                    let map = floor.map.as_ref().or(floor.assumed_map.as_ref()).unwrap();
                    assert!(map.concealed_cells.contains(&ebony.cell), "seed {seed}");
                    let listed: Vec<_> = floor
                        .items
                        .iter()
                        .filter(|group| {
                            group
                                .source
                                .as_deref()
                                .is_some_and(|source| source.contains("ebony_mimic"))
                        })
                        .flat_map(|group| group.variants.iter())
                        .filter_map(|item| item.class_name().map(str::to_string))
                        .collect();
                    for item in &ebony.items {
                        let class_name = item.class_name();
                        if matches!(
                            class_name,
                            "Gold" | "Dewdrop" | "IronKey" | "GoldenKey" | "CrystalKey"
                        ) {
                            continue;
                        }
                        assert!(
                            listed.iter().any(|listed| listed == class_name),
                            "seed {seed} floor {} missing {class_name} in {listed:?}",
                            floor.depth
                        );
                    }
                    saw_ebony = true;
                }
                if let Some(crystal) = carried(floor, "CrystalMimic") {
                    assert_eq!(crystal.items.len(), 1, "seed {seed}");
                    assert!(!crystal.items[0].cursed);
                }
                if floor.depth >= tooth_from {
                    for mob in ["Mimic", "GoldenMimic"] {
                        if let Some(mimic) = carried(floor, mob) {
                            assert_eq!(
                                mimic.items.len(),
                                3,
                                "seed {seed} {mob} floor {}",
                                floor.depth
                            );
                        }
                    }
                }
            }
            if saw_ebony {
                break;
            }
        }
        assert!(saw_ebony, "no offered Mimic Tooth produced an ebony mimic");
    }

    #[test]
    fn cracked_spyglass_places_concealed_extra_loot() {
        let seed = (0..40)
            .find(|&seed| {
                TrinketSelectionReport::for_run(&init_run(seed))
                    .catalyst_options
                    .iter()
                    .any(|item| item.class_name() == "CrackedSpyglass")
            })
            .expect("a seed offering Cracked Spyglass");
        let report =
            analyze_seed_with_trinket(&seed.to_string(), 8, Some(TrinketKind::CrackedSpyglass))
                .unwrap();
        let from = report.trinket_selection.first_effective_depth;
        for floor in &report.floors {
            let map = floor.map.as_ref().or(floor.assumed_map.as_ref()).unwrap();
            if floor.depth < from || floor.depth % 5 == 0 {
                assert!(map.concealed_cells.is_empty(), "floor {}", floor.depth);
                continue;
            }
            // `(int)(Random.Float() + 1.5)` at +3: one or two extra heaps.
            assert!(
                (1..=2).contains(&map.concealed_cells.len()),
                "seed {seed} floor {}: {:?}",
                floor.depth,
                map.concealed_cells
            );
            for cell in &map.concealed_cells {
                assert!(map.heaps.iter().any(|heap| heap.cell == *cell
                    && heap.items[0].source.as_deref() == Some("cracked_spyglass")));
            }
        }
    }

    #[test]
    fn no_selection_is_the_merged_default_report() {
        let default = serde_json::to_value(analyze_seed("7", 6).unwrap()).unwrap();
        let none = serde_json::to_value(analyze_seed_with_trinket("7", 6, None).unwrap()).unwrap();
        assert_eq!(default, none);
    }

    #[test]
    fn option_effects_align_with_catalyst_options() {
        for seed in 0..20 {
            let selection = TrinketSelectionReport::for_run(&init_run(seed));
            assert_eq!(
                selection.catalyst_option_effects.len(),
                selection.catalyst_options.len()
            );
            for (item, effect) in selection
                .catalyst_options
                .iter()
                .zip(&selection.catalyst_option_effects)
            {
                let trinket = TrinketKind::from_class_name(item.class_name()).unwrap();
                assert_eq!(trinket.generation_effect(), *effect);
            }
        }
    }

    #[test]
    fn picked_trinket_is_held_at_plus_three_from_its_first_floor() {
        let selection = TrinketSelectionReport::for_run(&init_run(7));
        let trinket =
            TrinketKind::from_class_name(selection.catalyst_options[0].class_name()).unwrap();
        let profile = selection.catalyst_profile(trinket).unwrap();
        let first = selection.first_effective_depth;
        assert_eq!(profile.held_at(first - 1), None);
        let held = profile.held_at(first).unwrap();
        assert_eq!(held.trinket, trinket);
        assert_eq!(held.level, TrinketSelectionReport::PICKED_LEVEL);
        assert_eq!(held.level, 3);
    }

    #[test]
    fn rejects_a_trinket_the_catalyst_does_not_offer() {
        let report = analyze_seed("7", 1).unwrap();
        let missing = TrinketKind::ALL
            .into_iter()
            .find(|trinket| {
                !report
                    .trinket_selection
                    .catalyst_options
                    .iter()
                    .any(|item| item.class_name() == trinket.class_name())
            })
            .unwrap();
        assert!(matches!(
            analyze_seed_with_trinket("7", 6, Some(missing)),
            Err(AnalyzeError::Profile(ProfileError::TrinketNotOffered(_)))
        ));
    }

    #[test]
    fn selected_trinket_keeps_floors_before_it_can_act() {
        let mut checked = 0;
        for seed in 0..40 {
            let input = seed.to_string();
            let default = analyze_seed(&input, 8).unwrap();
            let Some(trinket) = offered(&default, TrinketGenerationEffect::Modeled) else {
                continue;
            };
            let selected = analyze_seed_with_trinket(&input, 8, Some(trinket)).unwrap();
            let first = selected.trinket_selection.first_effective_depth;
            assert_eq!(selected.trinket_selection, default.trinket_selection);
            for (before, after) in default.floors.iter().zip(&selected.floors) {
                if after.depth >= first {
                    break;
                }
                assert_eq!(
                    serde_json::to_value(before).unwrap(),
                    serde_json::to_value(after).unwrap(),
                    "seed {seed} depth {} changed before trinket takes effect at {first}",
                    after.depth
                );
            }
            checked += 1;
        }
        assert!(checked > 0, "no seed offered a modeled trinket");
    }

    #[test]
    fn selected_feeling_trinket_changes_a_later_layout() {
        let changed = (0..60).any(|seed| {
            let input = seed.to_string();
            let default = analyze_seed(&input, 10).unwrap();
            let offers_mossy = default
                .trinket_selection
                .catalyst_options
                .iter()
                .any(|item| item.class_name() == TrinketKind::MossyClump.class_name());
            if !offers_mossy {
                return false;
            }
            let selected =
                analyze_seed_with_trinket(&input, 10, Some(TrinketKind::MossyClump)).unwrap();
            default
                .floors
                .iter()
                .zip(&selected.floors)
                .any(|(before, after)| displayed_tiles(before) != displayed_tiles(after))
        });
        assert!(changed, "Mossy Clump never changed a later floor layout");
    }

    #[test]
    fn selection_floor_ghost_reward_stays_fixed_when_trinket_is_picked() {
        let base_zwd = analyze_seed("CHL-XBM-ZWD", 4).unwrap();
        assert_eq!(base_zwd.trinket_selection.selection_depth, 2);
        assert_eq!(base_zwd.trinket_selection.first_effective_depth, 3);
        let picked_zwd =
            analyze_seed_with_trinket("CHL-XBM-ZWD", 4, Some(TrinketKind::ParchmentScrap)).unwrap();
        assert_eq!(
            serde_json::to_value(&base_zwd.floors[1]).unwrap(),
            serde_json::to_value(&picked_zwd.floors[1]).unwrap(),
        );

        let base_zdd = analyze_seed("CHL-XBM-ZDD", 4).unwrap();
        assert_eq!(base_zdd.trinket_selection.selection_depth, 3);
        assert_eq!(base_zdd.trinket_selection.first_effective_depth, 4);
        let ghost_groups: Vec<_> = base_zdd.floors[1]
            .items
            .iter()
            .filter(|group| group.source.as_deref() == Some("Ghost.Quest"))
            .collect();
        assert_eq!(ghost_groups.len(), 2);
        assert!(ghost_groups
            .iter()
            .flat_map(|group| &group.variants)
            .all(|variant| variant.spawn_conditions.is_empty()));
        let picked_zdd =
            analyze_seed_with_trinket("CHL-XBM-ZDD", 4, Some(TrinketKind::TrapMechanism)).unwrap();
        for depth_idx in 0..3 {
            assert_eq!(
                serde_json::to_value(&base_zdd.floors[depth_idx]).unwrap(),
                serde_json::to_value(&picked_zdd.floors[depth_idx]).unwrap(),
            );
        }
    }
}
