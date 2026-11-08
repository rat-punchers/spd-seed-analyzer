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
    selected: Vec<String>,
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
fn secret_laboratory_hash_map_order_is_pinned_by_java_oracle() {
    let _journal = crate::fresh_journal();
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/java-oracle/fixtures/secret/secret-laboratory-order.json");
    let fixture: Fixture = serde_json::from_str(&fs::read_to_string(path).expect("read fixture"))
        .expect("parse fixture");
    assert_eq!(fixture.schema_version, 1);
    assert_eq!(fixture.contract, "secret-laboratory-order");
    assert_eq!(fixture.spd.version, spd_core::SPD_VERSION);
    assert_eq!(fixture.spd.commit, spd_core::SPD_COMMIT);
    assert_eq!(fixture.input.seed, "AAA-AAA-AAA");
    assert_eq!(fixture.input.numeric, 0);
    let expected = [
        ("PotionOfParalyticGas", 4.0),
        ("PotionOfLevitation", 4.0),
        ("PotionOfInvisibility", 4.0),
        ("PotionOfExperience", 6.0),
        ("PotionOfHaste", 4.0),
        ("PotionOfPurity", 4.0),
        ("PotionOfHealing", 1.0),
        ("PotionOfMindVision", 2.0),
        ("PotionOfFrost", 3.0),
        ("PotionOfLiquidFlame", 3.0),
        ("PotionOfToxicGas", 3.0),
    ];
    let actual: Vec<_> = fixture
        .entries
        .iter()
        .map(|entry| (entry.class_name.as_str(), entry.weight))
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(fixture.selected, ["PotionOfFrost", "PotionOfHaste"]);
}
