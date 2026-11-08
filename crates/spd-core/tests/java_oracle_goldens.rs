use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use spd_core::items::identities::IdentityEntry;
use spd_core::level::create_level_partial;
use spd_core::{analyze_seed, dungeon_from_run, init_run, parse_seed, SPD_COMMIT, SPD_VERSION};

const IDENTITY_SCHEMA_VERSION: u32 = 1;
const FLOOR_SCHEMA_VERSION: u32 = 2;
const FINAL_HEAPS_SCHEMA_VERSION: u32 = 3;
const EXTENDED_FINAL_HEAPS_SCHEMA_VERSION: u32 = 4;

#[path = "java_oracle_goldens/crystal_path.rs"]
mod crystal_path;
#[path = "java_oracle_goldens/final_heaps.rs"]
mod final_heaps;
#[path = "java_oracle_goldens/pool.rs"]
mod pool;
#[path = "java_oracle_goldens/sacrifice_reward.rs"]
mod sacrifice_reward;
#[path = "java_oracle_goldens/secret_laboratory.rs"]
mod secret_laboratory;
#[path = "java_oracle_goldens/secret_library.rs"]
mod secret_library;
#[path = "java_oracle_goldens/toxic_gas.rs"]
mod toxic_gas;

/// The oracle runs on a fresh journal (only the intro page read), so guide
/// and alchemy pages spawn and consume RNG.
fn fresh_journal() -> spd_core::journal::LoreScope {
    spd_core::journal::scope_lore_complete(false)
}

#[derive(Debug, Deserialize)]
struct OracleFixture {
    schema_version: u32,
    #[serde(default)]
    contract: Option<String>,
    spd: SpdPin,
    input: OracleInput,
    identities: OracleIdentities,
    #[serde(default)]
    floors: Vec<OracleFloor>,
}

#[derive(Debug, Deserialize)]
struct SpdPin {
    version: String,
    commit: String,
}

