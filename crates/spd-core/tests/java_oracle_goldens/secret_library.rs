use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    schema_version: u32,
    contract: String,
    spd: SpdPin,
    input: Input,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct SpdPin {
    version: String,
    commit: String,
}

#[derive(Deserialize)]
struct Input {
    seed: String,
    numeric: i64,
}

#[derive(Deserialize, PartialEq, Debug)]
struct Entry {
    #[serde(rename = "class")]
    class_name: String,
    weight: f32,
}

#[test]
fn secret_library_hash_map_order_is_pinned_by_java_oracle() {
    let _journal = crate::fresh_journal();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/java-oracle/fixtures/secret/secret-library-order.json");
    let fixture: Fixture = serde_json::from_str(&fs::read_to_string(path).expect("read fixture"))
        .expect("parse fixture");
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.contract, "secret-library-order");
    assert_eq!(fixture.spd.version, spd_core::SPD_VERSION);
    assert_eq!(fixture.spd.commit, spd_core::SPD_COMMIT);
    assert_eq!(fixture.input.seed, "GFX-PZH-DCH");
    assert_eq!(fixture.input.numeric, 1_300_416_271_343);
    let expected = [
        ("ScrollOfRecharging", 3.0),
        ("ScrollOfTerror", 4.0),
        ("ScrollOfMirrorImage", 3.0),
        ("ScrollOfRetribution", 4.0),
        ("ScrollOfTeleportation", 3.0),
        ("ScrollOfLullaby", 4.0),
        ("ScrollOfTransmutation", 6.0),
        ("ScrollOfMagicMapping", 4.0),
        ("ScrollOfIdentify", 1.0),
        ("ScrollOfRage", 4.0),
        ("ScrollOfRemoveCurse", 2.0),
    ];
    let actual: Vec<_> = fixture
        .entries
        .iter()
        .map(|entry| (entry.class_name.as_str(), entry.weight))
        .collect();
    assert_eq!(actual, expected);
}
