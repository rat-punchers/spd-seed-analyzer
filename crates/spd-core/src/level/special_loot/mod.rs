//! Special / secret room prize generation (partial paint loot).
//!
//! Called after minimal geometry paint and **before** `createItems`, matching
//! RegularPainter room paint order (shuffle rooms → placeDoors → room prizes).
//! Returns door map + paint order for subsequent `paintDoors`.

mod crystal;
mod crystal_path;
mod gardens;
mod geometry;
mod pit_secrets;
mod placement;
mod quest_rooms;
mod secret_rooms;
mod shop_room;
mod special_rooms;
mod standard_rooms;
mod trap_rooms;

#[cfg(test)]
mod tests;

use crate::dungeon::DungeonState;
use crate::items::model::{GeneratedItem, ItemProvenance, RoomLootRole};
use crate::level::create_items::PlacedLoot;
use crate::level::painter::{
    apply_room_door_types, paint_connection_room, paint_sewer_boss_room, paint_standard_room,
    place_doors_for_room, DoorMap,
};
use crate::level::terrain::TerrainMap;
use crate::level::Feeling;
use crate::random::Random;
use crate::rooms::room::Room;
use crate::rooms::types::RoomKind;
use crate::trinkets::Challenge;

/// Result of special-room paint pass (prizes + doors for `paintDoors`).
pub struct SpecialPaintResult {
    pub loot: Vec<PlacedLoot>,
    pub doors: DoorMap,
    /// Room indices in RegularPainter shuffle order (for `paintDoors` iteration).
    pub paint_order: Vec<usize>,
    /// Number of portable room items painted before the first callback whose
    /// RNG path can differ with player state/history.
    pub first_sensitive_loot_index: Option<usize>,
    pub room_public_facts: Vec<super::room_public::RoomPublicFact>,
    /// Non-consuming main-RNG checkpoints after each `Room.paint` callback.
    #[doc(hidden)]
    pub rng_checkpoints: Vec<RoomPaintRngCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomPaintRngCheckpoint {
    pub room: String,
    pub rng: Vec<i32>,
}

/// Generate special/secret room prizes; may consume items from `items_to_spawn`
/// via `findPrizeItem` (TrinketCatalyst, forced potions, etc.).
pub fn special_room_loot(
    dungeon: &mut DungeonState,
    rooms: &[Room],
    map: &mut TerrainMap,
    items_to_spawn: &mut Vec<GeneratedItem>,
    shop_items: &[GeneratedItem],
    feeling: Feeling,
) -> SpecialPaintResult {
    let mut out = Vec::new();
    let mut doors = DoorMap::new();
    if rooms.is_empty() {
        return SpecialPaintResult {
            loot: out,
            doors,
            paint_order: Vec::new(),
            first_sensitive_loot_index: None,
            room_public_facts: Vec::new(),
            rng_checkpoints: Vec::new(),
        };
    }

    let mut order: Vec<usize> = (0..rooms.len()).collect();
    // RegularPainter.paint shuffles its ArrayList with Collections.shuffle.
    Random::shuffle_list(&mut order);

    let mut first_sensitive_loot_index = None;
    let mut room_public_facts = Vec::new();
    let mut rng_checkpoints = Vec::new();
    for &ri in &order {
        place_doors_for_room(rooms, ri, &mut doors);

        let room = &rooms[ri];
        if first_sensitive_loot_index.is_none() && callback_tail_sensitive(room) {
            first_sensitive_loot_index = Some(
                out.iter()
                    .filter(|placed| placed.heap_type != "plant")
                    .count(),
            );
        }
        if room.is_empty() {
            continue;
        }
        // Room.paint door-type upgrades (LOCKED / HIDDEN / REGULAR / …).
        apply_room_door_types(rooms, ri, &mut doors);
        if room.kind == RoomKind::Connection {
            paint_connection_room(map, rooms, ri, &doors, feeling == Feeling::Chasm);
        }
        let standard_paint = if paint_sewer_boss_room(map, room, ri, &doors) {
            Default::default()
        } else {
            paint_standard_room(
                map,
                rooms,
                room,
                ri,
                &doors,
                &mut dungeon.generator,
                dungeon.depth,
                dungeon.challenges & Challenge::BarrenLand.mask() != 0,
            )
        };
        // `RingRoom` only calls `placeCenterDetail` when both dimensions are
        // at least 10. Smaller instances are ring-shaped geometry with no
        // reward callback, so they must not emit an item contract.
        let mut fact = None;
        if room.name != "RingRoom" || standard_paint.center_loot.is_some() {
            fact = super::room_public::RoomPublicFact::for_room(room, dungeon.depth);
        }
        let mut standard_loot = standard_rooms::paint_center_loot(
            dungeon,
            room,
            map,
            standard_paint.center_loot,
            items_to_spawn,
        );
        out.append(&mut standard_loot);

        match room.kind {
            RoomKind::Special | RoomKind::Secret => {
                let geometry_prize_cell =
                    geometry::paint_at_depth(map, room, ri, &doors, dungeon.depth);
                let mut loot = paint_special(
                    dungeon,
                    rooms,
                    ri,
                    map,
                    &doors,
                    geometry_prize_cell,
                    items_to_spawn,
                );
                if let Some(fact) = fact.as_mut().filter(|_| {
                    room.name == "SecretLaboratoryRoom" && first_sensitive_loot_index.is_none()
                }) {
                    fact.set_energy_crystals(
                        loot.iter()
                            .filter(|placed| placed.item.class_name == "EnergyCrystal")
                            .map(|placed| placed.item.quantity)
                            .sum(),
                    );
                }
                out.append(&mut loot);
            }
            RoomKind::Standard
                if room.name == "RitualSiteRoom" || room.name == "BlacksmithRoom" =>
            {
                let geometry_prize_cell =
                    geometry::paint_at_depth(map, room, ri, &doors, dungeon.depth);
                let mut loot = paint_special(
                    dungeon,
                    rooms,
                    ri,
                    map,
                    &doors,
                    geometry_prize_cell,
                    items_to_spawn,
                );
                out.append(&mut loot);
            }
            RoomKind::Shop => {
                shop_room::paint(map, room, ri, &doors, shop_items);
            }
            _ => {}
        }
        room_public_facts.extend(fact);
        rng_checkpoints.push(RoomPaintRngCheckpoint {
            room: room.name.clone(),
            rng: Random::peek_ints(8),
        });
    }

    SpecialPaintResult {
        loot: out,
        doors,
        paint_order: order,
        first_sensitive_loot_index,
        room_public_facts,
        rng_checkpoints,
    }
}

fn paint_special(
    dungeon: &mut DungeonState,
    rooms: &[Room],
    ri: usize,
    map: &mut TerrainMap,
    doors: &DoorMap,
    geometry_prize_cell: Option<usize>,
    items_to_spawn: &mut Vec<GeneratedItem>,
) -> Vec<PlacedLoot> {
    let room = &rooms[ri];
    let name = room.name.as_str();
    let mut loot = match name {
        "CryptRoom" => {
            let prize = special_rooms::crypt_prize(dungeon, items_to_spawn);
            let cell = geometry_prize_cell.expect("CryptRoom geometry returns its tomb cell");
            map.record_heap(cell, "tomb", prize.item.clone());
            vec![prize]
        }
        "ArmoryRoom" => {
            let entrance = room
                .connected
                .iter()
                .find_map(|&other| doors.get(ri, other))
                .map(|door| crate::geom::Point::new(door.x, door.y))
                .expect("placed ArmoryRoom has an entrance");
            special_rooms::armory_prizes_on_map(dungeon, room, map, entrance, items_to_spawn)
        }
        "LibraryRoom" => special_rooms::library_prizes(dungeon, room, map, items_to_spawn),
        "TreasuryRoom" => special_rooms::treasury_prizes_on_map(dungeon, room, map, items_to_spawn),
        "PoolRoom" => {
            let prize = special_rooms::pool_prize_on_map(dungeon, room, map, items_to_spawn);
            let cell = geometry_prize_cell.expect("PoolRoom geometry returns its pedestal cell");
            map.record_heap(cell, "chest", prize.item.clone());
            vec![prize]
        }
        "StorageRoom" => special_rooms::storage_prizes(dungeon, room, map, items_to_spawn),
        "RunestoneRoom" => {
            special_rooms::runestone_prizes_on_map(dungeon, room, map, items_to_spawn)
        }
        "LaboratoryRoom" => {
            special_rooms::paint_laboratory(dungeon, room, ri, map, doors, items_to_spawn)
        }
        "StatueRoom" => {
            let entrance = room
                .connected
                .iter()
                .find_map(|&other| doors.get(ri, other))
                .map(|door| crate::geom::Point::new(door.x, door.y))
                .expect("placed StatueRoom has an entrance");
            special_rooms::statue_prizes(dungeon, room, map, entrance, items_to_spawn)
        }
        "SecretLibraryRoom" => secret_rooms::secret_library(room, map),
        "SecretRunestoneRoom" => secret_rooms::secret_runestone(dungeon, room, map, items_to_spawn),
        "SecretArtilleryRoom" => secret_rooms::secret_artillery(dungeon, room, map),
        "SecretLaboratoryRoom" => secret_rooms::secret_laboratory(room, map),
        "SecretLarderRoom" => secret_rooms::secret_larder(
            dungeon.depth,
            room,
            map,
            dungeon.challenges & Challenge::BarrenLand.mask() != 0,
        ),
        "SecretHoardRoom" => secret_rooms::secret_hoard(dungeon, room, map),
        // Garden / well / pit / remaining secrets with portable prizes
        "GardenRoom" => gardens::garden_prizes(
            room,
            map,
            items_to_spawn,
            dungeon.challenges & Challenge::BarrenLand.mask() != 0,
        ),
        "SecretGardenRoom" => gardens::secret_garden_prizes(
            room,
            map,
            dungeon.challenges & Challenge::BarrenLand.mask() != 0,
        ),
        "MagicWellRoom" => gardens::magic_well(room, map, items_to_spawn),
        "SecretWellRoom" => {
            let entrance = room
                .connected
                .first()
                .and_then(|&other| doors.get(ri, other))
                .map(|door| crate::geom::Point::new(door.x, door.y))
                .expect("SecretWellRoom has one entrance");
            gardens::secret_well(room, map, entrance)
        }
        "PitRoom" => pit_secrets::pit_prizes(dungeon, room, map),
        "SecretMazeRoom" => {
            let prize = pit_secrets::secret_maze_prize(dungeon);
            let cell = geometry_prize_cell.expect("SecretMazeRoom geometry returns a prize cell");
            map.record_heap(cell, "chest", prize.item.clone());
            vec![prize]
        }
        "SecretSummoningRoom" => {
            let prize = pit_secrets::secret_summoning_prize(dungeon);
            let cell =
                geometry_prize_cell.expect("SecretSummoningRoom geometry returns its center cell");
            map.record_heap(cell, "skeleton", prize.item.clone());
            vec![prize]
        }
        "SecretChestChasmRoom" => {
            pit_secrets::secret_chest_chasm(dungeon, room, map, items_to_spawn)
        }
        // Layout-only (no portable prize items)
        "WeakFloorRoom" | "DemonSpawnerRoom" => Vec::new(),
        "SentryRoom" => vec![trap_rooms::sentry_prize(
            dungeon,
            rooms,
            ri,
            map,
            doors,
            items_to_spawn,
        )],
        "TrapsRoom" => vec![trap_rooms::traps_prize(
            dungeon,
            rooms,
            ri,
            map,
            doors,
            items_to_spawn,
        )],
        "MagicalFireRoom" => {
            trap_rooms::magical_fire_prizes(dungeon, rooms, ri, map, doors, items_to_spawn)
        }
        "SacrificeRoom" => vec![trap_rooms::sacrifice_prize(dungeon, rooms, ri, doors)],
        "ToxicGasRoom" => {
            trap_rooms::toxic_gas_prizes(dungeon, rooms, ri, map, doors, items_to_spawn)
        }
        "SecretHoneypotRoom" => trap_rooms::secret_honeypot(rooms, ri, map, doors),
        "CrystalVaultRoom" => {
            crystal::crystal_vault(dungeon, rooms, ri, map, doors, items_to_spawn)
        }
        "CrystalChoiceRoom" => {
            crystal::crystal_choice(dungeon, rooms, ri, map, doors, items_to_spawn)
        }
        "CrystalPathRoom" => crystal_path::paint(dungeon, rooms, ri, map, doors, items_to_spawn),
        // Wandmaker quest rooms
        "MassGraveRoom" => quest_rooms::mass_grave_prizes(dungeon, room, map, items_to_spawn),
        "RitualSiteRoom" => quest_rooms::ritual_site_setup(rooms, ri, map, doors, items_to_spawn),
        "RotGardenRoom" => quest_rooms::rot_garden_setup(room, items_to_spawn),
        // Blacksmith quest room — two random equip drops + NPC / exit placement RNG
        "BlacksmithRoom" => quest_rooms::blacksmith_room_prizes(dungeon, rooms, ri, map, doors),
        // Imp quest room — NPC offset burns IntRange(-1,1); no prize items
        "AmbitiousImpRoom" => {
            quest_rooms::ambitious_imp_room_npc(rooms, ri, map, doors);
            Vec::new()
        }
        _ => Vec::new(),
    };

    if runtime_sensitive_room(name) {
        for placed in &mut loot {
            let role = match placed.item.provenance {
                ItemProvenance::Room(role) => role,
                _ => match name {
                    "CryptRoom" => RoomLootRole::CryptArmor,
                    "StatueRoom" => RoomLootRole::StatueWeapon,
                    "CrystalVaultRoom" => RoomLootRole::CrystalVaultPrize,
                    _ => RoomLootRole::RuntimeSensitive,
                },
            };
            placed.item.provenance = ItemProvenance::Room(role);
        }
        // Heap items were cloned before the common provenance pass. Tag those
        // internal clones too so map projection never relies on source text.
        for heap in map.known_heaps.iter_mut().flatten() {
            for item in &mut heap.items {
                if item
                    .source
                    .as_deref()
                    .is_some_and(|source| source.split(':').next() == Some(name))
                    && !matches!(item.provenance, ItemProvenance::Room(_))
                {
                    let role = match name {
                        "CryptRoom" => RoomLootRole::CryptArmor,
                        "StatueRoom" => RoomLootRole::StatueWeapon,
                        "CrystalVaultRoom" => RoomLootRole::CrystalVaultPrize,
                        _ => RoomLootRole::RuntimeSensitive,
                    };
                    item.provenance = ItemProvenance::Room(role);
                }
            }
        }
    }
    loot
}

fn runtime_sensitive_room(name: &str) -> bool {
    matches!(
        name,
        "CryptRoom"
            | "ArmoryRoom"
            | "LibraryRoom"
            | "TreasuryRoom"
            | "StorageRoom"
            | "RunestoneRoom"
            | "LaboratoryRoom"
            | "PoolRoom"
            | "StatueRoom"
            | "SentryRoom"
            | "TrapsRoom"
            | "MagicalFireRoom"
            | "ToxicGasRoom"
            | "CrystalVaultRoom"
            | "CrystalChoiceRoom"
            | "CrystalPathRoom"
            | "PitRoom"
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
            | "SecretHoneypotRoom"
    )
}

fn callback_tail_sensitive(room: &Room) -> bool {
    let name = room.name.as_str();
    if name == "RingRoom" && room.width().min(room.height()) < 10 {
        return false;
    }
    matches!(
        name,
        "ArmoryRoom"
            | "StudyRoom"
            | "RitualRoom"
            | "RingRoom"
            | "SuspiciousChestRoom"
            | "GrassyGraveRoom"
            | "LibraryRoom"
            | "TreasuryRoom"
            | "StorageRoom"
            | "RunestoneRoom"
            | "LaboratoryRoom"
            | "PoolRoom"
            | "StatueRoom"
            | "SentryRoom"
            | "TrapsRoom"
            | "MagicalFireRoom"
            | "ToxicGasRoom"
            | "CrystalVaultRoom"
            | "CrystalChoiceRoom"
            | "CrystalPathRoom"
            | "PitRoom"
            | "SecretSummoningRoom"
            | "SecretChestChasmRoom"
    )
}
