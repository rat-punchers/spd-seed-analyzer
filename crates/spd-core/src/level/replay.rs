//! Ascending floor replays that turn per-floor level states into reports.

use super::{create_level_internal, mining, trinkets, vault, LevelOutput, LevelState};
use crate::dungeon::DungeonState;
use crate::report::FloorReport;
use crate::MapProfile;

pub fn analyze_layouts_with_profile(
    dungeon: &mut DungeonState,
    max_floors: u32,
    profile: &MapProfile,
) -> Vec<FloorReport> {
    let mut availability_preview = dungeon.clone();
    let (catalyst_depth, pot_depth, _) = first_trinket_availability(&mut availability_preview);
    let first_effective_trinket_depth = catalyst_depth.max(pot_depth) + 1;
    dungeon.challenges = profile.challenge_mask();
    let mut floors = Vec::new();
    trinkets::reset(dungeon.seed);
    let max = max_floors.clamp(1, 26) as i32;
    for depth in 1..=max {
        dungeon.depth = depth;
        dungeon.branch = 0;
        let depth_u32 = depth as u32;
        trinkets::set_held(profile.held_at(depth_u32));
        let level = create_level_internal(
            dungeon,
            !profile.has_unmodeled_generation_inputs_at(depth_u32),
            LevelOutput::LAYOUT,
        );
        let mut report =
            level.to_floor_report_with_trinket_availability(true, first_effective_trinket_depth);
        attach_side_branches(dungeon, true, &mut report);
        floors.push(report);
    }
    floors
}

/// Finds the first Trinket Catalyst and alchemy pot, both guaranteed by depth 4.
#[derive(Default)]
struct TrinketScan {
    catalyst_depth: Option<u32>,
    first_pot: Option<(u32, bool)>,
}

impl TrinketScan {
    fn observe(&mut self, level: &LevelState) {
        let depth = level.depth as u32;
        if self.catalyst_depth.is_none()
            && level
                .initial_forced_items
                .iter()
                .any(|item| item.class_name == "TrinketCatalyst")
        {
            self.catalyst_depth = Some(depth);
        }
        if self.first_pot.is_none() {
            self.first_pot = level
                .rooms
                .iter()
                .find(|room| matches!(room.as_str(), "LaboratoryRoom" | "SecretLaboratoryRoom"))
                .map(|room| (depth, room == "SecretLaboratoryRoom"));
        }
    }

    fn first_effective_depth(&self) -> Option<u32> {
        Some(self.catalyst_depth?.max(self.first_pot?.0) + 1)
    }
}

pub(crate) fn first_trinket_availability(dungeon: &mut DungeonState) -> (u32, u32, bool) {
    dungeon.challenges = 0;
    trinkets::reset(dungeon.seed);
    let mut scan = TrinketScan::default();
    for depth in 1..=4 {
        dungeon.depth = depth;
        dungeon.branch = 0;
        trinkets::set_held(None);
        scan.observe(&create_level_internal(dungeon, true, LevelOutput::ROOMS));
    }
    let catalyst_depth = scan
        .catalyst_depth
        .expect("one Trinket Catalyst is guaranteed by depth 4");
    let (pot_depth, pot_is_secret) = scan
        .first_pot
        .expect("a LaboratoryRoom is guaranteed by depth 4");
    (catalyst_depth, pot_depth, pot_is_secret)
}

pub fn analyze_floors(dungeon: &mut DungeonState, max_floors: u32) -> Vec<FloorReport> {
    analyze_floors_with_profile(dungeon, max_floors, None)
}

pub fn analyze_floors_with_profile(
    dungeon: &mut DungeonState,
    max_floors: u32,
    profile: Option<&MapProfile>,
) -> Vec<FloorReport> {
    let mut availability_preview = dungeon.clone();
    let (catalyst_depth, pot_depth, _) = first_trinket_availability(&mut availability_preview);
    let first_effective_trinket_depth = catalyst_depth.max(pot_depth) + 1;
    analyze_floors_with_profile_and_trinket_availability(
        dungeon,
        max_floors,
        profile,
        first_effective_trinket_depth,
    )
}

pub(crate) fn analyze_floors_with_profile_and_trinket_availability(
    dungeon: &mut DungeonState,
    max_floors: u32,
    profile: Option<&MapProfile>,
    first_effective_trinket_depth: u32,
) -> Vec<FloorReport> {
    replay_floors(
        dungeon,
        max_floors,
        profile,
        Some(first_effective_trinket_depth),
        LevelOutput::FULL,
        |_| true,
    )
}

/// Item and room replay for a conditional route. Same generation as
/// [`analyze_floors_with_profile_and_trinket_availability`], minus map
/// projection, parity probes, and side branches, which route merges ignore.
pub(crate) fn analyze_route_floors(
    dungeon: &mut DungeonState,
    max_floors: u32,
    profile: &MapProfile,
    first_effective_trinket_depth: u32,
) -> Vec<FloorReport> {
    replay_floors(
        dungeon,
        max_floors,
        Some(profile),
        Some(first_effective_trinket_depth),
        LevelOutput::ITEMS,
        |_| true,
    )
}

