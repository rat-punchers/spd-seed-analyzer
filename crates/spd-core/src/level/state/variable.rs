use std::collections::HashMap;

use super::conditions::{item_conditions, item_conditions_typed};
use super::forced_queue::relocating_room;
use super::{is_blacklisted, is_unpublished_main_loot, sources::reported_source, LevelState};
use crate::items::model::{
    ForcedDropRole, GeneratedItem, ItemCategory, ItemProvenance, QuestRewardRole, RoomLootRole,
    ShopStockRole,
};
use crate::report::{ItemEnchantment, ItemEntry, ItemPredictionKind};

const VARIES_NOTE: &str =
    "Varies with gameplay; player-controlled generation history can change this result.";

pub(super) fn collect_samples(state: &LevelState) -> Vec<ItemEntry> {
    if !state.expose_variable_items {
        return Vec::new();
    }
    let mut out = Vec::new();
    if state.depth > 1 {
        for item in &state.spawned_queue_items {
            if matches!(
                item.provenance,
                ItemProvenance::Forced(ForcedDropRole::BaseFood | ForcedDropRole::LargeFeelingFood)
            ) {
                out.push(food_entry(item, "guaranteed floor spawn"));
            }
        }
        for item in &state.placed_items {
            if let Some(room) = relocating_room(item) {
                if item.category == ItemCategory::Food {
                    out.push(food_entry(item, room));
                }
            }
        }
    }
    for (index, item) in state.placed_items.iter().enumerate() {
        // A room prize pulled from the spawn queue is published as its
        // guaranteed floor spawn, located at the room.
        if relocating_room(item).is_some() {
            continue;
        }
        let past_boundary = state
            .runtime_sensitive_placed_items_from
            .is_some_and(|boundary| index >= boundary);
        let Some(entry) = item_entry(item, past_boundary) else {
            continue;
        };
        // Compact/search stay selective; also publish room/shop/floor samples
        // that keep the floor list aligned with the floor map, and every
        // non-gold item a mimic carries.
        let keep = (crate::report::is_variable_highlight(&entry)
            || is_floor_uncover_sample(&entry)
            || is_mimic_carried_source(entry.source.as_deref())
            || is_floor_loot_source(entry.source.as_deref()))
            && (state.depth > 1
                || !matches!(
                    entry.source.as_deref(),
                    Some(
                        "PoolRoom:equipment"
                            | "SuspiciousChestRoom:gold"
                            | "SuspiciousChestRoom:mimic_reward"
                    )
                ));
        if keep {
            out.push(entry);
        }
    }
    out
}

/// Append variable samples that are not already covered by `items` (such as
/// exact room contracts like Double Bomb, Honeypot, or Energy Crystals).
pub(super) fn append_unprojected_samples(items: &mut Vec<ItemEntry>, samples: &[ItemEntry]) {
    let mut remaining = HashMap::<(spd_objects::ItemId, Option<String>), i32>::new();
    for entry in items.iter() {
        if is_room_spawn_queue_potion(entry) {
            continue;
        }
        let Some(item_id) = entry.item else {
            continue;
        };
        let key = (
            item_id,
            room_family(entry.source.as_deref()).map(str::to_string),
        );
        let covered = entry
            .quantity_range
            .map_or(entry.quantity.max(1), |range| range.max.max(entry.quantity));
        *remaining.entry(key).or_default() += covered;
    }

    for sample in samples {
        if let Some(item_id) = sample.item {
            let key = (
                item_id,
                room_family(sample.source.as_deref()).map(str::to_string),
            );
            if let Some(covered) = remaining.get_mut(&key) {
                if *covered >= sample.quantity {
                    *covered -= sample.quantity;
                    continue;
                }
                if *covered > 0 {
                    let mut remainder = sample.clone();
                    remainder.quantity -= *covered;
                    *covered = 0;
                    items.push(remainder);
                    continue;
                }
            }
        }
        items.push(sample.clone());
    }
}

fn food_entry(item: &GeneratedItem, source: &str) -> ItemEntry {
    let name = match item.class_name.as_str() {
        "Food" => "ration of food",
        "Pasty" => "pasty",
        "MysteryMeat" => "mystery meat",
        _ => "food",
    };
    ItemEntry {
        name: name.into(),
        quantity: item.quantity.max(1),
        quantity_range: None,
        label: None,
        item: Some(crate::report::item_id(&item.class_name)),
        candidates: Vec::new(),
        category: "food".into(),
        tier: None,
        tier_range: None,
        level: Some(0),
        level_range: None,
        cursed: Some(false),
        enchantment: None,
        prediction: ItemPredictionKind::Variable,
        spawn_conditions: Vec::new(),
        conditions: Vec::new(),
        notes: vec![VARIES_NOTE.into()],
        source: Some(source.into()),
    }
}

