import type {
  BlobId,
  Enchantment,
  ItemId,
  MobId,
  PlantId,
  RoomId,
  TrapId,
} from '@/generated/game-ids'
import type { WandmakerRewardSlot } from '@/lib/wandmaker-reward'
import init, * as wasmBindings from '@/wasm/spd_wasm'

export type { WandmakerRewardSlot }

export type SeedInfo = {
  input: string
  numeric: number
  code: string | null
  formatted: string
  daily: boolean
}

export type IdentityEntry = {
  item: ItemId
  appearance: string
}

export type IdentityMaps = {
  potions: IdentityEntry[]
  scrolls: IdentityEntry[]
  rings: IdentityEntry[]
}

export type ItemVariant = {
  /** Engine description, present only when `item` is absent. */
  label?: string
  /** Number of identical items represented by this entry. */
  quantity: number
  /** Total-count bounds when the seed fixes only a range (`quantity` is the minimum). */
  quantity_range?: { min: number; max: number } | null
  /** Concrete identity, when the entry has exactly one. */
  item?: ItemId
  /** Ordered seed-determined identities when run history can shift a deck index. */
  candidates?: ItemId[]
  category: string
  tier?: number | null
  tier_range?: { min: number; max: number } | null
  level?: number | null
  level_range?: { min: number; max: number } | null
  /** Present when the item is cursed (chip in item list). */
  cursed?: boolean | null
  /** Seed-determined enchantment or glyph, with any retention conditions. */
  enchantment?: ItemEnchantment | null
  prediction: 'exact' | 'variable' | 'constrained'
  /** Alternative dependency clauses; any one clause may produce this variant. */
  spawn_conditions?: ItemSpawnCondition[]
  conditions?: ItemCondition[]
}

export type ItemGroup = {
  source?: string | null
  /** Alternative projections of one logical spawn, not additional items. */
  variants: ItemVariant[]
}

export type Challenge =
  | 'champion_enemies'
  | 'badder_bosses'
  | 'on_diet'
  | 'faith_is_my_armor'
  | 'pharmacophobia'
  | 'barren_land'
  | 'swarm_intelligence'
  | 'into_darkness'
  | 'forbidden_runes'

export type TrinketEvent = {
  before_depth: number
  kind: 'acquired' | 'upgraded' | 'transmuted'
  trinket?: string
  min_upgrades?: number
}

export type ArtifactEvent = {
  before_depth: number
  kind: 'obtained' | 'transmuted'
  artifact: string
}

export type ItemDependencyCondition =
  | { type: 'challenge'; challenge: Challenge; enabled: boolean }
  | { type: 'trinket'; events?: TrinketEvent[] }
  | { type: 'artifact'; events?: ArtifactEvent[] }

export type ItemSpawnCondition = {
  all_of?: ItemDependencyCondition[]
}

export type ItemCondition =
  | { type: 'challenge'; challenge: Challenge; enabled: boolean }
  | { type: 'trinket'; events: TrinketEvent[] }
  | { type: 'artifact'; events: ArtifactEvent[] }
  | { type: 'quest'; quest_id: string; depth?: number }
  | {
      type: 'choice'
      group_id: string
      option_count: number
      selected_count: number
      favor_requirement?: number
    }
  | { type: 'inventory'; requirement_id: string }
  | { type: 'runtime'; state_id: string }

export type ItemEnchantment = {
  type: Enchantment
  conditions: ItemCondition[]
}

/** Per-cell boolean mask sent as `0`/`1` numbers to keep reports small. */
export type CellMask = ReadonlyArray<0 | 1>

