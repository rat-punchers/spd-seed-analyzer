//! Pure Rust port of Shattered Pixel Dungeon generation logic needed for seed analysis.
//!
//! Target game source: local clone of 00-Evan/shattered-pixel-dungeon (see README).

pub mod builders;
mod conditional;
pub mod dungeon;
pub mod dungeon_seed;
pub mod exact;
pub mod generator;
pub mod geom;
pub mod items;
pub mod java_random;
pub mod journal;
pub mod level;
pub mod quests;
pub mod random;
pub mod report;
pub mod rooms;
pub mod run;
mod run_settings;
pub mod search;
mod selected_trinket;
pub mod trinkets;

pub use dungeon_seed::{DungeonSeed, SeedError, FIRST_DAILY_DATE, TOTAL_SEEDS};
pub use exact::{analyze_seed_exact, ExactOptions, ExactReport};
pub use items::IdentityMaps;
pub use java_random::JavaRandom;
pub use random::Random;
pub use report::{
    AmbitiousImpQuestContract, AmbitiousImpQuestVariable, AnalyzeError, BlacksmithObjective,
    BranchAccessReport, BranchFloorId, BranchFloorKind, BranchFloorReport, FloorReport,
    GhostTarget, GuaranteedAppearance, GuaranteedAppearanceKind, ItemCondition,
    ItemDependencyCondition, ItemEnchantment, ItemSpawnCondition, OldWandmakerQuestContract,
    OldWandmakerQuestVariable, QuestDepthRange, QuestReport, QuestRewardSelection,
    SadGhostQuestContract, SadGhostQuestVariable, SeedInfo, SeedReport,
    TrollBlacksmithQuestContract, TrollBlacksmithQuestVariable, WandmakerObjective,
};
pub use run::{dungeon_from_run, init_run, RunState};
pub use run_settings::analyze_seed_for_run;
pub use search::{
    search_seeds, ItemConstraint, ItemMatchEvidence, MatchMode, SearchError, SeedMatch,
    SeedSearchRequest, SeedSearchResult, VariableItemEvidence, MAX_SEARCH_CANDIDATES,
    MAX_SEARCH_CONSTRAINTS, MAX_SEARCH_FLOORS, MAX_SEARCH_MATCHES,
};
pub use selected_trinket::analyze_seed_with_trinket;
pub use trinkets::{
    ArtifactEvent, ArtifactEventAction, ArtifactKind, Challenge, ChallengeEffect, ClaimState,
    MapProfile, ProfileError, TrinketEvent, TrinketEventAction, TrinketGenerationEffect,
    TrinketKind, TrinketSelectionReport,
};

/// Pinned SPD version this port targets (from local clone at scaffold time).
pub const SPD_VERSION: &str = "v4.0.2";
pub const SPD_COMMIT: &str = "57a4e06a4";

/// Parse a user seed string into display info (no levelgen).
pub fn parse_seed(input: &str) -> Result<SeedInfo, SeedError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(SeedError::Empty);
    }
    let daily = DungeonSeed::is_daily_date_text(trimmed);
    let numeric = if daily {
        DungeonSeed::convert_from_daily_date(trimmed)?
    } else {
        DungeonSeed::convert_from_text(trimmed)?
    };
    let code = if !daily {
        DungeonSeed::convert_to_code(numeric).ok()
    } else {
        None
    };
    Ok(SeedInfo {
        input: trimmed.to_string(),
        numeric,
        code,
        formatted: DungeonSeed::format_text(trimmed),
        daily,
    })
}

/// Analyze a seed and merge every currently supported item-generating profile
/// into each floor's item list with structured spawn conditions.
pub fn analyze_seed(input: &str, floors: u32) -> Result<SeedReport, AnalyzeError> {
    let default_profile = MapProfile::default();
    let mut report = analyze_seed_internal(input, floors, Some(&default_profile))?;
    let routes = conditional::discover_routes(
        &report.trinket_selection,
        &report.floors,
        report.floors_requested,
        None,
    );
    let mut alternatives = Vec::new();
    for route in routes {
        let floors = analyze_route(input, &route, &report.trinket_selection)?;
        alternatives.push((route, floors));
    }
    conditional::merge_possible_rooms(
        &mut report.floors,
        conditional::default_condition(),
        &alternatives,
    );
    conditional::merge_possible_items(
        &mut report.floors,
        conditional::default_condition(),
        &mut alternatives,
    );
    // Parity probes retain the latest replay trace for oracle inspection. Keep
    // that diagnostic state on the conservative finder projection rather than
    // whichever conditional route happened to be discovered last. The replay
    // is discarded, so only test builds (which read the traces) pay for it.
    #[cfg(test)]
    let _ = analyze_seed_internal(input, floors, None)?;
    Ok(report)
}