/// Item-only replay for the seed finder, using the default profile.
///
/// Generation still paints every floor (painting consumes the RNG later loot
/// depends on), but it skips map projection, parity probes, and side
/// branches, none of which feed item results. The default profile replays
/// the same floors 1-4 as `first_trinket_availability`, so trinket
/// availability is read from this replay instead of a separate preview.
/// After each floor, `keep_going` sees every report so far (and may adjust
/// the newest one); returning `false` stops before the next floor.
pub(crate) fn search_floors(
    dungeon: &mut DungeonState,
    max_floors: u32,
    keep_going: impl FnMut(&mut [FloorReport]) -> bool,
) -> Vec<FloorReport> {
    replay_floors(
        dungeon,
        max_floors,
        Some(&MapProfile::default()),
        None,
        LevelOutput::ITEMS,
        keep_going,
    )
}

/// Raw per-floor states for the default profile (no challenges, no held
/// trinket, no prior player history), in ascending depth order.
pub(crate) fn replay_default_profile_states(
    dungeon: &mut DungeonState,
    max_floors: u32,
    maps: bool,
) -> Vec<LevelState> {
    let profile = MapProfile::default();
    dungeon.challenges = profile.challenge_mask();
    dungeon.ghost_rewards_profiled = !profile.has_unmodeled_generation_inputs();
    dungeon.expose_variable_items = true;
    trinkets::reset(dungeon.seed);
    let output = if maps {
        LevelOutput::FULL
    } else {
        LevelOutput::ITEMS
    };
    (1..=max_floors.clamp(1, 26) as i32)
        .map(|depth| {
            dungeon.depth = depth;
            dungeon.branch = 0;
            trinkets::set_held(None);
            create_level_internal(dungeon, true, output)
        })
        .collect()
}

/// Replays floors in ascending order. With `first_effective_trinket_depth`
/// unknown, reports are held back until floors 1-4 reveal it.
fn replay_floors(
    dungeon: &mut DungeonState,
    max_floors: u32,
    profile: Option<&MapProfile>,
    first_effective_trinket_depth: Option<u32>,
    output: LevelOutput,
    mut keep_going: impl FnMut(&mut [FloorReport]) -> bool,
) -> Vec<FloorReport> {
    debug_assert!(
        first_effective_trinket_depth.is_some() || !output.maps,
        "side branches read the current dungeon depth and cannot be deferred"
    );
    dungeon.challenges = profile.map_or(0, MapProfile::challenge_mask);
    trinkets::reset(dungeon.seed);
    let max = max_floors.clamp(1, 26) as i32;
    let mut first_effective = first_effective_trinket_depth;
    let mut scan = TrinketScan::default();
    let mut pending = Vec::new();
    let mut floors = Vec::new();
    for depth in 1..=26 {
        if depth > max && first_effective.is_some() {
            break;
        }
        let depth_u32 = depth as u32;
        let configured =
            profile.is_some_and(|profile| !profile.has_unmodeled_generation_inputs_at(depth_u32));
        dungeon.depth = depth;
        dungeon.branch = 0;
        dungeon.ghost_rewards_profiled = configured;
        trinkets::set_held(profile.and_then(|profile| profile.held_at(depth_u32)));
        let level = create_level_internal(dungeon, configured, output);
        if first_effective.is_none() {
            scan.observe(&level);
            first_effective = scan.first_effective_depth();
        }
        if depth <= max {
            pending.push(level);
        }
        let Some(first_effective) = first_effective else {
            continue;
        };
        for level in pending.drain(..) {
            let level_configured = profile.is_some_and(|profile| {
                !profile.has_unmodeled_generation_inputs_at(level.depth as u32)
            });
            let mut report =
                level.to_floor_report_with_trinket_availability(level_configured, first_effective);
            if output.maps {
                attach_side_branches(dungeon, level_configured, &mut report);
            } else {
                // Boss floors still build their fixed layout to decide `build_ok`.
                report.map = None;
                report.assumed_map = None;
            }
            floors.push(report);
            if !keep_going(&mut floors) {
                return floors;
            }
        }
    }
    floors
}

pub(super) fn attach_side_branches(
    dungeon: &DungeonState,
    expose_map: bool,
    report: &mut FloorReport,
) {
    if dungeon.blacksmith.depth == dungeon.depth {
        if let Some(branch) = mining::generate(dungeon, expose_map) {
            report.branches.push(branch.report);
        }
    }
    if dungeon.imp.spawned && dungeon.imp.depth == dungeon.depth {
        let mut vault_dungeon = dungeon.clone();
        if let Some(layout) = vault::generate(&mut vault_dungeon, expose_map) {
            report.branches.push(layout.report);
        }
    }
}
