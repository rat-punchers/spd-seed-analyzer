import { itemClassName, type MobId, mobClassName } from '@/generated/game-ids'
import type { MapHeapItem } from '@/lib/spd-wasm'

/** Idle animation of a pinned v4.0.2 `*Sprite` class on its `TextureFilm`. */
export type MobSprite = {
  /** File stem under `/assets/sprites/`. */
  sheet: string
  width: number
  height: number
  /** `MovieClip.Animation` frame rate; 0 holds the first frame. */
  fps: number
  frames: readonly number[]
  /** `CharSprite.perspectiveRaise` in pixels. */
  raise: number
  /** Still frame while a passive mob hides in plain sight. */
  hidingFrame?: number
}

function mimic(offset: number, hidingFrame?: number): MobSprite {
  return {
    ...sprite('mimic', 16, 16, 5, MIMIC, offset),
    // MimicSprite drops one pixel of raise for its hiding border.
    raise: 5,
    hidingFrame: hidingFrame == null ? undefined : hidingFrame + offset,
  }
}

function sprite(
  sheet: string,
  width: number,
  height: number,
  fps: number,
  frames: readonly number[],
  offset = 0
): MobSprite {
  return {
    sheet,
    width,
    height,
    fps,
    frames: frames.map((frame) => frame + offset),
    raise: 6,
  }
}

function hold(frame: number, count: number) {
  return Array<number>(count).fill(frame)
}

const BLINK = [0, 0, 0, 1]
const SHIFT = [0, 0, 0, 1, 0, 0, 1, 1]
const LOOK = [0, 0, 0, 1, 0, 0, 0, 0, 1]
const STAND = [0, 0, 0, 0, 0, 1, 1]
const SPIN = [0, 0, 0, 0, 0, 1, 0, 1]
const CRAB = [0, 1, 0, 2]
const SLIME = [0, 1, 1, 0]
const ELEMENTAL = [0, 1, 2]
const SKELETON = [...hold(0, 13), 1, 2, 3]
const MIMIC = [3, 3, 3, 4, 4]
const PIRANHA = [0, 1, 2, 1]
const IMP = [
  0,
  1,
  2,
  3,
  0,
  1,
  2,
  3,
  0,
  0,
  0,
  ...hold(4, 10),
  0,
  1,
  2,
  3,
  0,
  1,
  2,
  3,
  0,
  1,
  3,
  0,
  0,
  0,
  ...hold(4, 8),
  0,
  0,
  0,
  ...hold(4, 8),
]

/**
 * Keyed by mob class. Variant sprites add their `texOffset()` / `c` constant.
 * Statues use the tier-0 frames because the weapon tier is mob state.
 */
