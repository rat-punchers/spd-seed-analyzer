# Generator decks — verified facts

Verified against SPD **v4.0.2 @ `57a4e06a4`**. Java line numbers are from that
commit. **Re-verify only when the pinned version changes** — the deck
mechanism, the call sites, and the challenge behaviour below are all
version-sensitive.

Method: `Generator.java` was read at this commit. Sections below record that
source's call shape. VaultLevel `usingDefaults` is pinned by
`tools/java-oracle/fixtures/vault/aaa-aaa-aaa-floor-17.json`.

## 1. Category sub-decks make item identity index-keyed, not stream-keyed

`Generator.random(Category)` (`items/Generator.java:709+`) draws the class from
a **per-category seeded sub-RNG**:

```java
if (cat.defaultProbs != null && cat.seed != null){
    Random.pushGenerator(cat.seed);
    for (int i = 0; i < cat.dropped; i++) Random.Long();
}
int i = Random.chances(cat.probs);   // then popGenerator(); cat.dropped++
```

`cat.seed` is rolled during `Generator.fullReset()` from the run seed. So the
class of the *k*-th item a run draws from a category is a **pure function of
(dungeon seed, k)**, independent of where in a floor's RNG stream the draw
lands. Ported at `generator/state.rs::random_deck_item`.

Confirmed empirically: drawing 10 rings from a fresh generator under two
different ambient RNG streams yields identical sequences, and that sequence
matches — in order — the rings a full 19-floor analysis of the same seed
generates.

`randomUsingDefaults` bypasses both `probs` and `dropped` entirely.

## 2. Cross-floor RNG coupling is deck counters only for a fixed profile

Level generation is wrapped in `Random.pushGenerator(Dungeon.seedCurDepth())`,
so a stream difference on floor *N* cannot directly reach floor *N+1*. For a
fixed challenge/trinket/meta profile, the cross-floor RNG state relevant to
item identities is the deck counters. Parity for any deck-drawn identity
therefore reduces to: **the same draw sites fire the same number of times in
the same order**. Player choices can change which draw sites fire on later
floors; the Sad Ghost examples are catalogued in
`specs/analysis/quest-sad-ghost.md`.

## 3. Ring-deck draw sites (complete)

Direct ring deck draws (`Generator.random`) are levelgen:

| Site | Java |
|------|------|
| `createItems` general-deck picks | `RegularLevel.java:388` |
| Ordinary + golden mimic prizes | `Mimic.java:349` (`useDecks=true`) |
| Shop rare, 1/10 | `ShopRoom.java:339` |
| Pit room main loot | `PitRoom.java:72` |
| Crystal choice hidden prize | `CrystalChoiceRoom.java:124` |
| Crystal vault prize cycle | `CrystalVaultRoom.java:104` |
| Grassy grave / mass grave / secret summoning | `Generator.random()` |
| Ambitious Imp spawn | `Imp.java:308-353` (`Quest.spawn`) |

The `random(ARTIFACT)` fallback is not a ring-deck site; see §4. Imp spawn
is: `randomArtifact()` (`Imp.java:319`); on miss `random(RING)` (`:324`); a
second `random(RING)` with class uniqueness (`:330-336`); `Random.Int(2)`
then `random(WEP_T5)+random(MIS_T4)` or `random(MIS_T5)+random(WEP_T4)`
(`:338-344`); `new PlateArmor().inscribe()` (no ARMOR deck, `:345`);
`random(WAND)` (`:346`). All `cursed = false` (`:351-353`). Call site:
`CityLevel.initRooms` (`CityLevel.java:191-194`).

Note `Generator.random()` (no-arg) is itself a category deck, so its ring
count over a deck cycle is stable even if the in-cycle order shifts.

## 4. Direct runtime ring requests bypass the ring deck

> Scope note: calls that directly request a runtime ring use defaults.
> Artifact exhaustion is also defaults: `Generator.random(ARTIFACT)` falls
> through to `randomUsingDefaults(RING)` (`Generator.java:706-710`) and does
> **not** move `RING.dropped`. Runtime `randomUsingDefaults(ARTIFACT)` still
> uses the artifact deck (`Generator.java:744-752`). See §11.

All use `randomUsingDefaults`: mob loot including Thief's
`oneOf(RING, ARTIFACT)` (`Thief.java:52`, category branch
`Mob.java:1141-1143`), Ring of Wealth
(`RingOfWealth.java:288`), Scroll of Transmutation
(`ScrollOfTransmutation.java:276`), Cursed Wand (`CursedWand.java:1081`,
`:1174`), and runtime-spawned mimics (`useDecks=false` at
`CursedWand.java:1081`, `DistortionTrap.java:120`).

