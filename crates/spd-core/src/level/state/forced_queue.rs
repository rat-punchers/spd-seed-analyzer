//! Seed-only projection of the exact initial `itemsToSpawn` queue.

use crate::items::model::{ForcedDropRole, GeneratedItem, ItemCategory, ItemProvenance};
use crate::report::{ItemDependencyCondition, ItemEntry, ItemPredictionKind, ItemSpawnCondition};
use crate::trinkets::Challenge;

fn constrained_spawn_entry(name: &str, class_name: &str, category: &str) -> ItemEntry {
    ItemEntry {
        name: name.into(),
        quantity: 1,
        quantity_range: None,
        label: None,
        item: Some(crate::report::item_id(class_name)),
        candidates: Vec::new(),
        category: category.into(),
        tier: None,
        tier_range: None,
        level: None,
        level_range: None,
        cursed: None,
        enchantment: None,
        prediction: ItemPredictionKind::Constrained,
        spawn_conditions: vec![ItemSpawnCondition {
            all_of: vec![ItemDependencyCondition::Challenge {
                challenge: Challenge::ForbiddenRunes,
                enabled: false,
            }],
        }],
        conditions: Vec::new(),
        notes: Vec::new(),
        source: Some("guaranteed floor spawn".into()),
    }
}

fn guaranteed_spawn_entry(name: &str, class_name: &str, category: &str) -> ItemEntry {
    ItemEntry {
        name: name.into(),
        quantity: 1,
        quantity_range: None,
        label: None,
        item: Some(crate::report::item_id(class_name)),
        candidates: Vec::new(),
        category: category.into(),
        tier: None,
        tier_range: None,
        level: Some(0),
        level_range: None,
        cursed: None,
        enchantment: None,
        prediction: ItemPredictionKind::Exact,
        spawn_conditions: Vec::new(),
        conditions: Vec::new(),
        notes: Vec::new(),
        source: Some("guaranteed floor spawn".into()),
    }
}

fn food_name(class_name: &str) -> &str {
    match class_name {
        "Food" => "ration of food",
        "Pasty" => "pasty",
        "MysteryMeat" => "mystery meat",
        _ => "food",
    }
}

pub(super) fn public_entries(depth: i32, initial: &[GeneratedItem]) -> Vec<ItemEntry> {
    if !matches!(depth, 1..=4 | 6..=9 | 11..=14 | 16..=19 | 21..=24) {
        return Vec::new();
    }
    let mut entries = if depth == 1 {
        initial
            .iter()
            .find(|item| item.provenance == ItemProvenance::Forced(ForcedDropRole::BaseFood))
            .map(|item| {
                vec![guaranteed_spawn_entry(
                    food_name(&item.class_name),
                    &item.class_name,
                    "food",
                )]
            })
            .unwrap_or_default()
    } else {
        vec![ItemEntry {
            name: "food".into(),
            quantity: 1,
            quantity_range: None,
            label: None,
            item: None,
            candidates: Vec::new(),
            category: "food".into(),
            tier: None,
            tier_range: None,
            level: None,
            level_range: None,
            cursed: None,
            enchantment: None,
            prediction: ItemPredictionKind::Constrained,
            spawn_conditions: Vec::new(),
            conditions: Vec::new(),
            notes: Vec::new(),
            source: Some("guaranteed floor spawn".into()),
        }]
    };
    if initial
        .iter()
        .any(|item| item.provenance == ItemProvenance::Forced(ForcedDropRole::HallsTorch))
    {
        let mut torches = guaranteed_spawn_entry("Torch", "Torch", "other");
        torches.quantity = initial
            .iter()
            .filter(|item| item.provenance == ItemProvenance::Forced(ForcedDropRole::HallsTorch))
            .map(|item| item.quantity)
            .sum();
        entries.insert(0, torches);
    }
    if initial
        .iter()
        .any(|item| item.provenance == ItemProvenance::Forced(ForcedDropRole::LargeFeelingFood))
    {
        entries.push(ItemEntry {
            name: "food".into(),
            quantity: 1,
            quantity_range: None,
            label: None,
            item: None,
            candidates: Vec::new(),
            category: "food".into(),
            tier: None,
            tier_range: None,
            level: None,
            level_range: None,
            cursed: None,
            enchantment: None,
            prediction: ItemPredictionKind::Constrained,
            spawn_conditions: Vec::new(),
            conditions: Vec::new(),
            notes: Vec::new(),
            source: Some("guaranteed floor spawn".into()),
        });
    }
    for item in initial {
        let (name, class_name) = match item.provenance {
            ItemProvenance::Forced(ForcedDropRole::HallsTorch) => continue,
            ItemProvenance::Forced(ForcedDropRole::StrengthPotion) => {
                ("Potion of Strength", "PotionOfStrength")
            }
            ItemProvenance::Forced(ForcedDropRole::UpgradeScroll {
                forbidden_runes_sensitive: false,
            }) => ("Scroll of Upgrade", "ScrollOfUpgrade"),
            ItemProvenance::Forced(ForcedDropRole::UpgradeScroll {
                forbidden_runes_sensitive: true,
            }) => {
                entries.push(constrained_spawn_entry(
                    "Scroll of Upgrade",
                    "ScrollOfUpgrade",
                    "scroll",
                ));
                continue;
            }
            ItemProvenance::Forced(ForcedDropRole::ArcaneStylus) => ("Arcane Stylus", "Stylus"),
            ItemProvenance::Forced(ForcedDropRole::EnchantmentStone) => {
                ("Stone of Enchantment", "StoneOfEnchantment")
            }
            ItemProvenance::Forced(ForcedDropRole::IntuitionStone) => {
                ("Stone of Intuition", "StoneOfIntuition")
            }
            ItemProvenance::Forced(ForcedDropRole::TrinketCatalyst) => {
                let mut catalyst =
                    guaranteed_spawn_entry("Trinket Catalyst", "TrinketCatalyst", "other");
                catalyst.candidates = crate::report::item_ids(&item.candidate_classes);
                entries.push(catalyst);
                continue;
            }
            _ => continue,
        };
        entries.push(guaranteed_spawn_entry(
            name,
            class_name,
            &format!("{:?}", item.category).to_ascii_lowercase(),
        ));
    }
    entries
}

/// Room source of a queued guaranteed spawn that a room's `findPrizeItem`
/// pulled, e.g. `RingRoom:forced`.
pub(super) fn relocating_room(item: &GeneratedItem) -> Option<&str> {
    let source = item.source.as_deref()?;
    let room = source.split(':').next()?;
    (item.from_spawn_queue && room.ends_with("Room")).then_some(source)
}

/// Publish a relocated guaranteed spawn at its room so the floor list keeps
/// the item's location. Returns the relocated non-food entries: unlike food,
/// they have no variable sample of their own.
pub(super) fn locate_in_rooms<'a>(
    entries: &mut [ItemEntry],
    prizes: impl IntoIterator<Item = &'a GeneratedItem>,
) -> Vec<ItemEntry> {
    let mut relocated = Vec::new();
    for prize in prizes {
        let Some(room) = relocating_room(prize) else {
            continue;
        };
        let entry = entries.iter_mut().find(|entry| {
            entry.source.as_deref() == Some("guaranteed floor spawn")
                && match entry.class_name() {
                    Some(class_name) => class_name == prize.class_name,
                    None => entry.category == "food" && prize.category == ItemCategory::Food,
                }
        });
        if let Some(entry) = entry {
            entry.source = Some(room.into());
            if prize.category != ItemCategory::Food {
                relocated.push(entry.clone());
            }
        }
    }
    relocated
}