const MOB_SPRITES: Record<string, MobSprite> = {
  Rat: sprite('rat', 16, 15, 2, BLINK),
  Albino: sprite('rat', 16, 15, 2, BLINK, 16),
  FetidRat: sprite('rat', 16, 15, 2, BLINK, 32),
  Snake: sprite('snake', 12, 11, 10, [
    ...hold(0, 15),
    ...hold(1, 10),
    2,
    3,
    2,
    1,
    1,
  ]),
  Gnoll: sprite('gnoll', 12, 15, 2, SHIFT),
  GnollExile: sprite('gnoll', 12, 15, 2, SHIFT, 21),
  GnollTrickster: sprite('gnoll', 12, 15, 2, SHIFT, 42),
  Swarm: sprite('swarm', 16, 16, 15, [0, 1, 2, 3, 4, 5]),
  Crab: sprite('crab', 16, 16, 5, CRAB),
  HermitCrab: sprite('crab', 16, 16, 5, CRAB, 16),
  GreatCrab: sprite('crab', 16, 16, 5, CRAB, 32),
  Slime: sprite('slime', 14, 12, 3, SLIME),
  CausticSlime: sprite('slime', 14, 12, 3, SLIME, 9),
  Skeleton: sprite('skeleton', 12, 15, 12, SKELETON),
  Thief: sprite('thief', 12, 13, 1, LOOK),
  Bandit: sprite('thief', 12, 13, 1, LOOK, 21),
  DM100: sprite('dm100', 16, 14, 1, [0, 1]),
  Guard: sprite('guard', 12, 16, 2, SHIFT),
  Necromancer: sprite('necromancer', 16, 16, 1, LOOK),
  SpectralNecromancer: sprite('necromancer', 16, 16, 1, LOOK, 16),
  Bat: sprite('bat', 15, 15, 8, [0, 1]),
  Brute: sprite('brute', 12, 16, 2, SHIFT),
  ArmoredBrute: sprite('brute', 12, 16, 2, SHIFT, 21),
  Shaman: sprite('shaman', 12, 15, 2, SHIFT),
  RedShaman: sprite('shaman', 12, 15, 2, SHIFT),
  BlueShaman: sprite('shaman', 12, 15, 2, SHIFT, 21),
  PurpleShaman: sprite('shaman', 12, 15, 2, SHIFT, 42),
  Spinner: sprite('spinner', 16, 16, 10, SPIN),
  DM200: sprite('dm200', 21, 18, 10, [0, 1]),
  DM201: sprite('dm200', 21, 18, 2, [0, 1], 12),
  Ghoul: sprite('ghoul', 12, 14, 2, BLINK),
  Elemental: sprite('elemental', 12, 14, 10, ELEMENTAL),
  FireElemental: sprite('elemental', 12, 14, 10, ELEMENTAL),
  FrostElemental: sprite('elemental', 12, 14, 10, ELEMENTAL, 28),
  ShockElemental: sprite('elemental', 12, 14, 10, ELEMENTAL, 42),
  ChaosElemental: sprite('elemental', 12, 14, 10, ELEMENTAL, 56),
  Warlock: sprite('warlock', 12, 15, 2, SHIFT),
  Monk: sprite('monk', 15, 14, 6, [1, 0, 1, 2]),
  Senior: sprite('monk', 15, 14, 6, [18, 17, 18, 19]),
  Golem: sprite('golem', 17, 19, 4, [0, 1]),
  Succubus: sprite('succubus', 12, 15, 8, [...hold(0, 14), 1, 2, 2, 2, 2, 1]),
  Eye: sprite('eye', 16, 18, 8, [0, 1, 2]),
  Scorpio: sprite('scorpio', 17, 17, 12, [...hold(0, 8), 1, 2, 1, 2, 1, 2]),
  Acidic: sprite('scorpio', 17, 17, 12, [...hold(0, 8), 1, 2, 1, 2, 1, 2], 15),
  RipperDemon: sprite('ripper', 15, 14, 4, [1, 0, 1, 2]),
  DemonSpawner: sprite('spawner', 16, 16, 8, [...Array(16).keys()]),
  Statue: sprite('statue', 12, 15, 2, STAND),
  // `ArmoredStatue.sprite()` defaults to tier 3 when armor is absent.
  ArmoredStatue: sprite('statue', 12, 15, 2, STAND, 43),
  Mimic: mimic(0),
  GoldenMimic: mimic(16),
  CrystalMimic: mimic(32),
  // `MimicSprite.advancedHiding`.
  EbonyMimic: mimic(48, 0),
  Piranha: sprite('piranha', 12, 16, 8, PIRANHA),
  PhantomPiranha: sprite('piranha', 12, 16, 8, PIRANHA, 21),
  RotHeart: sprite('rot_heart', 16, 16, 0, [0]),
  RotLasher: sprite('rot_lasher', 12, 16, 0, [0]),
  Sentry: sprite('sentry', 8, 15, 0, [0]),
  Shopkeeper: sprite('shopkeeper', 14, 14, 10, [1, 1, 1, 1, 1, 0, 0, 0, 0]),
  ImpShopkeeper: sprite('imp', 12, 14, 10, IMP),
  Imp: sprite('imp', 12, 14, 10, IMP),
  Ghost: sprite('ghost', 14, 15, 5, [0, 1]),
  Wandmaker: sprite('wandmaker', 12, 14, 10, [
    ...hold(0, 14),
    1,
    2,
    ...hold(3, 6),
    2,
    1,
  ]),
  Blacksmith: sprite('blacksmith', 13, 16, 15, [...hold(0, 7), 1, 2, 2, 2, 3]),
  RatKing: sprite('ratking', 16, 16, 2, BLINK),
  Goo: sprite('goo', 20, 14, 10, [2, 1, 0, 0, 1]),
  // PylonSprite drops one pixel of raise and starts inactive.
  Pylon: { ...sprite('pylon', 10, 20, 0, [0]), raise: 5 },
  Sheep: sprite('sheep', 16, 15, 8, [...hold(0, 12), 1, 2, 3, 0]),
  Bee: sprite('bee', 16, 16, 12, [0, 1, 1, 0, 2, 2]),
  Wraith: sprite('wraith', 14, 15, 5, [0, 1]),
  TormentedSpirit: sprite('wraith', 14, 15, 5, [0, 1], 9),
  CrystalWisp: sprite('crystal_wisp', 12, 14, 0, [0]),
  CrystalGuardian: sprite('crystal_guardian', 12, 15, 2, STAND),
  CrystalSpire: sprite('crystal_spire', 29, 40, 0, [0]),
  GnollGuard: sprite('gnoll_guard', 12, 16, 2, SHIFT),
  GnollSapper: sprite('gnoll_sapper', 12, 15, 2, SHIFT),
  GnollGeomancer: sprite('gnoll_geomancer', 12, 16, 2, SHIFT),
  FungalSentry: sprite('fungal_sentry', 18, 18, 0, [0]),
  FungalSpinner: sprite('fungal_spinner', 16, 16, 10, SPIN),
  FungalCore: sprite('fungal_core', 27, 27, 0, [0]),
  VaultSkeleton: sprite('skeleton', 12, 16, 12, SKELETON, 21),
  VaultDM100: sprite('dm100', 16, 14, 1, [0, 1], 16),
  VaultShaman: sprite('shaman', 12, 15, 2, SHIFT, 63),
  VaultDM200: sprite('dm200', 21, 18, 10, [0, 1], 24),
  VaultGhoul: sprite('ghoul', 12, 14, 2, BLINK),
  VaultGolem: sprite('golem', 17, 19, 4, [0, 1]),
  VaultSentry: sprite('sentry', 8, 15, 0, [1]),
  VaultLaser: sprite('sentry', 8, 15, 0, [2]),
  VaultMirror: sprite('vault_mirror', 16, 23, 15, [
    ...hold(0, 48),
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
  ]),
  VaultTokenDoor: sprite('vault_tokens_door', 16, 16, 0, [0]),
}

