//! NPC quest generation (Ghost / Wandmaker / Blacksmith / Imp / …).

mod blacksmith;
mod ghost;
mod imp;
mod wandmaker;

pub use blacksmith::{
    take_pending as take_blacksmith_pending, try_spawn as try_spawn_blacksmith,
    BlacksmithQuestState, BlacksmithQuestType,
};
pub use ghost::{try_spawn_ghost, GhostQuestState, GhostType};
pub use imp::{
    build_reward_slots, take_pending as take_imp_pending, total_upgrade_value as imp_upgrade_value,
    try_spawn as try_spawn_imp, upgrades_pinned as imp_upgrades_pinned, ImpQuestState, ImpSlotDraw,
    ImpSlotKind, IMP_DEEP_DRIFT, IMP_DEFAULT_DRIFT,
};
pub use wandmaker::{
    build_reward_slots as build_wandmaker_slots, try_spawn_room as try_spawn_wandmaker_room,
    try_spawn_wandmaker, WandmakerQuestState, WandmakerQuestType,
    DEFAULT_DRIFT as WANDMAKER_DEFAULT_DRIFT,
};

#[cfg(test)]
mod placement_oracle_tests {
    use serde::Deserialize;
    use std::path::Path;

    #[derive(Deserialize)]
    struct Fixture {
        input: Input,
        ghost: SpawnFact,
        wandmaker: SpawnFact,
    }

    #[derive(Deserialize)]
    struct Input {
        seed: String,
    }

    #[derive(Debug, Deserialize)]
    struct SpawnFact {
        depth: i32,
        quest_type: i32,
        cell: usize,
        rng_tail: Vec<i32>,
    }

    #[test]
    fn quest_npc_cells_and_reward_tails_match_oracle_matrix() {
        // Pinned to a fresh-journal Java oracle run.
        let _journal = crate::journal::scope_lore_complete(false);
        let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tools/java-oracle/fixtures/generator");
        let mut paths: Vec<_> = std::fs::read_dir(&fixture_dir)
            .expect("quest oracle fixture directory")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with("-quest-npc-placement.json"))
            })
            .collect();
        paths.sort();
        assert_eq!(paths.len(), 9, "complete Wandmaker depth/type matrix");

        let mut ghost_depths = [false; 3];
        let mut wandmaker_matrix = [[false; 3]; 3];
        let mut mismatches = Vec::new();
        for path in paths {
            let fixture: Fixture = serde_json::from_str(
                &std::fs::read_to_string(&path).expect("read quest oracle fixture"),
            )
            .expect("parse quest oracle fixture");
            crate::analyze_seed(&fixture.input.seed, 9).expect("analyze oracle seed");

            let ghost = super::ghost::take_placement_trace().expect("Ghost spawned");
            if !trace_matches(&fixture.input.seed, "Ghost", ghost, &fixture.ghost) {
                mismatches.push(format!("{} Ghost", fixture.input.seed));
            }
            ghost_depths[(fixture.ghost.depth - 2) as usize] = true;

            let wandmaker = super::wandmaker::take_placement_trace().expect("Wandmaker spawned");
            if !trace_matches(
                &fixture.input.seed,
                "Wandmaker",
                wandmaker,
                &fixture.wandmaker,
            ) {
                mismatches.push(format!("{} Wandmaker", fixture.input.seed));
            }
            wandmaker_matrix[(fixture.wandmaker.depth - 7) as usize]
                [(fixture.wandmaker.quest_type - 1) as usize] = true;
        }
        assert!(ghost_depths.into_iter().all(|covered| covered));
        assert!(wandmaker_matrix
            .into_iter()
            .flatten()
            .all(|covered| covered));
        assert!(
            mismatches.is_empty(),
            "quest placement mismatches: {mismatches:?}"
        );
    }

    fn trace_matches(
        seed: &str,
        npc: &str,
        actual: (i32, i32, usize, Vec<i32>),
        expected: &SpawnFact,
    ) -> bool {
        let expected = (
            expected.depth,
            expected.quest_type,
            expected.cell,
            expected.rng_tail.clone(),
        );
        if actual != expected {
            eprintln!("{seed} {npc}: actual={actual:?} expected={expected:?}");
        }
        actual == expected
    }
}
