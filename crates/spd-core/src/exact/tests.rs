use super::*;
use crate::report::ItemPredictionKind;

const SEED: &str = "HDN-UJD-MJF";

fn report(maps: bool) -> ExactReport {
    analyze_seed_exact(SEED, 26, ExactOptions { maps }).expect("analyze exact")
}

fn floor(report: &ExactReport, depth: u32) -> &ExactFloor {
    report
        .floors
        .iter()
        .find(|floor| floor.depth == depth)
        .expect("floor")
}

fn has(items: &[ExactItem], class_name: &str) -> bool {
    items.iter().any(|item| item.item == class_name)
}

#[test]
fn reports_every_requested_floor_without_predictions() {
    let report = report(false);
    assert_eq!(report.seed, SEED);
    assert_eq!(report.floors.len(), 26);
    let json = serde_json::to_string(&report).expect("serialize");
    assert!(!json.contains("constrained"));
    assert!(!json.contains("prediction"));
}

#[test]
fn formerly_constrained_entries_are_concrete() {
    // Pinned to a fresh-journal Java oracle run.
    let _journal = crate::journal::scope_lore_complete(false);
    let report = report(false);
    let shop = &floor(&report, 6).items;
    for class_name in [
        "Spear",
        "FishingSpear",
        "WandOfLivingEarth",
        "MagicalHolster",
    ] {
        assert!(has(shop, class_name), "{class_name} missing from shop");
    }
    let statue = shop
        .iter()
        .find(|item| item.src.as_deref() == Some("StatueRoom"))
        .expect("statue weapon");
    assert_eq!(statue.item, "RoundShield");
    assert_eq!(statue.ench.as_deref(), Some("Eldritch"));

    let ghost = &floor(&report, 2).quests[0];
    assert_eq!(ghost.quest, "ghost");
    let hand_axe = &ghost.rewards[0];
    assert_eq!((hand_axe.item.as_str(), hand_axe.lvl), ("HandAxe", 1));
    assert_eq!(hand_axe.ench, None, "Vorpal needs Parchment Scrap");

    let wandmaker = &floor(&report, 8).quests[0];
    let wands: Vec<_> = wandmaker
        .rewards
        .iter()
        .map(|item| (item.item.as_str(), item.lvl))
        .collect();
    assert_eq!(wands, [("WandOfWarding", 1), ("WandOfRegrowth", 3)]);

    let imp = floor(&report, 18)
        .quests
        .iter()
        .find(|quest| quest.quest == "imp")
        .expect("imp quest");
    assert_eq!(imp.rewards.len(), 6);
    assert!(floor(&report, 9)
        .quests
        .iter()
        .all(|q| q.rewards.is_empty()));
}

#[test]
fn room_pulled_queue_items_are_not_double_counted() {
    // HallsLevel queues two Torches; one RitualRoom pulls one as its prize.
    let report = analyze_seed_exact("IXF-QZT-QPP", 21, ExactOptions::default()).expect("analyze");
    let items = &floor(&report, 21).items;
    let torches: i32 = items
        .iter()
        .filter(|item| item.item == "Torch")
        .map(|item| item.qty)
        .sum();
    assert_eq!(torches, 2);
    assert!(items
        .iter()
        .any(|item| item.item == "Torch" && item.src.as_deref() == Some("RitualRoom")));
}

#[test]
fn public_exact_items_appear_in_exact_report() {
    for seed in [
        SEED,
        "RZN-LKU-EFS",
        "GFX-PZH-DCH",
        "AAA-AAA-AAA",
        "ZZZ-QWE-RTY",
    ] {
        assert_public_exact_items_present(seed);
    }
}

fn assert_public_exact_items_present(seed: &str) {
    let exact = analyze_seed_exact(seed, 26, ExactOptions::default()).expect("analyze exact");
    let public = crate::analyze_seed(seed, 26).expect("analyze");
    for public_floor in &public.floors {
        let floor = floor(&exact, public_floor.depth);
        let rewards: Vec<_> = floor.quests.iter().flat_map(|q| &q.rewards).collect();
        for entry in public_floor.item_variants() {
            if entry.prediction != ItemPredictionKind::Exact || !entry.spawn_conditions.is_empty() {
                continue;
            }
            let Some(class_name) = entry.class_name() else {
                continue;
            };
            assert!(
                has(&floor.items, class_name) || rewards.iter().any(|r| r.item == class_name),
                "{seed} depth {}: {class_name} ({:?}) missing",
                public_floor.depth,
                entry.source
            );
        }
    }
}

#[test]
fn compact_map_matches_public_layout() {
    let exact = report(true);
    let public = crate::analyze_seed(SEED, 4).expect("analyze");
    for public_floor in &public.floors {
        let Some(public_map) = &public_floor.map else {
            continue;
        };
        let map = floor(&exact, public_floor.depth)
            .map
            .as_ref()
            .expect("exact map");
        assert_eq!(
            (map.width, map.height),
            (public_map.width, public_map.height)
        );
        assert_eq!(decode_rle(&map.tiles), public_map.tiles);
        assert_eq!(map.traps.len(), public_map.traps.len());
    }
}

#[test]
fn rle_round_trips() {
    let tiles = [4, 4, 4, 1, 2, 2, 4];
    assert_eq!(encode_rle(&tiles), "4*3,1,2*2,4");
    assert_eq!(decode_rle(&encode_rle(&tiles)), tiles);
    assert_eq!(encode_rle(&[]), "");
}