Branch side-levels add no ring-deck site. `MiningLevel` only calls
`randomUsingDefaults(FOOD)`. `VaultLevel.createEquipment`
(`VaultLevel.java:381`) draws WEP/MIS/WAND/RING via `randomUsingDefaults`
(`:241-376`); `createConsumabe` (`:485`) does not touch those decks;
`build()` adds three default FOOD (`:141-143`). ARTIFACT is unused.
`RING.dropped` stays flat: AAA-AAA-AAA vault fixture
`tools/java-oracle/fixtures/vault/aaa-aaa-aaa-floor-17.json` records 0→0
around `VaultLevel.create()`.

Method: reading `Generator.java:706-710` on the pinned v4 clone, plus a
`spd-core` unit test that exhausts `random_artifact` and snapshots
`RING.dropped` around the fallback `random(ARTIFACT)`. Vault counters come
from that fixture.

## 5. Challenge item-block rerolls cannot shift a deck index

`Challenges.isItemBlocked` (`Challenges.java:71`) returns true **only** for a
`Dewdrop` under `NO_HERBALISM`. No ring/wand/artifact/weapon/armor generator
can produce a Dewdrop, so the reroll loops in `Mimic.generatePrize`, `PitRoom`,
and `CrystalVaultRoom` never iterate from challenges.

Of the nine challenges, only `NO_SCROLLS` touches generation at all
(`Level.java:234`, skips a Scroll of Upgrade `addItemToSpawn` — no RNG, no
deck at that call site). `DARKNESS` and `STRONGER_BOSSES` affect view distance
and boss setup. `NO_SCROLLS` is nevertheless not reward-neutral: omitting the
queued Scroll can empty `itemsToSpawn`, after which a room painter can take a
generated fallback that consumes the main floor stream before an NPC callback.
Sad Ghost has a reachable `RitualRoom` counterexample; see
`specs/analysis/quest-sad-ghost.md`.

## 6. Trinket offers are seed-determined; the held trinket is not

`TRINKET (0, 0, Trinket.class)` (`Generator.java:222`) has zero weight in both
general decks, so no trinket comes from floor loot. The only levelgen source is
the Trinket Catalyst: `Dungeon.trinketCataNeeded()` (`Dungeon.java:584`) grants
exactly one per run on floors 1–4, guaranteed by depth 4.

The catalyst window rolls `NUM_TRINKETS = 4` options through
`Generator.random(Category.TRINKET)` (`TrinketCatalyst.java:178`). Being the
deck's only consumer, **its four offers are always TRINKET draws 0–3** — fully
seed-determined and computable before any play. Scroll of Transmutation on the
taken trinket (`ScrollOfTransmutation.java:325`) is the only other route, and
lands on draw 4.

Measured over 400 seeds: Mimic Tooth is among the four offers in **101 (25.3%,
= 4/17)**, its first-appearance index uniform across the 17-class deck. So
~75% of seeds rule out taking Mimic Tooth **directly from the Catalyst**. They
do not rule it out for the whole run: a Scroll of Transmutation advances to
draw 4 (and later transmutations advance farther), so that route must be checked
separately.

The player chooses one offer, may transmute it, and chooses when/how far to
upgrade it. Those choices are not encoded by the seed. In particular Mossy
Clump, Mimic Tooth, and Rat Skull can change generation work before an NPC
reward callback; see the Sad Ghost audit in
`specs/analysis/quest-sad-ghost.md`.

Not having the trinket costs no RNG: `RegularLevel.java:406` evaluates
`Random.Float()` before applying `MimicTooth.mimicChanceMultiplier()`, so the
tooth-free stream is the baseline stream.

### Spyglass opacity, Censer gases, Exotic Crystals

Cracked Spyglass extra hidden items are generation (§11). Hidden-heap sprite
alpha (`sprites/ItemSprite.java:236`, `heap.hidden ? 0.1f`) is renderer, not
generation; public maps omit heaps.

Chaotic Censer `GAS_CAT_CHANCES` and the common/uncommon/rare gas tables
(`ChaoticCenser.java:300-328`; Stench common, Regrowth uncommon) are chosen in
`produceGas` (`:151-232`) from `CenserGasTracker.act` — runtime spew, not
`Generator`, `Level.create`, or quest spawn. Trinket `upgradeEnergyCost`
comments are not generation.

