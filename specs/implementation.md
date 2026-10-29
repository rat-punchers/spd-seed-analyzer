# Implementation status

The analyzer renders the Troll Blacksmith's Crystal or Gnoll mine as the
once-through painter-complete map for the seed. Branch maps appear inside the
origin floor's quest panel, with reciprocal transitions, access conditions,
objective-specific tilesets, and painter-complete layout. Rust generation
matches pinned Java painter fixtures exactly; browser snapshots cover both
objectives. Entry still requires accepting the quest, carrying the Pickaxe,
and confirming travel.

The overall analyzer remains partial.

Main-path floor maps publish heaps, mobs, and loot markers with an inspect
panel (hover/tap). `uncertain_cells` marks "May differ" per entity: only
entities recorded on a level stream that may have desynced (a levelgen
artifact draw after depth 1, any deck draw once an earlier floor desynced, a
history-dependent quest roll) or placed by player-dependent logic (shop stock,
the createItems tail after rose/rations, Champion Giants). Rules and evidence:
`specs/generator-decks.md` §11 "Map entity certainty"; a perturbation test
replays runtime artifact draws and asserts exact entities do not move. Sewer
floors are mostly exact; from the first shop floor on most seeds flag nearly
everything. `hidden_cells` (secret-door-only areas) pulse on the canvas. Boss
maps publish their level-creation entities (Goo, Rat King and gold; prison
cages and IronKey; Pylons; the Amulet), always exact, plus the depth-20 Imp
shop as uncertain cells. Boss terrain keeps its decorations and the Java
custom layers; depth 25 shows the exit opened
(`specs/analysis/boss-floor-entities.md`).

Vault maps publish room-painted entities plus `VaultLevel.createItems`
(leftover `itemsToSpawn`, Darkness torches, extra Potion of Healing), matching
the Java oracle snapshot taken after `createItems`. Vault "May differ" cells are
the Imp reward pedestals (route) and consumables from `setupConsumables`, whose
`Collections.shuffle` is unseeded JDK RNG (published as `per_run_cells`, with
their own panel note); the leading Potion of Healing in
tiers 0–2 stays exact. Vault `sentry_scans` replay each `VaultSentry` scan
cycle (`Ballistica` + `ConeAOE` on the final map) as animated `CheckedCell`
flashes; the sentry's own field-of-view filter is not modeled. `laser_beams`
carry each firing `VaultLaser`'s cooldowns and `STOP_SOLID` path, drawn as
`DeathRay` beams with `TargetedCell` warnings; flame vents burn and warn from
their packed `VaultFlameTraps` cooldowns, all on the sentry turn clock.
Mining maps publish room-painted entities plus
`MiningLevel.createMobs` (Crystal Wisps / Gnoll Guards) and `createItems`
(food, Darkness torch), matching the oracle snapshot after `createItems`.

The Ambitious Imp quest panel reports each of the six take-out slots with its
seed-independent guarantees and a candidate distribution over a +-3 deck-index
window, seed draw first. The window is exact rather than sampled: a
category's draw order is fixed by the seed, so run history only shifts which
index the Imp lands on. Percentages are the share of that enumerated window, a
stability measure, not an observed player frequency.

Upgrades and enchants are stronger than the classes: they are pinned by the
seed, because every class in a category costs the same ambient RNG. The panel
reports each slot's concrete `+N`, its enchant by name, and the pool's total
Scroll of Upgrade worth. `upgrades_pinned` goes false only when an
`UnstableSpellbook` draw is reachable in slot 0, which shifts every later roll.

The Old Wandmaker quest panel reports two ordered options, wand1 then wand2,
each over a +-3 deck-index window around the seed-drawn wand1 index, seed
draw first. The class at each index is a pure function of the seed, so run history
only shifts which index the reward lands on. Percentages are the share of that
enumerated window, a stability measure, not an observed player frequency.
wand1's +N is pinned across the window. wand2's +N is the seed roll and stays
pinned only when every offset in the window rejects the same number of
duplicate draws.

The finder catalogue includes the weapon, missile, and armor classes those
quests can roll. Its upgrade filter reaches +5 for that equipment. Rings and
wands still stop at +4. The Deep search switch fills the seed-only Imp and Wandmaker entries with
the same class window so those quest rewards can be matched at all; it is off
by default.

Each report opens with a run header: the seed in the pixel font with a copy
button, and toggles for the five challenges that change generated floors
(`Challenge::generation_effect`). Toggling calls `analyze_seed_for_run`
(WASM export of the same name) with the trinket pick and challenge list;
declared challenges are never spawn conditions, and choices persist per seed
in localStorage. Forbidden Runes, Barren Land, Into Darkness (floor torch,
`RegularLevel.java:472-490`) and Badder Bosses are replayed and match
all-challenge oracle fixtures on FVX-YTN-SLC floors 2/4/8/12. Champion
Enemies is replayed exactly on floor 1 (oracle floor-1 Giant fixtures); later
floors mark entities uncertain when a possible Giant sits outside open space.