export type FloorMap = {
  width: number
  height: number
  tileset: string
  tiles: number[]
  /** Sent as base64; decoded by `analyzeSeed`. */
  tile_variance: ArrayLike<number>
  /** Per-cell `cleanWalls()` visibility, `1` = discoverable. */
  discoverable: CellMask
  markers: MapMarker[]
  heaps: MapHeap[]
  mobs: MapMob[]
  transitions: MapTransition[]
  traps: MapTrap[]
  plants: MapPlant[]
  blobs: MapBlob[]
  custom_tiles: MapCustomTile[]
  custom_terrain?: MapCustomTile[]
  custom_walls: MapCustomTile[]
  /** Entity cells whose heap or mob can differ with run history. */
  uncertain_cells?: number[]
  /** Subset of `uncertain_cells` that differs on every run (unseeded shuffle). */
  per_run_cells?: number[]
  /** Cells reachable only through a secret door, plus the doors. */
  hidden_cells?: number[]
  /** Entities seen only after searching: Ebony Mimics, Cracked Spyglass loot. */
  concealed_cells?: number[]
  /** Imp vault sentry scans; omitted when the floor has none. */
  sentry_scans?: MapSentryScan[]
  /** Imp vault laser schedules; visual-only lasers are omitted. */
  laser_beams?: MapLaserBeam[]
}

/**
 * One `VaultLaser`: it fires on turn `cooldown`, `shots` turns in a row per
 * cycle, then waits `interval` turns from the last shot to the next. With
 * `warning`, `path` is targeted on the turn before each shot.
 */
export type MapLaserBeam = {
  cell: number
  cooldown: number
  interval: number
  shots: number
  warning: boolean
  /** Beam cells after the laser, ending at the collision cell. */
  path: number[]
}

/**
 * One `VaultSentry` scan cycle: it scans on turns 1, 1 + `cooldown`, …,
 * using `steps` in order. Each step lists the cells that scan checks.
 */
export type MapSentryScan = {
  cell: number
  cooldown: number
  steps: number[][]
}

/** A pinned static `CustomTilemap` layer from the game level. */
export type MapCustomTile = {
  class: string
  texture: string
  x: number
  y: number
  width: number
  height: number
  static_data: number[]
}

export type MapMarkerKind = 'item' | 'mob'

export type MapMarker = {
  cell: number
  kind: MapMarkerKind
  label: string
}

export type MapHeap = {
  cell: number
  heap_type: string
  items: MapHeapItem[]
}

export type MapHeapItem = {
  item: ItemId
  quantity: number
  level: number
  cursed: boolean
  /** Weapon enchantment or armor glyph, curses included. */
  enchantment?: Enchantment
}

export type MapMobState = 'sleeping' | 'wandering' | 'passive'

export type MapMob = {
  cell: number
  mob: MobId
  /** `Mob.state` at level creation, before the hero arrives. */
  state: MapMobState
  /** Items this mob drops when defeated. */
  items?: MapHeapItem[]
  /** Render-only sprite alpha set by the concealed map layer; never sent by WASM. */
  opacity?: number
}

export type MapTransition = {
  cell: number
  type: string
  left: number
  top: number
  right: number
  bottom: number
  dest_depth: number
  dest_branch: number
  dest_type: string | null
}

export type MapTrap = {
  cell: number
  trap: TrapId
  visible: boolean
  active: boolean
  color: number
  shape: number
}

export type MapPlant = {
  cell: number
  plant: PlantId
  image: number
}

export type MapBlobCell = {
  cell: number
  value: number
}

export type MapBlob = {
  blob: BlobId
  volume: number
  always_visible: boolean
  cells: MapBlobCell[]
}

export type FloorReport = {
  depth: number
  feeling?: string | null
  builder?: string | null
  rooms?: RoomId[]
  /** Alternate room counts produced by modeled player/trinket profiles. */
  possible_rooms?: PossibleRoom[]
  guaranteed_appearances?: GuaranteedAppearance[]
  initial_encounters?: InitialEncounter[]
  /** Includes exact, constrained, and gameplay-variable entries. */
  items: ItemGroup[]
  quests: QuestReport[]
  map?: FloorMap | null
  assumed_map?: FloorMap | null
  branches?: BranchFloorReport[]
}

export type PossibleRoom = {
  room: RoomId
  quantity: number
  spawn_conditions?: ItemSpawnCondition[]
}

export type BranchFloorId = {
  depth: number
  branch: number
}

export type BranchFloorReport = {
  id: BranchFloorId
  origin: BranchFloorId
  kind: 'blacksmith_mine' | 'imp_vault'
  objective: string
  access: {
    quest_id: string
    requires_acceptance: boolean
    required_item?: ItemId
  }
  rooms: RoomId[]
  map?: FloorMap | null
  assumed_map?: FloorMap | null
}