Held Exotic Crystals convert generated potions and scrolls at
`consumableExoticChance` `0.2 + 0.2*level` (`ExoticCrystals.java:52-58`).
Levelgen sites: `Generator.random` (`Generator.java:730-737`),
`randomUsingDefaults` (`:758-766`), Secret Laboratory
(`SecretLaboratoryRoom.java:101`), Secret Library (`SecretLibraryRoom.java:100`),
and Crystal Path (`CrystalPathRoom.java:173-176`). Ring of Wealth
(`RingOfWealth.java:264-266`) and mob loot (`Mob.java:1111-1115`) stay
runtime. Method: pinned-source call-site audit.

## 7. Ambitious Imp spawn

`Imp.Quest.spawn` fills six `rewardOptions` during `CityLevel.initRooms` on
the first success of `Random.Int(20 - depth) == 0` for depths 17–19
(`Imp.java:306-353`). Options 3–4 are the `Random.Int(2)` weapon/missile pair
(`Imp.java:338-344`). Every option is then `cursed = false`
(`Imp.java:351-353`). The concrete index for a fixed profile is pinned by
replaying that profile.

## 8. Consequence for the Imp reward pool

Spawn always writes a take-one pool of six (`Imp.java:318-353`). The seed
determines that pool under a fixed profile; it does not determine which of
the six the player keeps. Shop access (`earnedShop` / score) is not a seed
fact.

Option 1 is an ARTIFACT deck draw (`randomArtifact`). If that deck is
exhausted, the same slot is a RING deck draw (`Generator.random(RING)`,
`Imp.java:323-327`) — labelled, and not the `random(ARTIFACT)` defaults
fallback of §4. Option 2 is always a RING
deck draw; the uniqueness loop can consume extra RING draws only when
option 1 was already a ring. Options 3–4 are one WEP/MIS pair from the
coin flip in §7. Option 5 is a constructed `PlateArmor` (no ARMOR deck).
Option 6 is a WAND deck draw. Every option is then `cursed = false`.

For a fixed profile, each deck-drawn class is index-keyed (§1). No `Ring`
subclass overrides `Ring.random()` (`Ring.java:259`), so a RING slot's
class is decoupled from its later floor-stream level roll. Across playable
routes, spawn depth, deck indices, and the WEP/MIS coin flip are not
seed-only: Mimic Tooth changes levelgen mimic prizes; Rat Skull and Mimic
Tooth can turn the Crystal Vault's second chest into a Crystal Mimic, which
keeps the vault prize and draws nothing (`CrystalMimic.java:177-183`; Java
oracle `aaa-aaa-bnk-final-heaps-floor-3-mimic-tooth-3-before-2.json` fails
when a mimic prize is replayed there); earlier map/challenge paths can change
levelgen draw sites; before `Imp.Quest.spawn`, branch 0 cases 7–13 draw
both feeling floats and a Mossy hit skips only `TrapMechanism.getNextFeeling()`
without touching the main stream (`Level.java:287-290`,
`MossyClump.java:77-88`). Runtime artifact requests still advance
`ARTIFACT.dropped` (§11), which can flip option 1 from artifact to ring.
Quest conditions are in `specs/analysis/quest-ambitious-imp.md`.

## 9. Old Wandmaker reward (floors 7–9)

The call shape below is for a fresh profile with no map-affecting trinket, no
challenge, and no external artifact. It is not a universal pair for every
playable route. On branch 0, cases 7–13 always draw both `mossyChance` and
`trapMechChance` before either override (`Level.java:287-290`). A Mossy hit
skips only `TrapMechanism.getNextFeeling()`. `MossyClump.getNextFeeling`
shuffles under `pushGenerator(Dungeon.seed+1)` (`MossyClump.java:77-88`) and
does not touch the main stream. GRASS/WATER do not by themselves change the
pre-spawn room-count formulas. Trap Mechanism can alter painting after
selection. Mimic Tooth, Rat Skull, artifact
constructor history, and challenge-dependent prize queues can also add or
remove painter and levelgen item draws before the NPC callback. The full state
audit is in `specs/analysis/quest-wandmaker.md`.

`Wandmaker.Quest.spawnWandmaker` (`Wandmaker.java:303`) runs at the **start** of
`PrisonLevel.createMobs`, before `super.createMobs()`
(`PrisonLevel.java:89-92`), i.e. after paint and after an NPC placement loop
that burns two `Random.IntRange` calls per attempt against painted terrain.

