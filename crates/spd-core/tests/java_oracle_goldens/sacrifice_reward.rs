use serde::Deserialize;
use spd_core::level::create_level_partial;
use spd_core::{dungeon_from_run, init_run, parse_seed, SPD_COMMIT, SPD_VERSION};

#[derive(Deserialize)]
struct Fixture {
    contract: String,
    spd: Pin,
    input: Input,
    lifecycle: String,
    reward: Reward,
}

#[derive(Deserialize)]
struct Pin {
    version: String,
    commit: String,
}

#[derive(Deserialize)]
struct Input {
    seed: String,
    numeric: i64,
    depth: i32,
    parchment_scrap_level: i32,
}

#[derive(Deserialize)]
struct Reward {
    #[serde(rename = "class")]
    class_name: String,
    quantity: i32,
    level: i32,
    cursed: bool,
    enchantment: Option<String>,
}

#[test]
fn parchment_three_floor_thirteen_sacrifice_reward_matches_java() {
    let _journal = crate::fresh_journal();
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../../tools/java-oracle/fixtures/player-state/hkt-jzn-xqq-sacrifice-reward-parchment-3.json"
    ))
    .expect("valid sacrifice fixture");
    assert_eq!(fixture.contract, "sacrifice_reward_player_state");
    assert_eq!(fixture.spd.version, SPD_VERSION);
    assert_eq!(fixture.spd.commit, SPD_COMMIT);
    assert_eq!(
        fixture.lifecycle,
        "created_during_room_paint_and_stored_in_blob"
    );

    let seed = parse_seed(&fixture.input.seed).expect("seed");
    assert_eq!(seed.numeric, fixture.input.numeric);
    let mut dungeon = dungeon_from_run(init_run(seed.numeric));
    for depth in 1..fixture.input.depth {
        dungeon.depth = depth;
        create_level_partial(&mut dungeon);
    }
    dungeon.depth = fixture.input.depth;
    dungeon.sacrifice_parchment_scrap_level = Some(fixture.input.parchment_scrap_level);
    let floor = create_level_partial(&mut dungeon);
    let reward = floor
        .placed_items
        .iter()
        .find(|item| item.source.as_deref() == Some("SacrificeRoom"))
        .expect("blob-held reward is exposed by analyze path");
    assert_eq!(reward.class_name, fixture.reward.class_name);
    assert_eq!(reward.quantity, fixture.reward.quantity);
    assert_eq!(reward.level, fixture.reward.level);
    assert_eq!(reward.cursed, fixture.reward.cursed);
    assert_eq!(reward.enchantment, fixture.reward.enchantment);

    let report = floor.to_floor_report();
    if let Some(public) = report
        .items
        .iter()
        .find(|item| item.source.as_deref() == Some("SacrificeRoom"))
    {
        assert_eq!(public.class_name(), None);
        assert_eq!(public.level, None);
        assert_eq!(public.cursed, Some(true));
        assert_eq!(public.tier, None);
        assert_eq!(public.name, "weapon reward");
        assert_eq!(
            public.prediction,
            spd_core::report::ItemPredictionKind::Constrained
        );
        assert!(!public.name.contains("corrupting"));
    } else {
        assert!(
            report.rooms.is_empty(),
            "room contract needs a public layout"
        );
    }
    assert!(report
        .map
        .as_ref()
        .into_iter()
        .flat_map(|map| &map.heaps)
        .filter(|heap| heap.heap_type == "sacrificial")
        .all(|heap| heap.items.is_empty()));
    let public_json = serde_json::to_string(&report).expect("serialize public floor report");
    assert!(!public_json.contains(&fixture.reward.class_name));
    assert!(!public_json.contains("Corrupting"));
    assert!(!public_json.contains("+2"));
}