/// Full seed-only projection: the parity reference for `search_seed_floors`.
/// It deliberately avoids replaying the UI-facing modeled branch matrix.
#[cfg(test)]
pub(crate) fn analyze_seed_seed_only(input: &str, floors: u32) -> Result<SeedReport, AnalyzeError> {
    // A seed-only search still replays the default profile: no challenges,
    // held trinket, or prior player history. Replaying that profile prevents the
    // default-feeling callback from tainting all later layout facts merely
    // because this internal fast path omitted an explicit profile.
    analyze_seed_internal(input, floors, Some(&MapProfile::default()))
}

/// Item-only seed-only replay for the finder. Floor items and quests match
/// `analyze_seed_seed_only`; maps, branches, encounters' map facts, and the
/// alchemy-pot appearance are left out. `keep_going` can stop generation
/// after any floor.
pub(crate) fn search_seed_floors(
    input: &str,
    floors: u32,
    keep_going: impl FnMut(&mut [FloorReport]) -> bool,
) -> Result<(SeedInfo, IdentityMaps, Vec<FloorReport>), AnalyzeError> {
    let info = parse_seed(input)?;
    let mut dungeon = dungeon_from_run(init_run(info.numeric));
    dungeon.expose_variable_items = true;
    let identities = dungeon.identities.clone();
    let floors = level::search_floors(&mut dungeon, floors, keep_going);
    Ok((info, identities, floors))
}

/// Internal profile replay used by parity tests.
/// It is not exposed through the WASM API: users cannot configure a run state.
#[cfg(test)]
pub(crate) fn analyze_seed_with_profile(
    input: &str,
    floors: u32,
    profile: &MapProfile,
) -> Result<SeedReport, AnalyzeError> {
    analyze_seed_internal(input, floors, Some(profile))
}

/// Floors a conditional route contributes to the merged report: only items
/// and rooms are read, so maps and side branches are skipped. The trinket
/// selection depends only on the seed and is shared with the default replay.
fn analyze_route(
    input: &str,
    route: &conditional::DiscoveredRoute,
    selection: &TrinketSelectionReport,
) -> Result<Vec<FloorReport>, AnalyzeError> {
    let info = parse_seed(input)?;
    route.profile.validate(selection.first_effective_depth)?;
    let mut dungeon = dungeon_from_run(init_run(info.numeric));
    dungeon.expose_variable_items = route.profile == MapProfile::default();
    Ok(level::analyze_route_floors(
        &mut dungeon,
        route.max_depth.clamp(1, 26),
        &route.profile,
        selection.first_effective_depth,
    ))
}

fn analyze_seed_internal(
    input: &str,
    floors: u32,
    profile: Option<&MapProfile>,
) -> Result<SeedReport, AnalyzeError> {
    let info = parse_seed(input)?;
    let run = init_run(info.numeric);
    let trinket_selection = TrinketSelectionReport::for_run(&run);
    let expose_variable_items = profile.is_some_and(|profile| profile == &MapProfile::default());
    analyze_run(
        info,
        run,
        trinket_selection,
        floors,
        profile,
        expose_variable_items,
    )
}

/// Replays `run` under `profile`. Variable items carry a concrete sample only
/// when `expose_variable_items` is set: the profile is the player's own.
fn analyze_run(
    info: SeedInfo,
    run: RunState,
    trinket_selection: TrinketSelectionReport,
    floors: u32,
    profile: Option<&MapProfile>,
    expose_variable_items: bool,
) -> Result<SeedReport, AnalyzeError> {
    let floors = floors.clamp(1, 26);
    if let Some(profile) = profile {
        profile.validate(trinket_selection.first_effective_depth)?;
    }
    let mut dungeon = dungeon_from_run(run);
    dungeon.expose_variable_items = expose_variable_items;
    let identities = dungeon.identities.clone();
    let mut floor_reports = level::analyze_floors_with_profile_and_trinket_availability(
        &mut dungeon,
        floors,
        profile,
        trinket_selection.first_effective_depth,
    );
    if let Some(floor) = floor_reports
        .iter_mut()
        .find(|floor| floor.depth == trinket_selection.first_alchemy_pot_depth)
    {
        let source = if trinket_selection.first_alchemy_pot_is_secret {
            spd_objects::RoomId::SecretLaboratoryRoom
        } else {
            spd_objects::RoomId::LaboratoryRoom
        };
        if !floor
            .guaranteed_appearances
            .iter()
            .any(|appearance| appearance.source == Some(source))
        {
            floor.guaranteed_appearances.push(GuaranteedAppearance {
                kind: GuaranteedAppearanceKind::AlchemyPot,
                source: Some(source),
            });
        }
    }

    Ok(SeedReport {
        seed: info,
        spd_version: SPD_VERSION.to_string(),
        spd_commit: SPD_COMMIT.to_string(),
        floors_requested: floors,
        identities,
        trinket_selection,
        floors: floor_reports,
        status: "partial".to_string(),
    })
}

#[cfg(test)]
#[path = "analyze_smoke.rs"]
mod analyze_smoke;