Reward code (`Wandmaker.java:334-346`):

```java
wand1 = Generator.random(WAND); wand1.cursed = false; wand1.upgrade();
wand2 = Generator.random(WAND);
while (wand2.getClass() == wand1.getClass()) { toUndo.add(wand2); wand2 = Generator.random(WAND); }
for (Item i : toUndo) Generator.undoDrop(i);
wand2.cursed = false; wand2.upgrade();
```

Verified properties:

- **Level** is `1 + n` from `Wand.random()` (`Wand.java:546-566`): +1 66.7%,
  +2 26.7%, +3 6.7%. **Never cursed** — `cursed = false` precedes `upgrade()`,
  and `Wand.upgrade()`'s `Random.Int(3)` curse-clear (`Wand.java:358-364`) is
  consumed but cannot re-curse.
- **Floor-stream cost is class-independent.** No `Wand` subclass overrides
  `random()` or `upgrade()`, and no wand instance initializer or constructor
  touches `Random` (checked all 13).
- **`undoDrop` is a no-op for concrete classes** (`Generator.java:662-673`:
  `cls.isAssignableFrom(cat.superClass)` is false for e.g. `WandOfFireblast`),
  so rejected duplicate draws permanently advance the deck. `undo_drop` at
  `generator/state.rs:418` correctly does nothing.
- **`wand2`'s level is coupled to identity** — unlike the Imp ring. The
  duplicate-rejection loop burns an extra `Wand.random()` off the floor stream,
  so resolving `wand2`'s level requires knowing the deck index. `wand1`'s level
  is decoupled.
- **No runtime path directly moves the wand deck.** Every `Category.WAND`
  reference outside levelgen uses `randomUsingDefaults`: Shaman loot via
  `Mob.createLoot` (`Mob.java:1141-1143`), Scroll of Transmutation
  (`ScrollOfTransmutation.java:211`, `:339`), Cursed Wand
  (`CursedWand.java:363`, `:371`). Consistent with §4. Player state can still
  change which levelgen WAND draw sites run before the reward, so the concrete
  deck index and both floor-stream levels require a fixed profile.

The concrete index for a fixed profile is pinned by replaying that profile.

## 10. Sad Ghost reward (floors 2–4)

The call shape below is for the default profile with no map-affecting trinket,
no challenge, and no external artifact. It does not make the concrete pair a
pure function of the seed across every playable route. Mossy Clump can change
the paint stream before `Ghost.Quest.spawn`; Mimic Tooth and Rat Skull can
change pre-Ghost mimic/statue generation; and external artifact history can
change a later artifact constructor tail. The complete state audit is in
`specs/analysis/quest-sad-ghost.md`.

`Ghost.Quest.spawn` (`Ghost.java:303-362`) runs at the **start** of
`SewerLevel.createMobs`, before `super.createMobs()` (`SewerLevel.java:140-143`)
— same placement-loop-then-reward shape as the Wandmaker.

Verified properties:

- **The armor uses no deck at all.** `Random.chances({0,0,10,6,3,1})` selects a
  tier and the class is constructed directly (`Ghost.java:322-328`): 2 =
  `LeatherArmor`, 3 = `MailArmor`, 4 = `ScaleArmor`, 5 = `PlateArmor`. Tier and
  class are the same fact once the tier roll is fixed.
- **Only the weapon is deck-drawn** — `Generator.random(wepTiers[tier-1])`
  (`Ghost.java:331`), i.e. the per-tier `WEP_T2…WEP_T5` decks, each with its own
  seed and `dropped` counter.
- **The weapon draw's floor-stream cost is class-independent**:
  `randomize_weapon` spends `Int(4)` [+ `Int(5)`] + one `Long`, with the
  curse/enchant roll inside a pushed generator. So the deck index cannot move
  anything that follows.
- **The shared upgrade level** is one `Random.Float()` → +0 50%, +1 30%,
  +2 15%, +3 5%, applied to **both** items (`Ghost.java:339-351`). Neither
  `upgrade()` spends RNG on this path: `Weapon.upgrade(false)`
  (`Weapon.java:375-389`) and `Armor.upgrade(false)` (`Armor.java:454-468`) only
  roll when an enchantment/glyph is already present, and the Ghost clears both
  first.
