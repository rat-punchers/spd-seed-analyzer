use super::{ItemEntry, ItemGroup, ItemPredictionKind};

pub(crate) fn group_item_entries(entries: Vec<ItemEntry>) -> Vec<ItemGroup> {
    let mut groups: Vec<ItemGroup> = Vec::with_capacity(entries.len());
    for mut entry in entries {
        entry.finalize_label();
        let paired_prediction = matches!(
            entry.prediction,
            ItemPredictionKind::Constrained | ItemPredictionKind::Variable
        );
        let entry_room = room_group_key(entry.source.as_deref()).map(str::to_string);
        let pair_index = (paired_prediction && entry_room.is_some()).then(|| {
            let entry_room = entry_room.as_deref();
            let matches_group = |group: &ItemGroup, loose_category: bool| {
                if room_group_key(group.source.as_deref()) != entry_room {
                    return false;
                }
                let multi_sample_contract = entry_room == Some("CrystalVaultRoom");
                if multi_sample_contract {
                    // One vault contract collects every default-profile prize.
                    let has_constrained = group
                        .variants
                        .iter()
                        .any(|variant| variant.prediction == ItemPredictionKind::Constrained);
                    return match entry.prediction {
                        ItemPredictionKind::Variable => has_constrained,
                        ItemPredictionKind::Constrained => {
                            !has_constrained
                                && group.variants.iter().any(|variant| {
                                    variant.prediction == ItemPredictionKind::Variable
                                })
                        }
                        ItemPredictionKind::Exact => false,
                    };
                }
                // One contract <-> one default-profile sample. Never join Exact.
                if group.variants.len() != 1 {
                    return false;
                }
                let other = &group.variants[0];
                let special = matches!(
                    entry_room,
                    Some(
                        "PoolRoom:equipment"
                            | "SuspiciousChestRoom:mimic_reward"
                            | "CrystalChoiceRoom:hidden_reward"
                            | "GrassyGraveRoom:prize"
                    )
                );
                let category_ok = other.category == entry.category
                    || special
                    || (loose_category
                        && (other.category == "other"
                            || entry.category == "other"
                            || other.category.contains('/')
                            || entry.category.contains('/')));
                category_ok
                    && matches!(
                        (other.prediction, entry.prediction),
                        (
                            ItemPredictionKind::Constrained,
                            ItemPredictionKind::Variable
                        ) | (
                            ItemPredictionKind::Variable,
                            ItemPredictionKind::Constrained
                        )
                    )
            };
            // Prefer exact category (e.g. gold piles) before slash/other contracts.
            groups
                .iter()
                .position(|group| matches_group(group, false))
                .or_else(|| groups.iter().position(|group| matches_group(group, true)))
        });
        if let Some(Some(index)) = pair_index {
            let group = &mut groups[index];
            // Prefer the bare room name once samples join a contract.
            if let Some(room) = entry_room.as_deref() {
                entry.source = Some(room.to_string());
                group.source = Some(room.to_string());
            }
            group.variants.push(entry);
            group
                .variants
                .sort_by_key(|variant| variant.prediction == ItemPredictionKind::Variable);
        } else {
            groups.push(ItemGroup::single(entry));
        }
    }
    groups
}

/// Pair constrained contracts with variable samples from the same spawn family.
/// Shop and quest sources stay ungrouped: each stub/sample is its own spawn.
fn room_group_key(source: Option<&str>) -> Option<&str> {
    let source = source?;
    if source.contains("ShopRoom") || source.contains("Quest") {
        return None;
    }
    if source == "guaranteed floor spawn"
        || source.ends_with(":forced")
        || source == "SuspiciousChestRoom:mimic"
    {
        return Some(source);
    }
    // Each mimic carries several items. They are separate spawns, not
    // alternate projections of one room contract.
    if source.ends_with(":mimic")
        || source.ends_with(":golden_mimic")
        || source.ends_with(":ebony_mimic")
        || source.ends_with(":crystal_mimic")
    {
        return None;
    }
    // Keep distinct contracts that share a room but describe different spawns.
    match source {
        "PoolRoom:equipment"
        | "SuspiciousChestRoom:gold"
        | "SuspiciousChestRoom:mimic_reward"
        | "CrystalChoiceRoom:hidden_reward"
        | "SecretHoneypotRoom:bomb"
        | "GrassyGraveRoom:prize"
        | "GrassyGraveRoom:gold_tombs" => Some(source),
        other if other.contains("Room") => Some(other.split(':').next().unwrap_or(other)),
        _ => None,
    }
}