export type InitialEncounter = {
  mob: MobId
  quantity: number
  combat_rewards?: CombatReward[]
}

export type CombatReward = {
  item?: ItemId
  category: string
  /** Upgrade level of a reward generated with the floor, when above zero. */
  level?: number
  cursed?: boolean
  enchantment?: Enchantment
  prediction: 'guaranteed' | 'runtime_chance' | 'generated_with_floor'
  chance?: { numerator: number; denominator: number } | null
}

export type QuestDepthRange = {
  min: number
  max: number
}

export type QuestRewardSelection = {
  item_source: string
  option_count: number
  selected_count: number
  favor_requirement?: number | null
}

export type GhostTarget = 'fetid_rat' | 'gnoll_trickster' | 'great_crab'

export type WandmakerObjective = 'corpse_dust' | 'elemental_embers' | 'rotberry'

export type BlacksmithObjective = 'crystal' | 'gnoll'
export type QuestReport =
  | {
      type: 'sad_ghost'
      contract: {
        spawn_depth_range: QuestDepthRange
        target_rules: { spawn_depth: number; target: GhostTarget }[]
        rewards: QuestRewardSelection
      }
      variable: { target: GhostTarget }
    }
  | {
      type: 'old_wandmaker'
      contract: {
        spawn_depth_range: QuestDepthRange
        objective_options: WandmakerObjective[]
        rewards: QuestRewardSelection
        /** Wand 1 then wand 2. Absent when the reward draw did not run. */
        slots?: WandmakerRewardSlot[]
      }
      variable: { objective: WandmakerObjective }
    }
  | {
      type: 'troll_blacksmith'
      contract: {
        spawn_depth_range: QuestDepthRange
        objective_options: BlacksmithObjective[]
        rewards: QuestRewardSelection
      }
      variable: { objective: BlacksmithObjective }
    }
  | {
      type: 'ambitious_imp'
      contract: {
        spawn_depth_range: QuestDepthRange
        rewards: QuestRewardSelection
        slots?: ImpRewardSlot[]
        /**
         * True when no reachable deck drift changes any slot's upgrade or
         * enchant, making every slot `level` a seed-only guarantee.
         */
        upgrades_pinned?: boolean
        /** Scroll of Upgrade equivalents across all six options. */
        total_upgrade_value?: number
      }
      variable: { spawn_depth: number }
    }

export type ImpSlotRole =
  | 'artifact'
  | 'ring'
  | 'weapon'
  | 'missile'
  | 'armor'
  | 'wand'

/** `Random.IntRange(min, max)`, or `Artifact.transferUpgrade(transfer)`. */
export type ImpLevelRule =
  | { type: 'range'; min: number; max: number }
  | { type: 'transfer_upgrade'; transfer: number }

export type ImpSlotCandidate = {
  item: ItemId
  /** Deck-index offsets from the seed draw, nearest first. 0 is the seed draw. */
  drifts: number[]
  weight: number
}

export type ImpRewardSlot = {
  slot: number
  role: ImpSlotRole
  /** The seed-drawn class. */
  item: ItemId
  level_rule: ImpLevelRule
  enchanted: boolean
  /** The rolled upgrade. Pinned by the seed unless `upgrades_pinned` is false. */
  level: number
  /** The rolled enchantment or glyph, pinned on the same stream as the level. */
  enchantment?: Enchantment
  /** Set only when the class cannot vary (the plate armor slot). */
  fixed_item?: ItemId
  candidates?: ImpSlotCandidate[]
  /** Denominator for candidate weights. */
  scenario_count: number
}

export type GuaranteedAppearance = {
  kind: 'alchemy_pot'
  source?: RoomId
}

export type SeedReport = {
  seed: SeedInfo
  spd_version: string
  spd_commit: string
  floors_requested: number
  identities: IdentityMaps
  trinket_selection: TrinketSelectionReport
  floors: FloorReport[]
  status: string
}