#[derive(Debug, Deserialize)]
struct OracleInput {
    seed: String,
    numeric: i64,
    #[serde(default)]
    depths: Vec<u32>,
    #[serde(default)]
    challenge: Option<String>,
    #[serde(default)]
    trinket: Option<String>,
    #[serde(default)]
    trinket_level: Option<u8>,
    #[serde(default)]
    trinket_before_depth: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OracleIdentities {
    potions: Vec<OracleIdentity>,
    scrolls: Vec<OracleIdentity>,
    rings: Vec<OracleIdentity>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OracleIdentity {
    item: String,
    appearance: String,
}

#[derive(Debug, Deserialize)]
struct OracleFloor {
    depth: u32,
    #[serde(default)]
    width: u32,
    #[serde(default)]
    height: u32,
    #[serde(default)]
    pre_paint_rng: Vec<i32>,
    #[serde(default)]
    pre_doors_rng: Vec<i32>,
    #[serde(default)]
    post_doors_rng: Vec<i32>,
    #[serde(default)]
    pre_mobs_rng: Vec<i32>,
    #[serde(default)]
    pre_items_rng: Vec<i32>,
    #[serde(default)]
    rooms: Vec<String>,
    #[serde(default)]
    room_bounds: Vec<OracleRoomFact>,
    #[serde(default)]
    forced_items: Vec<OracleItem>,
    #[serde(default)]
    final_heaps: Vec<OracleHeap>,
    #[serde(default)]
    final_mobs: Vec<OracleMob>,
    #[serde(default)]
    quest_rewards: Vec<OracleItem>,
    terrain: Option<Vec<u16>>,
    discoverable: Option<Vec<bool>>,
    tile_variance: Option<Vec<u8>>,
    transitions: Option<Vec<OracleTransition>>,
    traps: Option<Vec<OracleTrap>>,
    plants: Option<Vec<OraclePlant>>,
    blobs: Option<Vec<OracleBlob>>,
    #[serde(default)]
    custom_tiles: Vec<OracleCustomTile>,
    #[allow(dead_code)] // parse-only until layout equality is asserted
    #[serde(default)]
    custom_terrain: Vec<OracleCustomTile>,
    #[serde(default)]
    custom_walls: Vec<OracleCustomTile>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct OracleRoomFact {
    #[serde(rename = "class")]
    class_name: String,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
struct OracleItem {
    #[serde(rename = "class")]
    class_name: String,
    quantity: i32,
    level: i32,
    cursed: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone)]
struct OracleHeap {
    cell: u32,
    heap_type: String,
    items: Vec<OracleItem>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct OracleMob {
    cell: u32,
    #[serde(rename = "class")]
    class_name: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OracleTransition {
    cell: u32,
    #[serde(rename = "type")]
    transition_type: String,
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
    dest_depth: i32,
    dest_branch: i32,
    dest_type: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OracleTrap {
    cell: u32,
    #[serde(rename = "class")]
    class_name: String,
    visible: bool,
    active: bool,
    #[serde(default)]
    color: u8,
    #[serde(default)]
    shape: u8,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OraclePlant {
    cell: u32,
    #[serde(rename = "class")]
    class_name: String,
    #[serde(default)]
    image: u8,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OracleBlob {
    #[serde(rename = "class")]
    class_name: String,
    #[serde(default)]
    volume: u32,
    #[serde(default)]
    always_visible: bool,
    cells: Vec<OracleBlobCell>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OracleBlobCell {
    cell: u32,
    value: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct OracleCustomTile {
    #[serde(rename = "class")]
    class_name: String,
    #[serde(default)]
    texture: Option<String>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    static_data: Vec<i16>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ComparableItem {
    class_name: String,
    cursed: bool,
}

fn fixture_paths() -> Vec<PathBuf> {
    let fixture_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/java-oracle/fixtures");
    let mut paths: Vec<_> = fs::read_dir(&fixture_dir)
        .unwrap_or_else(|error| {
            panic!(
                "failed to read Java oracle fixtures at {}: {error}",
                fixture_dir.display()
            )
        })
        .map(|entry| entry.expect("read fixture directory entry").path())
        .filter(|path| {
            path.file_name().is_some_and(|name| {
                name.to_string_lossy().ends_with(".json")
                    && name.to_string_lossy().contains("-final-heaps-floor-")
            })
        })
        .collect();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "no Java oracle JSON fixtures found at {}",
        fixture_dir.display()
    );
    paths
}

fn legacy_floor_fixture_paths() -> Vec<PathBuf> {
    let fixture_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/java-oracle/fixtures");
    fs::read_dir(&fixture_dir)
        .unwrap_or_else(|error| {
            panic!(
                "failed to read Java oracle fixtures at {}: {error}",
                fixture_dir.display()
            )
        })
        .map(|entry| entry.expect("read fixture directory entry").path())
        .filter(|path| {
            path.file_name().is_some_and(|name| {
                name.to_string_lossy().ends_with("-floor-1.json")
                    && !name.to_string_lossy().contains("-final-heaps-")
            })
        })
        .collect()
}

fn read_fixture(path: &Path) -> OracleFixture {
    let json = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&json)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn identity_pairs(entries: &[IdentityEntry]) -> Vec<OracleIdentity> {
    entries
        .iter()
        .map(|entry| OracleIdentity {
            item: entry.item.class_name().to_string(),
            appearance: entry.appearance.clone(),
        })
        .collect()
}

#[test]
fn run_identities_match_pinned_java_oracle() {
    let _journal = crate::fresh_journal();
    for path in fixture_paths() {
        let fixture = read_fixture(&path);
        let context = path.display();

        assert!(
            matches!(
                fixture.schema_version,
                IDENTITY_SCHEMA_VERSION
                    | FLOOR_SCHEMA_VERSION
                    | FINAL_HEAPS_SCHEMA_VERSION
                    | EXTENDED_FINAL_HEAPS_SCHEMA_VERSION
            ),
            "supported schema version in {context}"
        );
        assert_eq!(fixture.spd.version, SPD_VERSION, "SPD version in {context}");
        assert_eq!(fixture.spd.commit, SPD_COMMIT, "SPD commit in {context}");
        if fixture.schema_version == IDENTITY_SCHEMA_VERSION {
            assert!(
                fixture.contract.is_none()
                    && fixture.input.depths.is_empty()
                    && fixture.floors.is_empty(),
                "schema v1 identity fixture must not contain floors in {context}"
            );
        }

        let parsed = parse_seed(&fixture.input.seed)
            .unwrap_or_else(|error| panic!("invalid seed in {context}: {error}"));
        assert_eq!(
            parsed.numeric, fixture.input.numeric,
            "numeric seed in {context}"
        );

        let actual = init_run(fixture.input.numeric).identities;
        assert_eq!(
            identity_pairs(&actual.potions),
            fixture.identities.potions,
            "potion identities in {context}"
        );
        assert_eq!(
            identity_pairs(&actual.scrolls),
            fixture.identities.scrolls,
            "scroll identities in {context}"
        );
        assert_eq!(
            identity_pairs(&actual.rings),
            fixture.identities.rings,
            "ring identities in {context}"
        );
    }
}

#[test]
fn depth_one_forced_items_match_pinned_java_oracle() {
    let _journal = crate::fresh_journal();
    let mut compared = 0;
    for path in legacy_floor_fixture_paths() {
        let fixture = read_fixture(&path);
        if fixture.schema_version != FLOOR_SCHEMA_VERSION {
            continue;
        }
        let context = path.display();
        assert_eq!(fixture.input.depths, [1], "requested depths in {context}");
        assert_eq!(fixture.floors.len(), 1, "floor count in {context}");
        let expected_floor = &fixture.floors[0];
        assert_eq!(expected_floor.depth, 1, "floor depth in {context}");
        assert!(
            expected_floor
                .forced_items
                .iter()
                .all(|item| item.quantity == 1 && item.level == 0),
            "the first forced-item contract covers unit, level-zero items in {context}"
        );

        let expected: Vec<_> = expected_floor
            .forced_items
            .iter()
            .map(|item| ComparableItem {
                class_name: item.class_name.clone(),
                cursed: item.cursed,
            })
            .collect();
        let mut dungeon = dungeon_from_run(init_run(fixture.input.numeric));
        dungeon.depth = 1;
        let floor = create_level_partial(&mut dungeon);
        let actual: Vec<_> = floor
            .initial_forced_items
            .iter()
            .map(|item| ComparableItem {
                class_name: item.class_name.clone(),
                cursed: item.cursed,
            })
            .collect();
        assert_eq!(actual, expected, "ordered forced items in {context}");
        let report = floor.to_floor_report();
        assert!(
            report
                .items
                .iter()
                .all(|item| item.source.as_deref() != Some("forced")),
            "sampled forced items stay internal in {context}"
        );
        compared += 1;
    }
    assert!(compared > 0, "no schema v2 floor fixtures were compared");
}