The Trinket Catalyst picker sits on the seed's selection floor. Picking an
offer calls `analyze_seed_with_trinket` (WASM export of the same name): the
trinket is held at +3 from `first_effective_depth`, variable samples are
published, and only Forbidden Runes stays a spawn condition unless challenges
are declared. No pick is the merged `analyze_seed` report. Per-option `catalyst_option_effects` marks which
trinkets the replay models: Mossy Clump, Trap Mechanism, Mimic Tooth, Rat
Skull, Exotic Crystals, Cracked Spyglass, and Parchment Scrap. Every other
trinket only changes play. Parchment Scrap changes only the isolated
curse/enchant roll of weapons, missiles, and armor, plus the Ghost and
Blacksmith keep test, so layouts stay identical. Oracle fixtures
`*-parchment-scrap-3-before-3.json` pin heap enchantments. Map heap items
publish their enchantment or glyph, which the inspect panel shows in the
title. Cracked Spyglass's hidden extra loot is placed only in
the picked-trinket replay (oracle fixtures `*-cracked-spyglass-3-before-2.json`);
the merged default report does not enumerate Spyglass routes. Map
`concealed_cells` (Ebony Mimics, Spyglass loot) fade with the hidden-area
pulse. The pick persists per seed in
localStorage. Mimic carried items are on the floor list and the map inspect
panel: the converted heap item, the generated prize, and the Mimic Tooth
extra when one is held. Ebony mimics (Mimic Tooth only) carry the prize,
the tooth extra, and one more default item, each uncursed and at least +1.
Crystal mimics keep the uncursed vault prize and lose it if they escape. Animated statues carry their enchanted weapon on both the floor list and the map inspect panel; armored statues also carry and drop their inscribed armor.
On floor 2 an ebony mimic's cell can depend on the unseeded entrance guide
page, and is then left off the map. Java oracle fixtures cover Mimic Tooth
+3 floors (`*-mimic-tooth-3-before-*.json`).

Every public object is a numeric id from `crates/spd-objects` (items,
enchantments, mobs, rooms, traps, plants, blobs). `bun run gen:ids` writes
`web/src/generated/game-ids.ts`; a Rust test fails when it drifts. Id lists are
append-only. The UI composes titles from ids, levels, curses, and enchantments;
a known enchantment or glyph always wraps the item name with the game's `%s`
template ("Blazing Javelin", "Plate Armor of Brimstone"), never a separate tag.
Items whose concrete result depends on gameplay carry `prediction: "variable"`
and keep their seed-drawn value.

The finder runs an item-only pass (`search_seed_floors`): no maps, branches,
probes, or report projection; it stops at the deepest constraint and, in
Match all mode, as soon as a closed depth window can no longer match. A parity
test pins it to the full seed-only analysis.

`spd_core::analyze_seed_exact` (CLI: `bun run analyze SEED [FLOORS] [--maps]
[--pretty] [--full]`, crate `spd-cli`) is a concise exact projection under one
declared default profile: no challenges, no trinkets, no items acquired outside
level generation, default inventory. It reads the engine's internal per-floor
items, so it has no `constrained`/`variable` entries; quest choose-one pools sit
under `quests[].rewards`, and maps are RLE terrain plus traps, plants, and
transitions. Tests pin that every public exact item appears in it. It stays a
CLI/test projection and is deliberately not exported through WASM: it drops
the `constrained`/`variable` distinction, so a web view would present
player-dependent outcomes as fixed (SEED-ANALYSIS, NO-CLAIM). The web report
already shows the default profile through Variable samples and "May differ"
labels.

Active-profile floor lists pair blank room and queued-food contracts with
concrete Variable samples (RingRoom, Sacrifice, CrystalVault, BlacksmithRoom,
labs, shops, queued food, …) and publish ordinary floor-heap, MassGrave
skeleton, Spyglass, and Into Darkness drops so the list matches the floor map
instead of hiding items or leaving "?" stubs. Remaining blank stubs are
branch-miss contracts (possible pool equipment, gold fallback, unused catalyst
slot, SecretGarden) with no sample on this profile; the UI hides those.

Next:

1. Performance (`bun run profile`): a 26-floor report is ~22.5 ms native,
   ~37 ms WASM; the default replay is ~55% and the ~7 trinket-route
   replays ~45%. `patch::generate` (~17%) is mostly the parity-bound
   `next_float` fill draws. Measured but not adopted: talc allocator (no
   WASM gain), `+simd128` (~7% WASM, drops pre-16.4 Safari), sharing the
   pre-trinket floor prefix across routes (~1–2 ms; needs trinket/RNG
   thread-local snapshots), and
   micro-changes that measured slower (compact free-space rects, hoisted
   maze RNG, reused `smooth` buffers).
