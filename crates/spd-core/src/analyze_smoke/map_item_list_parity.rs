use std::collections::HashMap;

use spd_objects::ItemId;

use crate::report::{FloorReport, ItemPredictionKind};
use crate::{analyze_seed, analyze_seed_for_run, Challenge};

fn is_map_plumbing(item: ItemId) -> bool {
    matches!(
        item.class_name(),
        "Gold"
            | "Dewdrop"
            | "IronKey"
            | "GoldenKey"
            | "CrystalKey"
            | "AlchemyPage"
            | "GuidePage"
            | "DocumentPage"
            | "TornPage"
            | "CorpseDust"
            | "Embers"
            | "CeremonialCandle"
            | "Pickaxe"
            | "Amulet"
    )
}

fn assert_map_items_listed(seed: &str, floors: &[FloorReport]) {
    for floor in floors {
        let Some(map) = floor.map.as_ref().or(floor.assumed_map.as_ref()) else {
            continue;
        };
        let mut map_counts = HashMap::<ItemId, i32>::new();
        for heap in &map.heaps {
            for item in &heap.items {
                if !is_map_plumbing(item.item) {
                    *map_counts.entry(item.item).or_default() += item.quantity.max(1);
                }
            }
        }
        for mob in &map.mobs {
            for item in &mob.items {
                if !is_map_plumbing(item.item) {
                    *map_counts.entry(item.item).or_default() += item.quantity.max(1);
                }
            }
        }

        let mut listed_counts = HashMap::<ItemId, i32>::new();
        for group in &floor.items {
            let source = group.source.as_deref().unwrap_or_default();
            if source.ends_with(".Quest") {
                continue;
            }
            let has_variable = group
                .variants
                .iter()
                .any(|variant| variant.prediction == ItemPredictionKind::Variable);
            for variant in &group.variants {
                if variant.prediction == ItemPredictionKind::Constrained && has_variable {
                    continue;
                }
                if let Some(item_id) = variant.item {
                    let count = variant
                        .quantity_range
                        .map_or(variant.quantity.max(1), |range| {
                            range.max.max(variant.quantity)
                        });
                    *listed_counts.entry(item_id).or_default() += count;
                } else {
                    for &candidate in &variant.candidates {
                        *listed_counts.entry(candidate).or_default() += variant.quantity.max(1);
                    }
                }
            }
        }

        for (item_id, map_qty) in map_counts {
            let listed_qty = listed_counts.get(&item_id).copied().unwrap_or(0);
            assert!(
                listed_qty >= map_qty,
                "{seed} floor {}: {} has {map_qty} on map but only {listed_qty} in floor.items",
                floor.depth,
                item_id.class_name()
            );
        }
    }
}

#[test]
fn floor_item_lists_cover_all_non_plumbing_map_heaps_and_mimic_drops() {
    let zwd = analyze_seed("CHL-XBM-ZWD", 15).unwrap();
    assert_map_items_listed("CHL-XBM-ZWD", &zwd.floors);
    assert!(zwd.floors[1]
        .item_variants()
        .any(|item| item.item == Some(ItemId::ScrollOfRemoveCurse)));

    let zdd = analyze_seed("CHL-XBM-ZDD", 15).unwrap();
    assert_map_items_listed("CHL-XBM-ZDD", &zdd.floors);
    assert!(zdd.floors[1]
        .item_variants()
        .any(|item| item.item == Some(ItemId::ScrollOfLullaby)));
    assert!(zdd.floors[1]
        .item_variants()
        .any(|item| item.item == Some(ItemId::Pasty)));

    let darkness =
        analyze_seed_for_run("FVX-YTN-SLC", 8, None, &[Challenge::IntoDarkness]).unwrap();
    assert_map_items_listed("FVX-YTN-SLC:into_darkness", &darkness.floors);

    for seed in 0..16 {
        let input = seed.to_string();
        let report = analyze_seed(&input, 15).unwrap();
        assert_map_items_listed(&input, &report.floors);
    }
}