const ARMORED_STATUE_OFFSETS: Record<string, number> = {
  ClothArmor: 21,
  LeatherArmor: 32,
  MailArmor: 43,
  ScaleArmor: 54,
  PlateArmor: 65,
}

export function mobSprite(
  mob: MobId,
  items?: readonly MapHeapItem[]
): MobSprite | undefined {
  const cls = mobClassName(mob)
  if (cls === 'ArmoredStatue' && items?.length) {
    for (const entry of items) {
      const offset = ARMORED_STATUE_OFFSETS[itemClassName(entry.item)]
      if (offset != null) {
        return sprite('statue', 12, 15, 2, STAND, offset)
      }
    }
  }
  return MOB_SPRITES[cls]
}

export function mobSheetUrl(sheet: string): string {
  return `/assets/sprites/${sheet}.png`
}

/**
 * Idle frame shown `seconds` into the loop. `phase` desynchronises mobs that
 * share a sprite, as their in-game clips start at different times.
 */
export function mobFrameAt(spriteDef: MobSprite, seconds: number, phase = 0) {
  const { fps, frames } = spriteDef
  if (fps <= 0 || frames.length < 2) return frames[0] ?? 0
  const step = Math.floor(seconds * fps) + phase
  return frames[step % frames.length] ?? frames[0] ?? 0
}

/** Source rectangle for a film frame on the sprite's loaded sheet. */
export function mobSourceRect(
  spriteDef: MobSprite,
  sheet: HTMLImageElement,
  frame = spriteDef.frames[0] ?? 0
) {
  const cols = Math.max(1, Math.floor(sheet.naturalWidth / spriteDef.width))
  return {
    x: (frame % cols) * spriteDef.width,
    y: Math.floor(frame / cols) * spriteDef.height,
  }
}
