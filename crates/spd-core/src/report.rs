//! Serializable analysis report types.

mod base64_bytes;
mod bit_mask;
mod compact;
mod grouping;
mod quests;

use serde::{Deserialize, Serialize};

use spd_objects::{BlobId, Enchantment, ItemId, MobId, PlantId, RoomId, TrapId};

use crate::dungeon_seed::SeedError;
use crate::items::IdentityMaps;
use crate::trinkets::{
    ArtifactEvent, Challenge, ProfileError, TrinketEvent, TrinketSelectionReport,
};

pub(crate) use compact::is_variable_highlight;
pub(crate) use grouping::group_item_entries;
pub use quests::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedInfo {
    pub input: String,
    pub numeric: i64,
    /// Canonical `ABC-DEF-GHI` when representable.
    pub code: Option<String>,
    pub formatted: String,
    #[serde(default)]
    pub daily: bool,
}

/// Floor map for canvas rendering (SPD terrain IDs + tileset key).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FloorMap {
    pub width: u32,
    pub height: u32,
    /// Main-region or mining-branch tileset key used by the web renderer.
    pub tileset: String,
    /// Row-major SPD `Terrain` values
    pub tiles: Vec<u16>,
    /// SPD `DungeonTileSheet.setupVariance` values for deterministic tile
    /// alternates. Serialized as a base64 string to keep the payload small.
    #[serde(with = "base64_bytes")]
    pub tile_variance: Vec<u8>,
    /// Pinned `Level.cleanWalls()` visibility mask, row-major and parallel to
    /// `tiles`. Serialized as `0`/`1` to keep the per-cell payload small.
    #[serde(default, with = "bit_mask")]
    pub discoverable: Vec<bool>,
    /// Exact cells known to the partial generator, including depth-one ambient mobs.
    #[serde(default)]
    pub markers: Vec<MapMarker>,
    /// Exact placed heap cells, types, and ordered item stacks.
    #[serde(default)]
    pub heaps: Vec<MapHeap>,
    /// Exact placed mob cells and pinned Java class names.
    #[serde(default)]
    pub mobs: Vec<MapMob>,
    /// Pinned Java `LevelTransition` facts, sorted by center cell then type.
    #[serde(default)]
    pub transitions: Vec<MapTransition>,
    /// Pinned Java trap facts, sorted by cell.
    #[serde(default)]
    pub traps: Vec<MapTrap>,
    /// Pinned Java plant facts, sorted by cell. Empty until a covered painter plants one.
    #[serde(default)]
    pub plants: Vec<MapPlant>,
    /// Active pinned Java blob concentrations, sorted by class then cell.
    #[serde(default)]
    pub blobs: Vec<MapBlob>,
    /// Pinned custom terrain overlays, ordered by their visual layer.
    #[serde(default)]
    pub custom_tiles: Vec<MapCustomTile>,
    /// Pinned `Level.customTerrain` overlays (banners, etc.).
    #[serde(default)]
    pub custom_terrain: Vec<MapCustomTile>,
    /// Pinned custom wall overlays, ordered by their visual layer.
    #[serde(default)]
    pub custom_walls: Vec<MapCustomTile>,
    /// Sorted entity cells whose heap or mob can differ with run history or
    /// player state. Other entity cells are seed-exact for the profile.
    #[serde(default)]
    pub uncertain_cells: Vec<u32>,
    /// Sorted subset of `uncertain_cells` that differs on every run, even
    /// with the same history: vault consumables placed by an unseeded shuffle.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub per_run_cells: Vec<u32>,
    /// Sorted non-wall cells reachable from the arrival point only through a
    /// `SECRET_DOOR`, plus the secret doors themselves.
    #[serde(default)]
    pub hidden_cells: Vec<u32>,
    /// Sorted entity cells the hero sees only after searching: Ebony Mimics
    /// and Cracked Spyglass loot (`Heap.hidden`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub concealed_cells: Vec<u32>,
    /// Imp vault `VaultSentry` scan patterns, in mob placement order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sentry_scans: Vec<MapSentryScan>,
    /// Imp vault `VaultLaser` firing schedules; visual-only lasers are omitted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub laser_beams: Vec<MapLaserBeam>,
    /// Sorted entity cells that run history or player state can change:
    /// recorded on a possibly desynced level stream, drawn from a deck at an
    /// unknown position, or placed by player-dependent logic. Never serialized.
    #[serde(skip)]
    #[doc(hidden)]
    pub history_cells: Vec<u32>,
    /// Stable heap positions whose concrete equipment contents/properties are
    /// player-state-sensitive. Never serialized.
    #[serde(skip)]
    #[doc(hidden)]
    pub constrained_equipment_cells: Vec<u32>,
}