/// Project one concrete result that varies with gameplay. It is the item this
/// seed rolls on the active profile; player choices can shift it, so it is
/// never a universal seed claim.
fn item_entry(item: &GeneratedItem, past_runtime_sensitive_boundary: bool) -> Option<ItemEntry> {
    let room_variable = matches!(
        item.provenance,
        ItemProvenance::Room(
            RoomLootRole::PoolEquipment
                | RoomLootRole::SuspiciousChestGold
                | RoomLootRole::SuspiciousChestMimicReward
                | RoomLootRole::CrystalChoiceHidden
                | RoomLootRole::CrystalVaultPrize
                | RoomLootRole::SecretHoneypotBomb
                | RoomLootRole::GrassyGravePrize
                | RoomLootRole::GrassyGraveGold
                | RoomLootRole::RuntimeSensitive
                | RoomLootRole::CryptArmor
                | RoomLootRole::StatueWeapon
        )
    );
    let quest_variable = matches!(
        item.provenance,
        ItemProvenance::Quest(
            QuestRewardRole::GhostWeapon { .. }
                | QuestRewardRole::WandmakerWand
                | QuestRewardRole::ImpVaultOption { .. }
                | QuestRewardRole::BlacksmithWeapon { .. }
                | QuestRewardRole::BlacksmithMissile { .. }
                | QuestRewardRole::BlacksmithArmor { .. }
                | QuestRewardRole::BlacksmithRoomWeapon { .. }
                | QuestRewardRole::BlacksmithRoomMissile { .. }
                | QuestRewardRole::BlacksmithRoomArmor { .. }
        )
    );
    let shop_variable = matches!(
        item.provenance,
        ItemProvenance::Shop(
            ShopStockRole::DeckWeapon { .. }
                | ShopStockRole::DeckMissile { .. }
                | ShopStockRole::ChooseBag
                | ShopStockRole::DeckRareWand
                | ShopStockRole::DeckRareRing
                | ShopStockRole::DeckRareArtifactOrRing
        )
    );
    // Main-loop and treasury mimics carry seed-generated prizes. Room prizes
    // that already have their own provenance stay on that path.
    let mimic_drop = !room_variable && is_mimic_carried_source(item.source.as_deref());
    let floor_loot = !room_variable
        && !mimic_drop
        && past_runtime_sensitive_boundary
        && matches!(item.provenance, ItemProvenance::None)
        && is_floor_loot_source(item.source.as_deref());
    if item.source.is_none()
        || !(room_variable || quest_variable || shop_variable || mimic_drop || floor_loot)
        || (is_blacklisted(item) && !room_variable)
        || (is_unpublished_main_loot(item) && !room_variable && !mimic_drop)
        || item.provenance == ItemProvenance::Quest(QuestRewardRole::WandmakerPersisted)
    {
        return None;
    }
    let quest_role = match item.provenance {
        ItemProvenance::Quest(role) => Some(role),
        _ => None,
    };
    // Publishing an internal category list on a concrete variable item makes
    // shared renderers present every class at the sample's exact level, which
    // is not a sound cross-profile claim. Imp still carries that list internally.
    let candidate_classes = if matches!(
        quest_role,
        Some(QuestRewardRole::WandmakerWand | QuestRewardRole::ImpVaultOption { .. })
    ) {
        Vec::new()
    } else {
        item.candidate_classes.clone()
    };
    let artifact_conditional = (item.artifact_conditional
        && item
            .source
            .as_deref()
            .is_some_and(|source| source.rsplit(':').next() == Some("heap")))
        || matches!(
            quest_role,
            Some(QuestRewardRole::ImpVaultOption { slot: 0 })
        );
    let title = item.title();
    let source = match item.provenance {
        ItemProvenance::Room(RoomLootRole::PoolEquipment) => Some("PoolRoom:equipment".into()),
        ItemProvenance::Room(RoomLootRole::SuspiciousChestGold) => {
            Some("SuspiciousChestRoom:gold".into())
        }
        ItemProvenance::Room(RoomLootRole::SuspiciousChestMimicReward) => {
            Some("SuspiciousChestRoom:mimic_reward".into())
        }
        _ => reported_source(item, false),
    };
    let mut public_name: String = if item.category == ItemCategory::Gold {
        "gold".into()
    } else {
        title.strip_prefix("cursed ").unwrap_or(&title).into()
    };
    if room_variable
        && item.level == 0
        && matches!(
            item.category,
            ItemCategory::Weapon | ItemCategory::Armor | ItemCategory::Missile | ItemCategory::Ring
        )
    {
        public_name.push_str(" +0");
    }
    Some(ItemEntry {
        name: public_name,
        quantity: item.quantity.max(1),
        quantity_range: None,
        label: None,
        item: Some(crate::report::item_id(&item.class_name)),
        candidates: crate::report::item_ids(&candidate_classes),
        category: format!("{:?}", item.category).to_ascii_lowercase(),
        tier: crate::generator::weapon_tier_for_class(&item.class_name),
        tier_range: None,
        level: Some(item.level),
        level_range: None,
        cursed: Some(item.cursed),
        enchantment: item
            .enchantment
            .clone()
            .map(|enchantment_type| ItemEnchantment {
                enchantment_type: spd_objects::Enchantment::from_class_name_lossy(
                    &enchantment_type,
                ),
                conditions: Vec::new(),
            }),
        prediction: ItemPredictionKind::Variable,
        spawn_conditions: item_conditions(artifact_conditional),
        conditions: item_conditions_typed(quest_role, false),
        notes: vec![VARIES_NOTE.into()],
        source,
    })
}

