//! Standard special-room prize painters (Crypt, Armory, Library, …).

mod consumable;
mod equip;
mod laboratory;

#[cfg(test)]
pub(super) use consumable::runestone_prizes;
pub(super) use consumable::{
    library_prizes, runestone_prizes_on_map, storage_prize, storage_prizes, treasury_prizes_on_map,
};
#[cfg(test)]
pub(super) use equip::pool_prize;
pub(super) use equip::{
    armory_prizes_on_map, bomb_random, crypt_prize, pool_prize_on_map, statue_prizes,
};
pub(super) use laboratory::paint as paint_laboratory;

use crate::items::model::GeneratedItem;

pub(super) fn is_curse_enchant(item: &GeneratedItem) -> bool {
    match item.enchantment.as_deref() {
        Some(e) => matches!(
            e,
            "Annoying"
                | "Displacing"
                | "Dazzling"
                | "Explosive"
                | "Friendly"
                | "Polarized"
                | "Pressurized"
                | "Sacrificial"
                | "Wayward"
                | "Wondrous"
                | "AntiEntropy"
                | "Corrosion"
                | "Displacement"
                | "Metabolism"
                | "Multiplicity"
                | "Stench"
                | "Overgrowth"
                | "Bulk"
        ),
        None => false,
    }
}

pub(super) fn is_good_glyph(item: &GeneratedItem) -> bool {
    match item.enchantment.as_deref() {
        Some(e) => !matches!(
            e,
            "AntiEntropy"
                | "Corrosion"
                | "Displacement"
                | "Metabolism"
                | "Multiplicity"
                | "Stench"
                | "Overgrowth"
                | "Bulk"
        ),
        None => false,
    }
}