/// One sentry's scan cycle. It scans on turns 1, 1 + `cooldown`, …, using
/// `steps` in order; each step lists the cells that scan checks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapSentryScan {
    pub cell: u32,
    pub cooldown: u32,
    pub steps: Vec<Vec<u32>>,
}

/// One laser's schedule. It fires on turn `cooldown` (1-based), then `shots`
/// consecutive turns per cycle with `interval` turns from the last shot to the
/// next; `warning` targets `path` on the turn before each shot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapLaserBeam {
    pub cell: u32,
    pub cooldown: u32,
    pub interval: u32,
    pub shots: u32,
    pub warning: bool,
    /// Beam cells after the laser, ending at the collision cell.
    pub path: Vec<u32>,
}

/// A pinned Java `CustomTilemap` layer, including its resolved source tiles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapCustomTile {
    #[serde(rename = "class")]
    pub class_name: String,
    pub texture: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub static_data: Vec<i16>,
}

macro_rules! class_name_accessor {
    ($($ty:ty => $field:ident),* $(,)?) => {
        $(impl $ty {
            pub fn class_name(&self) -> &'static str {
                self.$field.class_name()
            }
        })*
    };
}

class_name_accessor! {
    MapHeapItem => item,
    MapMob => mob,
    MapTrap => trap,
    MapPlant => plant,
    MapBlob => blob,
    InitialEncounter => mob,
    PossibleRoom => room,
}

impl FloorMap {
    /// Terrain/layout projection. Entity placement is intentionally excluded.
    pub(crate) fn into_layout_only(mut self) -> Self {
        self.markers.clear();
        self.heaps.clear();
        self.mobs.clear();
        self.uncertain_cells.clear();
        self.concealed_cells.clear();
        self.history_cells.clear();
        self.constrained_equipment_cells.clear();
        self
    }

    /// Painter-complete layout plus the populated floor's heaps, mobs, and
    /// markers. Only entities that run history or player state can change
    /// are flagged uncertain.
    pub(crate) fn with_entities_from(mut self, populated: &FloorMap) -> Self {
        self.markers = populated.markers.clone();
        self.heaps = populated.heaps.clone();
        self.mobs = populated.mobs.clone();
        self.flatten_grass_under_entities(populated);
        self.concealed_cells = populated.concealed_cells.clone();
        let mut uncertain: Vec<u32> = populated
            .history_cells
            .iter()
            .chain(&populated.constrained_equipment_cells)
            .copied()
            .collect();
        uncertain.sort_unstable();
        uncertain.dedup();
        self.uncertain_cells = uncertain;
        self
    }

