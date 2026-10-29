//! Drop-cell placement burns and `findPrizeItem` helpers shared by room painters.

use crate::items::model::{GeneratedItem, ItemCategory};
use crate::random::Random;
use crate::rooms::room::Room;

/// Consume Room.random()-style placement until unique cell (cap tries).
pub(super) fn burn_drop_pos(room: &Room, occupied: &mut Vec<(i32, i32)>) {
    burn_drop_pos_margin(room, 1, occupied);
}

/// `Room.random(m)` placement until unique cell.
pub(super) fn burn_drop_pos_margin(room: &Room, m: i32, occupied: &mut Vec<(i32, i32)>) {
    if room.width() <= 2 * m || room.height() <= 2 * m {
        return;
    }
    for _ in 0..50 {
        let x = Random::int_range_inclusive(room.left + m, room.right - m);
        let y = Random::int_range_inclusive(room.top + m, room.bottom - m);
        if !occupied.contains(&(x, y)) {
            occupied.push((x, y));
            return;
        }
    }
}

#[cfg(test)]
pub(super) fn burn_terrain_pos(room: &Room, _water: bool) {
    // simplified: consume the same RNG shape as a single accepted roll
    // (full map terrain not painted yet)
    let _x = Random::int_range_inclusive(room.left + 1, room.right - 1);
    let _y = Random::int_range_inclusive(room.top + 1, room.bottom - 1);
}

pub(super) fn find_prize_item(
    items_to_spawn: &mut Vec<GeneratedItem>,
    match_class: Option<&str>,
) -> Option<GeneratedItem> {
    if items_to_spawn.is_empty() {
        return None;
    }
    let mut pulled = match match_class {
        None => {
            // prefer TrinketCatalyst
            if let Some(i) = items_to_spawn
                .iter()
                .position(|it| it.class_name == "TrinketCatalyst")
            {
                items_to_spawn.remove(i)
            } else {
                let idx = Random::int_max(items_to_spawn.len() as i32) as usize;
                items_to_spawn.remove(idx)
            }
        }
        Some(class) => {
            let i = items_to_spawn
                .iter()
                .position(|it| it.class_name == class)?;
            items_to_spawn.remove(i)
        }
    };
    pulled.from_spawn_queue = true;
    Some(pulled)
}

pub(super) fn find_prize_item_category(
    items_to_spawn: &mut Vec<GeneratedItem>,
    cat: ItemCategory,
) -> Option<GeneratedItem> {
    let i = items_to_spawn.iter().position(|it| it.category == cat)?;
    let mut pulled = items_to_spawn.remove(i);
    pulled.from_spawn_queue = true;
    Some(pulled)
}