/// `mimic`, `golden_mimic`, and `ebony_mimic` sources, including `mimic:mimic`.
pub(super) fn is_mimic_carried_source(source: Option<&str>) -> bool {
    source.is_some_and(|source| {
        matches!(
            source.rsplit(':').next(),
            Some("mimic" | "golden_mimic" | "ebony_mimic")
        )
    })
}

fn is_floor_loot_source(source: Option<&str>) -> bool {
    let Some(source) = source else {
        return false;
    };
    source.rsplit(':').next() == Some("heap")
        || matches!(
            source,
            crate::level::create_items::SPYGLASS_SOURCE
                | crate::level::create_items::DARKNESS_TORCH_SOURCE
                | "MassGraveRoom"
        )
}

fn room_family(source: Option<&str>) -> Option<&str> {
    source.map(|source| source.split(':').next().unwrap_or(source))
}

/// Exact room-contract potions that are placed via `itemsToSpawn` (`spawned_queue_items`),
/// not inside `placed_items`. A room prize that also rolls the same potion class
/// is a second bottle on the map and must not be suppressed.
fn is_room_spawn_queue_potion(entry: &ItemEntry) -> bool {
    if entry.prediction != ItemPredictionKind::Exact {
        return false;
    }
    matches!(
        (entry.source.as_deref(), entry.class_name()),
        (Some("PoolRoom"), Some("PotionOfInvisibility"))
            | (Some("SentryRoom"), Some("PotionOfHaste"))
            | (
                Some("TrapsRoom" | "SecretChestChasmRoom"),
                Some("PotionOfLevitation")
            )
            | (
                Some("StorageRoom" | "SecretRunestoneRoom" | "MassGraveRoom"),
                Some("PotionOfLiquidFlame")
            )
            | (Some("MagicalFireRoom"), Some("PotionOfFrost"))
            | (Some("ToxicGasRoom"), Some("PotionOfPurity"))
    )
}

/// Default-profile room/shop samples that clear blank constrained stubs.
fn is_floor_uncover_sample(item: &ItemEntry) -> bool {
    let Some(source) = item.source.as_deref() else {
        return false;
    };
    if source.contains("ShopRoom") {
        return item.item.is_some();
    }
    let room = source.split(':').next().unwrap_or(source);
    matches!(
        room,
        "RingRoom"
            | "BlacksmithRoom"
            | "ArmoryRoom"
            | "StudyRoom"
            | "RitualRoom"
            | "CryptRoom"
            | "StatueRoom"
            | "SacrificeRoom"
            | "CrystalVaultRoom"
            | "CrystalChoiceRoom"
            | "CrystalPathRoom"
            | "LaboratoryRoom"
            | "LibraryRoom"
            | "TreasuryRoom"
            | "StorageRoom"
            | "RunestoneRoom"
            | "MagicalFireRoom"
            | "ToxicGasRoom"
            | "PitRoom"
            | "SentryRoom"
            | "TrapsRoom"
            | "PoolRoom"
            | "SuspiciousChestRoom"
            | "GrassyGraveRoom"
            | "SecretLibraryRoom"
            | "SecretRunestoneRoom"
            | "SecretArtilleryRoom"
            | "SecretLaboratoryRoom"
            | "SecretLarderRoom"
            | "SecretHoardRoom"
            | "SecretGardenRoom"
            | "SecretMazeRoom"
            | "SecretSummoningRoom"
            | "SecretChestChasmRoom"
    ) && item.item.is_some()
}