    /// `createMobs` / `createItems` set tall grass under each placed mob or
    /// drop to `GRASS`; keep that terrain wherever the entity is published.
    pub(crate) fn flatten_grass_under_entities(&mut self, populated: &FloorMap) {
        use crate::level::{FURROWED_GRASS, GRASS, HIGH_GRASS};
        let cells = populated
            .heaps
            .iter()
            .map(|heap| heap.cell)
            .chain(populated.mobs.iter().map(|mob| mob.cell));
        for cell in cells {
            let cell = cell as usize;
            let (Some(tile), Some(&after)) = (self.tiles.get_mut(cell), populated.tiles.get(cell))
            else {
                continue;
            };
            if matches!(i32::from(*tile), HIGH_GRASS | FURROWED_GRASS) && i32::from(after) == GRASS
            {
                *tile = after;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapHeap {
    pub cell: u32,
    pub heap_type: String,
    pub items: Vec<MapHeapItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapHeapItem {
    pub item: ItemId,
    pub quantity: i32,
    pub level: i32,
    pub cursed: bool,
    /// Weapon enchantment or armor glyph, curses included.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enchantment: Option<Enchantment>,
    /// Internal generation provenance for parity checks; never serialized.
    #[doc(hidden)]
    #[serde(skip)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapMob {
    pub cell: u32,
    pub mob: MobId,
    pub state: MapMobState,
    /// Items this mob drops when defeated. Empty for mobs that carry nothing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<MapHeapItem>,
}

/// `Mob.state` when the level is created, before the hero arrives.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MapMobState {
    Sleeping,
    Wandering,
    Passive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapTransition {
    pub cell: u32,
    #[serde(rename = "type")]
    pub transition_type: String,
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub dest_depth: i32,
    pub dest_branch: i32,
    pub dest_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapTrap {
    pub cell: u32,
    pub trap: TrapId,
    pub visible: bool,
    pub active: bool,
    /// Pinned `Trap` sprite color index.
    #[serde(default)]
    pub color: u8,
    /// Pinned `Trap` sprite shape index.
    #[serde(default)]
    pub shape: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapPlant {
    pub cell: u32,
    pub plant: PlantId,
    /// Pinned `Plant.image` sprite index.
    #[serde(default)]
    pub image: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapBlob {
    pub blob: BlobId,
    pub volume: u32,
    pub always_visible: bool,
    pub cells: Vec<MapBlobCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapBlobCell {
    pub cell: u32,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapMarker {
    /// Row-major index into `FloorMap.tiles`.
    pub cell: u32,
    pub kind: MapMarkerKind,
    pub label: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MapMarkerKind {
    Item,
    Mob,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FloorReport {
    pub depth: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feeling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builder: Option<String>,
    /// Room types selected by `initRooms` (order after shuffle).
    #[serde(default)]
    pub rooms: Vec<RoomId>,
    /// Room types that can occur in the modeled player/trinket profiles.
    ///
    /// `rooms` remains the no-extra-player-state room list.
    /// This field is the machine-readable projection when a profile can alter
    /// the layout. Each entry's conditions are alternatives (logical OR).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub possible_rooms: Vec<PossibleRoom>,
    /// Seed-determined non-loot features that are guaranteed to exist on this floor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guaranteed_appearances: Vec<GuaranteedAppearance>,
    /// Exact non-positional entity summary for the floor's initial generated state.
    /// Runtime summons and respawns are deliberately excluded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub initial_encounters: Vec<InitialEncounter>,
    /// Logical item spawns and their exact, constrained, or gameplay-variable variants.
    /// Only exact variants are finder evidence.
    pub items: Vec<ItemGroup>,
    pub quests: Vec<QuestReport>,
    /// Present when geometry build succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<FloorMap>,
    /// Layout produced by the analyzer's default-profile continuation after an
    /// unresolved player/meta-state branch. This is never an exact public
    /// prediction and must be presented with its assumption warning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumed_map: Option<FloorMap>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<BranchFloorReport>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchFloorId {
    pub depth: u32,
    pub branch: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BranchFloorKind {
    BlacksmithMine,
    ImpVault,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchAccessReport {
    pub quest_id: String,
    pub requires_acceptance: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_item: Option<ItemId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchFloorReport {
    pub id: BranchFloorId,
    pub origin: BranchFloorId,
    pub kind: BranchFloorKind,
    pub objective: String,
    pub access: BranchAccessReport,
    #[serde(default)]
    pub rooms: Vec<RoomId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<FloorMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumed_map: Option<FloorMap>,
}

impl FloorReport {
    pub fn item_variants(&self) -> impl Iterator<Item = &ItemEntry> {
        self.items.iter().flat_map(|group| &group.variants)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitialEncounter {
    pub mob: MobId,
    /// Internal English name; the wire carries `mob`.
    #[serde(skip)]
    pub name: String,
    pub quantity: u32,
    /// Base rewards associated with defeating this generated entity. Empty means
    /// the pinned class has no seed-analysis combat reward.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub combat_rewards: Vec<CombatReward>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CombatReward {
    /// Internal English name; the wire carries `item` and `category`.
    #[serde(skip)]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<ItemId>,
    pub category: String,
    /// Upgrade level of a reward generated with the floor, when above zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cursed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enchantment: Option<Enchantment>,
    pub prediction: CombatRewardPrediction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chance: Option<RewardChance>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CombatRewardPrediction {
    /// A fixed class reward on an ordinary eligible defeat.
    Guaranteed,
    /// The occurrence and/or identity is rolled on the gameplay RNG stream.
    RuntimeChance,
    /// The concrete carried reward was generated with the floor.
    GeneratedWithFloor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RewardChance {
    pub numerator: u32,
    pub denominator: u32,
}

/// A room type and count observed in one or more modeled generation profiles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PossibleRoom {
    pub room: RoomId,
    /// Number of rooms of this type for the associated profile condition.
    pub quantity: u32,
    /// Alternative player-state profiles under which this exact count occurs.
    /// An empty list means the modeled dependency axes do not change it.
    #[serde(
        default,
        skip_serializing_if = "spawn_conditions_empty_after_normalization",
        serialize_with = "serialize_spawn_conditions"
    )]
    pub spawn_conditions: Vec<ItemSpawnCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuaranteedAppearance {
    pub kind: GuaranteedAppearanceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<RoomId>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuaranteedAppearanceKind {
    AlchemyPot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemEntry {
    /// Internal English description used by compact text and tests. The wire
    /// carries `item` instead, or `label` when there is no single identity.
    #[serde(skip)]
    pub name: String,
    /// Description of an entry without a single item identity (room
    /// contracts, constrained rewards). Absent whenever `item` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Number of identical items represented by this entry.
    #[serde(default = "default_item_quantity")]
    pub quantity: i32,
    /// Inclusive total-count bounds when the seed fixes only a range;
    /// `quantity` is then the minimum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity_range: Option<NumericRange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<ItemId>,
    /// Ordered seed-determined identities when run history can shift a deck index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<ItemId>,
    pub category: String,
    /// Equipment tier when it is stable even though the concrete class is not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<i32>,
    /// Inclusive equipment-tier bounds when the tier is constrained but not exact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier_range: Option<NumericRange>,
    /// Current SPD item upgrade level (`0` is unupgraded).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    /// Inclusive upgrade-level bounds when the final level is constrained but not exact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level_range: Option<NumericRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursed: Option<bool>,
    /// Seed-determined enchantment or glyph, whether unconditional or conditional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enchantment: Option<ItemEnchantment>,
    pub prediction: ItemPredictionKind,
    /// Alternative player-state profiles under which this item may spawn.
    /// Any clause is sufficient; an empty list means the modeled dependency
    /// axes do not change this item.
    #[serde(
        default,
        skip_serializing_if = "spawn_conditions_empty_after_normalization",
        serialize_with = "serialize_spawn_conditions"
    )]
    pub spawn_conditions: Vec<ItemSpawnCondition>,
    /// Gate-only conditions that affect this item's presence or retained effect.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<ItemCondition>,
    /// Legacy internal diagnostics retained for parity tests; never serialized.
    #[serde(skip)]
    pub notes: Vec<String>,
    /// Internal provenance copied to `ItemGroup::source` in floor reports.
    #[serde(skip)]
    pub source: Option<String>,
}

impl ItemEntry {
    pub fn class_name(&self) -> Option<&'static str> {
        self.item.map(ItemId::class_name)
    }

    pub fn candidate_classes(&self) -> Vec<&'static str> {
        self.candidates
            .iter()
            .map(|item| item.class_name())
            .collect()
    }

    /// Fills the wire `label` for entries without a single identity.
    fn finalize_label(&mut self) {
        self.label = self.item.is_none().then(|| self.name.clone());
    }
}

/// Engine class name to public id. Unknown names map to `ItemId::Unknown`,
/// which the analyze sweep tests reject.
pub(crate) fn item_id(class_name: &str) -> ItemId {
    ItemId::from_class_name_lossy(class_name)
}

pub(crate) fn item_ids<S: AsRef<str>>(class_names: &[S]) -> Vec<ItemId> {
    class_names
        .iter()
        .map(|class| item_id(class.as_ref()))
        .collect()
}

/// One logical item spawn. Multiple variants are alternative projections of
/// that spawn, rather than additional items.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ItemGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub variants: Vec<ItemEntry>,
}

impl ItemGroup {
    pub fn single(mut item: ItemEntry) -> Self {
        item.finalize_label();
        Self {
            source: item.source.clone(),
            variants: vec![item],
        }
    }
}

impl From<ItemEntry> for ItemGroup {
    fn from(item: ItemEntry) -> Self {
        Self::single(item)
    }
}

impl std::ops::Deref for ItemGroup {
    type Target = ItemEntry;

    fn deref(&self) -> &Self::Target {
        self.variants
            .first()
            .expect("public item groups always contain a variant")
    }
}

impl<'de> Deserialize<'de> for ItemGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WireGroup {
            source: Option<String>,
            variants: Vec<ItemEntry>,
        }

        let mut wire = WireGroup::deserialize(deserializer)?;
        for variant in &mut wire.variants {
            variant.source = wire.source.clone();
        }
        Ok(Self {
            source: wire.source,
            variants: wire.variants,
        })
    }
}

const fn default_item_quantity() -> i32 {
    1
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct NumericRange {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemPredictionKind {
    /// Concrete identity and properties are safe to use for display and search.
    Exact,
    /// Concrete result this seed rolls when gameplay has not shifted the
    /// shared generation state. Player choices can change it.
    Variable,
    /// Only the explicitly populated constraints are seed-only guarantees.
    Constrained,
}

/// One conjunction in an item's possible-spawn condition. Multiple clauses on
/// an item are alternatives (logical OR).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemSpawnCondition {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub all_of: Vec<ItemDependencyCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemEnchantment {
    #[serde(rename = "type")]
    pub enchantment_type: Enchantment,
    pub conditions: Vec<ItemCondition>,
}

/// Typed gate conditions for item presence or retained effects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemCondition {
    Challenge {
        challenge: Challenge,
        enabled: bool,
    },
    Trinket {
        events: Vec<TrinketEvent>,
    },
    Artifact {
        events: Vec<ArtifactEvent>,
    },
    Quest {
        quest_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        depth: Option<u32>,
    },
    Choice {
        group_id: String,
        option_count: u32,
        selected_count: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        favor_requirement: Option<u32>,
    },
    Inventory {
        requirement_id: String,
    },
    Runtime {
        state_id: String,
    },
}

/// A condition axis that is allowed to affect public item presence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemDependencyCondition {
    Challenge {
        challenge: Challenge,
        enabled: bool,
    },
    /// An empty event list means no generation-affecting held trinket.
    Trinket {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        events: Vec<TrinketEvent>,
    },
    /// An empty event list means no external artifact history.
    Artifact {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        events: Vec<ArtifactEvent>,
    },
}

fn spawn_conditions_empty_after_normalization(conditions: &[ItemSpawnCondition]) -> bool {
    conditions.iter().all(|clause| {
        clause.all_of.iter().all(|dependency| {
            matches!(dependency, ItemDependencyCondition::Trinket { events } if events.is_empty())
        })
    })
}

fn serialize_spawn_conditions<S>(
    conditions: &[ItemSpawnCondition],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let normalized: Vec<ItemSpawnCondition> = conditions
        .iter()
        .filter_map(|clause| {
            let all_of = clause
                .all_of
                .iter()
                .filter(|dependency| {
                    !matches!(dependency, ItemDependencyCondition::Trinket { events } if events.is_empty())
                })
                .cloned()
                .collect::<Vec<_>>();
            (!all_of.is_empty()).then_some(ItemSpawnCondition { all_of })
        })
        .collect();
    normalized.serialize(serializer)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedReport {
    pub seed: SeedInfo,
    pub spd_version: String,
    pub spd_commit: String,
    pub floors_requested: u32,
    pub identities: IdentityMaps,
    pub trinket_selection: TrinketSelectionReport,
    pub floors: Vec<FloorReport>,
    /// `"partial"` while only forced drops exist; `"ok"` when full levelgen lands.
    pub status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error(transparent)]
    Seed(#[from] SeedError),
    #[error(transparent)]
    Profile(#[from] ProfileError),
}