- **Enchant and glyph are always generated**, deliberately, so the roll count is
  constant (`Ghost.java:353-356`). `Enchantment.random()` (`Weapon.java:606-615`)
  and `Glyph.random()` (`Armor.java:863-872`) each spend exactly
  `Random.chances` + `Random.element` — no deck, no player state.
- **Within that fixed profile, Parchment Scrap is the direct reward-local
  player-dependent bit**:
  `enchantRoll > 0.2f * ParchmentScrap.enchantChanceMultiplier()` clears both
  (`Ghost.java:358-362`). Multipliers are 1 / 2 / 4 / 7 / 10 for no scrap / +0 /
  +1 / +2 / +3 (`ParchmentScrap.java:52-65`), so the thresholds are 0.2 / 0.4 /
  0.8 / 1.4 / 2.0. **At Parchment Scrap +2 or better the reward is always
  enchanted.** The roll count does not change, so this never desyncs the stream.

Net for the fixed profile: everything except the weapon's *class* is
independent of the WEP sub-deck. This is not a seed-only claim across different
held-trinket, challenge, or artifact-history profiles; those can move the main
floor stream before the Ghost rolls tiers, level, and effects. The concrete
index for a fixed profile is pinned by replaying that profile.

## 11. Fixed-profile ordinary floor loot: ARTIFACT is the runtime deck leak

`RegularLevel.createItems` drops `3 + chances{6,3,1}` (`+2` if `LARGE`) items
from the no-arg `Generator.random()` (`RegularLevel.java:380-388`). Three
mechanisms would have to leak for run history to move them; two are closed.

- **Floor stream** — closed by the per-depth `pushGenerator` of §2.
- **General category deck** (`categoryProbs`, picks *which* category) — only
  no-arg `Generator.random()` decrements it (`Generator.java:676-683`), and its
  four call sites are all levelgen: `RegularLevel.java:388`,
  `GrassyGraveRoom.java:67`, `MassGraveRoom.java:108`,
  `SecretSummoningRoom.java:53`. `randomUsingDefaults()` reads `defaultCatProbs`
  and never decrements. **Closed.**
- **Per-category sub-decks** (pick the class) — closed for RING (§4), WAND (§9),
  and the WEP/MIS tiers: every runtime weapon site passes `useDefaults=true` or
  `useDecks=false` (`Statue.random(false)` from `DistortionTrap.java:125`,
  `WndBlacksmith.java:478 generateRewards(false)`, `TormentedSpirit.cleanse()`
  calling `Generator.randomWeapon(true)` at `TormentedSpirit.java:63-68` after
  an unseeded `Wraith.spawnAt` roll at `Wraith.java:144-150` from
  `Heap.java:86-98` and `Dungeon.java:254`). Armor uses no deck at all
  (`randomArmor` picks a tier by `chances` and indexes `cat.classes` directly).
  SEED is deliberately deck-free at levelgen — `Generator.java:684-688`
  special-cases it to `randomUsingDefaults` precisely because grass, not
  levelgen, is its dominant source.

  Troll Blacksmith's normal spawn-time `generateRewards(true)` also avoids the
  decks: its misleading `useDecks` parameter is passed directly into
  `randomWeapon` / `randomMissile` as `useDefaults=true`
  (`Blacksmith.java:364-384`, `Generator.java:808-818`, `:841-851`). The
  defensive `WndBlacksmith.generateRewards(false)` fallback is deck-backed,
  but the stored pool is already present in an ordinary fresh quest and one
  Smith purchase cannot reach that fallback. Full lifecycle details are in
  `specs/analysis/quest-blacksmith.md`.

Levelgen `StatueRoom` is not that runtime path. `paint` calls `Statue.random()`
(`StatueRoom.java:70`), which is `random(true)` (`Statue.java:202-204`).
`createWeapon` draws the weapon deck on the child (`Statue.java:63-71`,
`:206-219`). That is level generation, not a runtime leak. `DistortionTrap`
remains `random(false)` (`DistortionTrap.java:125`). Imp rewards roll in
`initRooms` before paint (`CityLevel.java:192-194`,
`RegularLevel.java:109-120`), so a statue on the Imp floor does not move that
floor's Imp pool.

