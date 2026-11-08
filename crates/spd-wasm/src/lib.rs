//! WASM bindings for `spd-core`.

use spd_core::{
    analyze_seed as core_analyze, analyze_seed_for_run as core_analyze_for_run,
    analyze_seed_with_trinket as core_analyze_with_trinket, parse_seed as core_parse,
    search_seeds as core_search, AnalyzeError, Challenge, SearchError, SeedError,
    SeedSearchRequest, TrinketKind,
};
use wasm_bindgen::prelude::*;

/// Uses serde_json before crossing the WASM boundary so Serde's flattened,
/// internally-tagged event enums retain their fields in JavaScript objects.
fn json_value<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let json =
        serde_json::to_string(value).map_err(|error| JsValue::from_str(&error.to_string()))?;
    js_sys::JSON::parse(&json)
}

fn seed_err(e: SeedError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn analyze_err(e: AnalyzeError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn search_err(e: SearchError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

/// Initialize panic hook for better console errors in the browser.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Parse a seed string into `{ input, numeric, code, formatted }`.
#[wasm_bindgen]
pub fn parse_seed(input: &str) -> Result<JsValue, JsValue> {
    let info = core_parse(input).map_err(seed_err)?;
    json_value(&info)
}

/// Analyze a seed for `floors` depths. Returns a `SeedReport` object.
#[wasm_bindgen]
pub fn analyze_seed(input: &str, floors: u32) -> Result<JsValue, JsValue> {
    let report = core_analyze(input, floors).map_err(analyze_err)?;
    json_value(&report)
}

/// Analyze a seed with a Catalyst trinket (class name, e.g. `MossyClump`) held
/// at +3 from the first floor it can influence. `None` matches `analyze_seed`.
#[wasm_bindgen]
pub fn analyze_seed_with_trinket(
    input: &str,
    floors: u32,
    trinket: Option<String>,
) -> Result<JsValue, JsValue> {
    let trinket = trinket
        .map(|class_name| {
            TrinketKind::from_class_name(&class_name)
                .ok_or_else(|| JsValue::from_str(&format!("unknown trinket: {class_name}")))
        })
        .transpose()?;
    let report = core_analyze_with_trinket(input, floors, trinket).map_err(analyze_err)?;
    json_value(&report)
}

/// Analyze a seed for a run with `challenges` enabled (snake_case names, e.g.
/// `forbidden_runes`) and an optional Catalyst trinket class name.
/// `lore_complete: false` replays a fresh journal, whose guide and alchemy
/// pages spawn. An empty list with a complete journal matches
/// `analyze_seed_with_trinket`.
#[wasm_bindgen]
pub fn analyze_seed_for_run(
    input: &str,
    floors: u32,
    trinket: Option<String>,
    challenges: JsValue,
    lore_complete: bool,
) -> Result<JsValue, JsValue> {
    let trinket = trinket
        .map(|class_name| {
            TrinketKind::from_class_name(&class_name)
                .ok_or_else(|| JsValue::from_str(&format!("unknown trinket: {class_name}")))
        })
        .transpose()?;
    let challenges: Vec<Challenge> = serde_wasm_bindgen::from_value(challenges)
        .map_err(|e| JsValue::from_str(&format!("invalid challenges: {e}")))?;
    let report = core_analyze_for_run(input, floors, trinket, &challenges, lore_complete)
        .map_err(analyze_err)?;
    json_value(&report)
}

/// Search a bounded, resumable chunk of numeric seeds using item constraints.
#[wasm_bindgen]
pub fn search_seeds(request: JsValue) -> Result<JsValue, JsValue> {
    let request: SeedSearchRequest = serde_wasm_bindgen::from_value(request)
        .map_err(|e| JsValue::from_str(&format!("invalid search request: {e}")))?;
    let result = core_search(&request).map_err(search_err)?;
    json_value(&result)
}

/// Pinned SPD version string.
#[wasm_bindgen]
pub fn spd_version() -> String {
    spd_core::SPD_VERSION.to_string()
}

/// Pinned SPD commit short hash.
#[wasm_bindgen]
pub fn spd_commit() -> String {
    spd_core::SPD_COMMIT.to_string()
}
