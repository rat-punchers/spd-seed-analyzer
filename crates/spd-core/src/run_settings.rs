//! Report for a seed whose challenges the player has declared.

use crate::trinkets::{Challenge, MapProfile};
use crate::{
    analyze_route, analyze_run, analyze_seed_with_trinket, conditional, init_run, parse_seed,
    AnalyzeError, SeedReport, TrinketKind, TrinketSelectionReport,
};

/// Analyze a seed for a run started with `challenges` enabled, optionally
/// holding the Catalyst `trinket` (see [`analyze_seed_with_trinket`]).
///
/// Challenges are player facts, so no challenge appears as a spawn
/// condition. Without a trinket, the Catalyst routes stay open as
/// conditions; with one, no alternatives are enumerated. An empty challenge
/// list is [`analyze_seed_with_trinket`].
///
/// `lore_complete` is the player's journal (see [`crate::journal`]): `true`,
/// the default elsewhere, spawns no guide or alchemy pages; `false` replays
/// a fresh journal, whose pages appear on the floors.
pub fn analyze_seed_for_run(
    input: &str,
    floors: u32,
    trinket: Option<TrinketKind>,
    challenges: &[Challenge],
    lore_complete: bool,
) -> Result<SeedReport, AnalyzeError> {
    crate::journal::with_lore_complete(lore_complete, || {
        analyze_declared_run(input, floors, trinket, challenges)
    })
}

fn analyze_declared_run(
    input: &str,
    floors: u32,
    trinket: Option<TrinketKind>,
    challenges: &[Challenge],
) -> Result<SeedReport, AnalyzeError> {
    let challenges = normalized(challenges);
    if challenges.is_empty() {
        return analyze_seed_with_trinket(input, floors, trinket);
    }
    let info = parse_seed(input)?;
    let run = init_run(info.numeric);
    let selection = TrinketSelectionReport::for_run(&run);
    let mut profile = match trinket {
        Some(trinket) => selection.catalyst_profile(trinket)?,
        None => MapProfile::default(),
    };
    profile.challenges = challenges.clone();
    let mut report = analyze_run(info, run, selection, floors, Some(&profile), true)?;

    let base = conditional::condition_for_profile(&profile, true);
    let routes = if trinket.is_some() {
        Vec::new()
    } else {
        conditional::discover_routes(
            &report.trinket_selection,
            &report.floors,
            report.floors_requested,
            Some(&challenges),
        )
    };
    let mut alternatives = Vec::new();
    for route in routes {
        let floors = analyze_route(input, &route, &report.trinket_selection)?;
        alternatives.push((route, floors));
    }
    conditional::merge_possible_rooms(&mut report.floors, base.clone(), &alternatives);
    conditional::merge_possible_items(&mut report.floors, base, &mut alternatives);
    Ok(report)
}

/// Deduplicated challenges in the game's mask order.
fn normalized(challenges: &[Challenge]) -> Vec<Challenge> {
    Challenge::ALL
        .into_iter()
        .filter(|challenge| challenges.contains(challenge))
        .collect()
}

#[cfg(test)]
#[path = "run_settings_tests.rs"]
mod tests;