**Not closed: ARTIFACT.** `randomUsingDefaults(cat)` routes artifacts straight
back into the deck (`Generator.java:744-752`, comment: *"except for artifacts,
which must always use a deck"*), so these runtime sources all advance
`ARTIFACT.dropped`:

| Runtime source | Java |
|----------------|------|
| Ring of Wealth equipment drop | `RingOfWealth.java:291` |
| Thief steal-loot | `Thief.java:52` → `Mob.java:1141-1143` |
| Scroll of Transmutation | `ScrollOfTransmutation.java:298` |
| Cursed Wand | `CursedWand.java:1174` |
| Gnoll Exile / Ebony Mimic / tooth-mimic extras | `randomUsingDefaults()` no-arg landing on ARTIFACT |

Note `RingOfWealth.java:288` (the RING case §4 cites) is a different branch from
`:291`.

Cracked Spyglass is an additional **automatic level-generation** route, not a
runtime drop. `RegularLevel.createItems` adds its 0–2 hidden items through
no-arg `Generator.randomUsingDefaults()` (`RegularLevel.java:679-689`, with the
level-dependent chance at `CrackedSpyglass.java:52-61`). The pushed floor RNG
isolates ambient random calls, but it does not roll back persistent Generator
state. When the default category roll lands on ARTIFACT, holding/upgrading the
Spyglass advances the same artifact deck before later floors. Method: pinned-
source call-site audit from `createItems` through
`Generator.randomUsingDefaults()` (`Generator.java:694-768`); no probability
measurement was used for this reachability fact.

The drift is not cosmetic. `randomArtifact` calls
`Reflection.newInstance(cls).random()` **after** `popGenerator`
(`Generator.java:856-878`), and `UnstableSpellbook`'s constructor burns a
variable number of `Random.chances` rolls in `setupScrolls`
(`UnstableSpellbook.java:91-105`; chances loop `:96-103`, transmutation
removed at `:104`; constructor `:85-89`). A shifted artifact index therefore
changes floor-stream consumption and desyncs every later draw on that
floor, which in turn shifts the deck counters every later floor reads. Deck exhaustion falls
through to `randomUsingDefaults(RING)` (`Generator.java:706-710`) and does
**not** move `RING.dropped`. Runtime `randomUsingDefaults(ARTIFACT)` still uses
the artifact deck (`Generator.java:744-752`). Only the ARTIFACT→RING
fallthrough changed.

**Consequence for a fixed challenge/trinket/meta profile:** everything generated
before the run's *first* levelgen ARTIFACT deck draw is free of artifact-history
drift. Everything after it is seed-determined under one additional condition —
*no artifact acquired outside level generation earlier in the run*.
`random_artifact` (`generator/state.rs:327`) is the single Rust site that moves
the counter, so the artifact gate is one monotone flag. Other player-state
hooks can still change which generation path reaches that draw; the quest
audits track those separately. The concrete index for a fixed profile is
pinned by replaying that profile.

### Map entity certainty (stream taint)

The public map marks an entity "May differ" exactly when it is recorded on a
level stream that may have desynced. Rules, each forced by a pinned-source fact:

- Every level after the first is generated after play, so any levelgen
  ARTIFACT draw at depth > 1 (or on a branch) taints the rest of that level
  stream (UnstableSpellbook cost / exhaustion above).
- A tainted level may make a different number of draws from every deck, so
  every later level treats all deck positions as unknown. Then any
  `Generator.random()` taints (category costs differ), and so does any
  category draw: callers re-roll on identity, e.g. `CrystalPathRoom`
  duplicate rejection (`CrystalPathRoom.java:255-280`).
- Quest spawn state is cross-floor. The Ghost rolls in `createMobs`
  (`SewerLevel.java:139-142`, `Ghost.java:304-312`), after paint, so a
  tainted floor can flip it, and every later sewer attempt then consumes
  differently. Rust records a per-quest `history_dependent` flag when a roll
  runs on a tainted stream and taints later attempts of that quest. Wandmaker,
  Blacksmith and Imp roll in `initRooms` (`PrisonLevel.java:85`,
  `CavesLevel.java:91`, `CityLevel.java:193`), before any levelgen draw on
  their floor, and get the same flag.
- Player-state cells stay uncertain independently: shop stock (hourglass
  sandbags and `ChooseBag`), the depth > 1 `createItems` tail after
  rose/rations, possible Champion Giants, and the Forbidden Runes upgrade
  queue.

Method: `level/history_tests.rs` replays seeds with 1–25 runtime artifact
draws injected before depths 2–17 and asserts every exact heap and mob is
unchanged. Over 40 seeds, sewer floors 2–4 keep ~66% of entities exact; from
the first shop floor on almost every entity is "May differ", because some
earlier levelgen artifact draw is nearly always reachable.