export type TrinketSelectionReport = {
  catalyst_depth: number
  first_alchemy_pot_depth: number
  first_alchemy_pot_is_secret: boolean
  selection_depth: number
  first_effective_depth: number
  catalyst_options: ItemId[]
  /** Index-aligned with `catalyst_options`. */
  catalyst_option_effects: TrinketGenerationEffect[]
  transmutation_sequence: ItemId[]
}

/** Whether holding a trinket changes generated floors, and whether it is modeled. */
export type TrinketGenerationEffect = 'modeled' | 'unmodeled' | 'none'

export type SeedSearchMatchMode = 'any' | 'all'

export type SeedSearchConstraint = {
  itemGroup: string
  item?: ItemId | null
  minLevel: number | null
  minDepth: number
  maxDepth: number
}

export type SeedSearchRequest = {
  startSeed: number
  candidateCount: number
  floors: number
  constraints: SeedSearchConstraint[]
  matchMode: SeedSearchMatchMode
  includeVariable?: boolean
  /**
   * Widen quest-reward matching to every class a deck-index shift can reach.
   * A hit then means "reachable in some run", not "present when gameplay is
   * unchanged".
   */
  deepSearch?: boolean
  maxMatches: number
}

export type SeedSearchEvidence = {
  constraintIndex: number
  itemGroup: string
  item?: ItemId
  depth: number
  /** Present only when the matched entry has no single identity. */
  label?: string
  level: number
  prediction: ItemVariant['prediction']
  source?: string | null
}

export type SeedSearchMatch = {
  seed: SeedInfo
  identities: IdentityMaps
  evidence: SeedSearchEvidence[]
  variableItems: SeedSearchVariableItem[]
}

export type SeedSearchVariableItem = ItemVariant & {
  depth: number
  source?: string | null
}

export type SeedSearchResult = {
  startSeed: number
  requestedCandidates: number
  candidatesScanned: number
  nextSeed: number | null
  exhausted: boolean
  matchLimitReached: boolean
  matchMode: SeedSearchMatchMode
  matches: SeedSearchMatch[]
  status: string
  message: string
}

type SearchBinding = (request: SeedSearchRequest) => SeedSearchResult

let ready: Promise<void> | null = null

export function ensureWasm(): Promise<void> {
  if (!ready) {
    ready = init().then(() => undefined)
  }
  return ready
}

export async function parseSeed(input: string): Promise<SeedInfo> {
  await ensureWasm()
  return wasmBindings.parse_seed(input) as SeedInfo
}

/**
 * `trinket` is a Catalyst option class name held at +3; `challenges` are the
 * run's enabled challenges; `lorePages` replays a fresh journal, so guide and
 * alchemy pages spawn. None set is the default report (complete journal).
 */
export async function analyzeSeed(
  input: string,
  floors: number,
  trinket: string | null = null,
  challenges: readonly Challenge[] = [],
  lorePages = false
): Promise<SeedReport> {
  await ensureWasm()
  const report = (
    trinket == null && challenges.length === 0 && !lorePages
      ? wasmBindings.analyze_seed(input, floors)
      : wasmBindings.analyze_seed_for_run(
          input,
          floors,
          trinket ?? undefined,
          [...challenges],
          !lorePages
        )
  ) as SeedReport
  for (const floor of report.floors) {
    decodeMaps(floor)
    for (const branch of floor.branches ?? []) decodeMaps(branch)
  }
  return report
}

function decodeMaps(holder: {
  map?: FloorMap | null
  assumed_map?: FloorMap | null
}) {
  for (const map of [holder.map, holder.assumed_map]) {
    if (map && typeof map.tile_variance === 'string') {
      map.tile_variance = Uint8Array.from(atob(map.tile_variance), (char) =>
        char.charCodeAt(0)
      )
    }
  }
}

export async function searchSeeds(
  request: SeedSearchRequest
): Promise<SeedSearchResult> {
  await ensureWasm()
  const search = (wasmBindings as unknown as { search_seeds?: SearchBinding })
    .search_seeds
  if (!search) {
    throw new Error('Seed search is unavailable. Rebuild the WASM package.')
  }
  return search(request)
}

export async function getSpdMeta(): Promise<{
  version: string
  commit: string
}> {
  await ensureWasm()
  return {
    version: wasmBindings.spd_version(),
    commit: wasmBindings.spd_commit(),
  }
}
