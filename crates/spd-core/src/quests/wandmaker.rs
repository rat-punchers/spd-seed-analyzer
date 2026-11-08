//! Port of `Wandmaker.Quest` (Old Wandmaker, prison floors 7–9).
//!
//! - `spawnRoom` runs at the end of `PrisonLevel.initRooms` (before shuffle)
//! - `spawnWandmaker` runs at the start of `PrisonLevel.createMobs`
//!
//! Quest-room terrain and contents are painted in `special_loot`.
//! NPC placement uses simplified entrance-room flags (same caveats as Ghost).

use crate::dungeon::DungeonState;
use crate::generator::Category;
use crate::items::model::GeneratedItem;
use crate::level::TerrainMap;
use crate::random::Random;
use crate::rooms::room::Room;
use crate::rooms::types::{RoomKind, RoomSpec};

#[path = "wandmaker/distribution.rs"]
mod distribution;
pub use distribution::{build_reward_slots, DEFAULT_DRIFT};

/// 1 = corpse dust, 2 = elemental embers, 3 = rotberry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WandmakerQuestType {
    CorpseDust = 1,
    ElementalEmbers = 2,
    Rotberry = 3,
}

impl WandmakerQuestType {
    pub fn as_str(self) -> &'static str {
        match self {
            WandmakerQuestType::CorpseDust => "Corpse Dust",
            WandmakerQuestType::ElementalEmbers => "Elemental Embers",
            WandmakerQuestType::Rotberry => "Rotberry",
        }
    }

    pub fn room_name(self) -> &'static str {
        match self {
            WandmakerQuestType::CorpseDust => "MassGraveRoom",
            WandmakerQuestType::ElementalEmbers => "RitualSiteRoom",
            WandmakerQuestType::Rotberry => "RotGardenRoom",
        }
    }

    fn from_int(v: i32) -> Self {
        match v {
            1 => WandmakerQuestType::CorpseDust,
            2 => WandmakerQuestType::ElementalEmbers,
            _ => WandmakerQuestType::Rotberry,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WandmakerQuestState {
    pub spawned: bool,
    /// A spawn attempt ran on a run-history dependent stream, so whether
    /// (and where) this quest spawned is not seed-determined.
    pub history_dependent: bool,
    /// 0 = undecided; 1–3 once type is chosen (when room is added).
    pub quest_type: i32,
    /// Set true on the floor where `spawnRoom` adds the quest room.
    pub quest_room_spawned: bool,
    pub depth: i32,
    /// Persistent quest choices, matching `Wandmaker.Quest.wand1/wand2`.
    pub wand1: Option<GeneratedItem>,
    pub wand2: Option<GeneratedItem>,
}

#[derive(Debug, Clone)]
pub struct WandmakerSpawnResult {
    pub quest_type: WandmakerQuestType,
    pub wand1: GeneratedItem,
    pub wand2: GeneratedItem,
    /// Cell occupied before `RegularLevel.createMobs` places ambient enemies.
    pub cell: usize,
    /// `WAND.dropped` before the first reward draw. Not recoverable afterwards:
    /// the duplicate loop's `undoDrop` does not restore a concrete wand.
    pub wand1_deck_index: i32,
}

/// `Wandmaker.Quest.spawnRoom(rooms)` — prison only; call before room shuffle.
///
/// SPD: `!spawned && (type != 0 || (depth > 6 && Random.Int(10 - depth) == 0))`.
/// Depth 6 never rolls; depth 9 always succeeds if not yet spawned.
pub fn try_spawn_room(
    wandmaker: &mut WandmakerQuestState,
    depth: i32,
    specs: &mut Vec<RoomSpec>,
) -> bool {
    wandmaker.quest_room_spawned = false;

    // Only PrisonLevel (depths 6–9) calls this. Depth 6 never passes `depth > 6`.
    if !(6..=9).contains(&depth) {
        return false;
    }
    if wandmaker.history_dependent {
        Random::taint_stream();
    }
    if wandmaker.spawned {
        return false;
    }
    wandmaker.history_dependent |= Random::stream_tainted();

    let type_ready = wandmaker.quest_type != 0;
    let roll_ok = depth > 6 && Random::int_max(10 - depth) == 0;
    if !type_ready && !roll_ok {
        return false;
    }

    if wandmaker.quest_type == 0 {
        wandmaker.quest_type = Random::int_max(3) + 1;
    }
    let qtype = WandmakerQuestType::from_int(wandmaker.quest_type);
    specs.push(quest_room_spec(qtype));
    wandmaker.quest_room_spawned = true;
    true
}

fn quest_room_spec(qtype: WandmakerQuestType) -> RoomSpec {
    match qtype {
        // SpecialRoom subclasses — maxConnections = 1
        WandmakerQuestType::CorpseDust | WandmakerQuestType::Rotberry => RoomSpec {
            name: qtype.room_name().into(),
            kind: RoomKind::Special,
            size_factor: 1,
            max_connections: 1,
        },
        // RitualSiteRoom extends StandardRoom
        WandmakerQuestType::ElementalEmbers => {
            // RitualSiteRoom extends StandardRoom, whose instance initializer
            // calls setSizeCat() even though its default weights force NORMAL.
            let size_factor = crate::rooms::standard::set_size_cat_default(qtype.room_name());
            RoomSpec {
                name: qtype.room_name().into(),
                kind: RoomKind::Standard,
                size_factor,
                max_connections: 16,
            }
        }
    }
}

/// `Wandmaker.Quest.spawnWandmaker(level, roomEntrance)` — after paint, before createItems.
pub fn try_spawn_wandmaker(
    dungeon: &mut DungeonState,
    entrance: &Room,
    map: &TerrainMap,
) -> Option<WandmakerSpawnResult> {
    if !dungeon.wandmaker.quest_room_spawned {
        return None;
    }
    dungeon.wandmaker.quest_room_spawned = false;

    // Placement loop on entrance (consumes RNG; terrain flags approximate).
    let cell = place_wandmaker(entrance, map)?;

    dungeon.wandmaker.spawned = true;
    dungeon.wandmaker.depth = dungeon.depth;
    let quest_type = WandmakerQuestType::from_int(dungeon.wandmaker.quest_type);

    let (wand1, wand2, wand1_deck_index) = generate_wands(dungeon);
    #[cfg(test)]
    LAST_WAND1_INDEX.with(|index| index.set(Some(wand1_deck_index)));

    #[cfg(test)]
    LAST_PLACEMENT_TRACE.with(|trace| {
        *trace.borrow_mut() = Some((dungeon.depth, quest_type as i32, cell, Random::peek_ints(8)));
    });
    dungeon.wandmaker.wand1 = Some(wand1.clone());
    dungeon.wandmaker.wand2 = Some(wand2.clone());

    Some(WandmakerSpawnResult {
        quest_type,
        wand1,
        wand2,
        cell,
        wand1_deck_index,
    })
}

fn place_wandmaker(room: &Room, map: &TerrainMap) -> Option<usize> {
    let entrance_cell = entrance_center_cell(room, map);
    let mut tries = 0;
    let mut dist = 2;
    for _ in 0..2000 {
        if tries > 30 && dist > 0 {
            tries = 0;
            dist -= 1;
        }
        let p = room.random_margin(dist);
        tries += 1;
        let Some(cell) = map.point_to_cell(p.x, p.y) else {
            continue;
        };
        if Some(cell) == entrance_cell {
            continue;
        }
        if map.is_solid(cell) {
            continue;
        }
        // Pinned `spawnWandmaker` checks the level trap registry directly and
        // does not consult the broader room `canPlaceCharacter` predicate.
        if map.trap_names.get(cell).is_some_and(Option::is_some) {
            continue;
        }
        if !map.passable.get(cell).copied().unwrap_or(false) {
            continue;
        }
        if map.map.get(cell).copied() == Some(crate::level::EMPTY_SP) {
            continue;
        }
        // Only Terrain.DOOR is rejected here. OPEN_DOOR and LOCKED_DOOR are
        // distinct terrain values in the pinned source.
        if has_adjacent_door(map, cell) {
            continue;
        }
        // Valid placement
        return Some(cell);
    }
    None
}

fn has_adjacent_door(map: &TerrainMap, cell: usize) -> bool {
    let w = map.width as usize;
    let x = cell % w;
    let y = cell / w;
    for (dx, dy) in [(0i32, -1), (1, 0), (0, 1), (-1, 0)] {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= map.width || ny >= map.height {
            continue;
        }
        let ncell = (nx + ny * map.width) as usize;
        if map.map.get(ncell).copied() == Some(crate::level::DOOR) {
            return true;
        }
    }
    false
}

fn generate_wands(dungeon: &mut DungeonState) -> (GeneratedItem, GeneratedItem, i32) {
    // Index of the next wand draw. `undoDrop` does not put a rejected wand
    // back, so `dropped` after the pair is not wand 2's index either.
    let wand1_deck_index = dungeon.generator.deck_dropped(Category::Wand);

    let mut wand1 = dungeon
        .generator
        .random_category(Category::Wand, dungeon.depth);
    wand1.cursed = false;
    upgrade_wand(&mut wand1);

    let mut wand2 = dungeon
        .generator
        .random_category(Category::Wand, dungeon.depth);
    let mut to_undo: Vec<String> = Vec::new();
    while wand2.class_name == wand1.class_name {
        to_undo.push(wand2.class_name.clone());
        wand2 = dungeon
            .generator
            .random_category(Category::Wand, dungeon.depth);
    }
    for class_name in &to_undo {
        dungeon.generator.undo_drop(class_name);
    }
    wand2.cursed = false;
    upgrade_wand(&mut wand2);

    wand1.source = Some("Wandmaker.Quest".into());
    wand2.source = Some("Wandmaker.Quest".into());
    wand1.provenance = crate::items::model::ItemProvenance::Quest(
        crate::items::model::QuestRewardRole::WandmakerWand,
    );
    wand2.provenance = crate::items::model::ItemProvenance::Quest(
        crate::items::model::QuestRewardRole::WandmakerWand,
    );

    (wand1, wand2, wand1_deck_index)
}

/// `Wand.upgrade()` after clearing curse: level++, `Random.Int(3)==0` curse clear.
fn upgrade_wand(wand: &mut GeneratedItem) {
    wand.level += 1;
    if Random::int_max(3) == 0 {
        wand.cursed = false;
    }
}

fn entrance_center_cell(room: &Room, map: &TerrainMap) -> Option<usize> {
    for y in room.top..=room.bottom {
        for x in room.left..=room.right {
            let Some(cell) = map.point_to_cell(x, y) else {
                continue;
            };
            if matches!(
                map.map[cell],
                crate::level::ENTRANCE | crate::level::ENTRANCE_SP
            ) {
                return Some(cell);
            }
        }
    }
    let cx = (room.left + room.right) / 2;
    let cy = (room.top + room.bottom) / 2;
    map.point_to_cell(cx, cy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::{dungeon_from_run, init_run};

    #[test]
    fn wand_rewards_deterministic() {
        Random::reset_generators();
        let run = init_run(42);
        Random::push_generator_seeded(12345);
        let (w1a, w1b, _) = {
            let mut d = dungeon_from_run(run.clone());
            d.depth = 9;
            generate_wands(&mut d)
        };
        Random::pop_generator();

        Random::reset_generators();
        Random::push_generator_seeded(12345);
        let (w2a, w2b, _) = {
            let mut d = dungeon_from_run(run);
            d.depth = 9;
            generate_wands(&mut d)
        };
        Random::pop_generator();

        assert_eq!(w1a.class_name, w2a.class_name);
        assert_eq!(w1a.level, w2a.level);
        assert_eq!(w1b.class_name, w2b.class_name);
        assert_ne!(w1a.class_name, w1b.class_name);
    }

    #[test]
    fn fixed_profile_rewards_match_committed_java_fixtures() {
        // Pinned to a fresh-journal Java oracle run.
        let _journal = crate::journal::scope_lore_complete(false);
        let cases = [
            (
                "AAA-AAA-AAA",
                9,
                ("WandOfTransfusion", 1),
                ("WandOfFrost", 1),
            ),
            (
                "GFX-PZH-DCH",
                7,
                ("WandOfBlastWave", 1),
                ("WandOfLightning", 2),
            ),
            (
                "HKT-JZN-XQQ",
                7,
                ("WandOfLivingEarth", 1),
                ("WandOfPrismaticLight", 3),
            ),
        ];

        for (seed, spawn_depth, expected1, expected2) in cases {
            Random::reset_generators();
            let numeric = crate::parse_seed(seed).expect("valid fixture seed").numeric;
            let run = init_run(numeric);
            let mut dungeon = dungeon_from_run(run);
            for depth in 1..=spawn_depth {
                dungeon.depth = depth;
                let _ = crate::level::create_level_partial(&mut dungeon);
            }

            let wand1 = dungeon.wandmaker.wand1.as_ref().expect("first reward");
            let wand2 = dungeon.wandmaker.wand2.as_ref().expect("second reward");
            assert_eq!(
                (wand1.class_name.as_str(), wand1.level),
                expected1,
                "{seed}"
            );
            assert_eq!(
                (wand2.class_name.as_str(), wand2.level),
                expected2,
                "{seed}"
            );
            assert!(!wand1.cursed && !wand2.cursed, "{seed}");
            assert_ne!(wand1.class_name, wand2.class_name, "{seed}");
        }
    }

    #[test]
    fn depth9_always_spawns_room_when_not_spawned() {
        Random::reset_generators();
        let mut wm = WandmakerQuestState::default();
        let mut specs = Vec::new();
        // Int(1) is always 0
        assert!(try_spawn_room(&mut wm, 9, &mut specs));
        assert_eq!(specs.len(), 1);
        assert!(wm.quest_room_spawned);
        assert!((1..=3).contains(&wm.quest_type));
    }

    #[test]
    fn depth6_never_spawns_room() {
        Random::reset_generators();
        let mut wm = WandmakerQuestState::default();
        let mut specs = Vec::new();
        // depth > 6 is false — no roll
        assert!(!try_spawn_room(&mut wm, 6, &mut specs));
        assert!(specs.is_empty());
    }
}

#[cfg(test)]
type PlacementTrace = (i32, i32, usize, Vec<i32>);

#[cfg(test)]
thread_local! {
    static LAST_PLACEMENT_TRACE: std::cell::RefCell<Option<PlacementTrace>> =
        const { std::cell::RefCell::new(None) };
    static LAST_WAND1_INDEX: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(super) fn take_wand1_index() -> Option<i32> {
    LAST_WAND1_INDEX.with(std::cell::Cell::take)
}

#[cfg(test)]
pub(super) fn take_placement_trace() -> Option<PlacementTrace> {
    LAST_PLACEMENT_TRACE.with(|trace| trace.borrow_mut().take())
}
