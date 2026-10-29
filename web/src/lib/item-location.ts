/**
 * Player-facing "where to find" text for an item `source`.
 * Door and container wording follows the pinned game's room painters.
 */

const ROOMS: Record<string, string> = {
  ShopRoom: 'Shop',
  ImpShopRoom: "Imp's shop",
  ArmoryRoom: 'Armory, behind a locked door',
  CryptRoom: 'Crypt, behind a locked door',
  CrystalChoiceRoom: 'Crystal choice room, behind a locked door',
  CrystalPathRoom: 'Crystal path room, behind crystal doors',
  CrystalVaultRoom: 'Crystal vault, behind a locked door',
  GardenRoom: 'Garden, behind a locked door',
  LaboratoryRoom: 'Alchemy laboratory, behind a locked door',
  LibraryRoom: 'Library, behind a locked door',
  MagicWellRoom: 'Magic well room, behind a locked door',
  RunestoneRoom: 'Runestone room, behind a locked door',
  StatueRoom:
    'Statue room, behind a locked door, carried by an animated statue',
  TreasuryRoom: 'Treasury, behind a locked door',
  StorageRoom: 'Storage room, behind a barricade',
  PitRoom: 'Pit room, behind a crystal door',
  PoolRoom: 'Piranha pool room, across the water',
  MagicalFireRoom: 'Room sealed by magical fire',
  SacrificeRoom: 'Sacrificial altar room',
  SentryRoom: 'Sentry room, past the watching sentry',
  ToxicGasRoom: 'Toxic gas room',
  TrapsRoom: 'Trap room, past the traps',
  SuspiciousChestRoom: 'Room with a suspicious chest',
  BlacksmithRoom: "Blacksmith's forge",
  MassGraveRoom: 'Mass grave',
  GrassyGraveRoom: 'Grassy graveyard',
  StudyRoom: 'Study, on the center pedestal',
  RitualRoom: 'Ritual room, on the center altar',
  RingRoom: 'Ring-shaped room, in the center',
  SecretArtilleryRoom: 'Secret artillery room',
  SecretChestChasmRoom: 'Secret chasm room, across the chasm',
  SecretGardenRoom: 'Secret garden',
  SecretHoardRoom: 'Secret gold hoard',
  SecretHoneypotRoom: 'Secret honeypot room',
  SecretLaboratoryRoom: 'Secret alchemy laboratory',
  SecretLarderRoom: 'Secret larder',
  SecretLibraryRoom: 'Secret library',
  SecretMazeRoom: 'Secret maze, at the far end',
  SecretRunestoneRoom: 'Secret runestone room',
  SecretSummoningRoom: 'Secret summoning room, in the trap field',
  SecretWellRoom: 'Secret well room',
}

/** Heap types and room roles, phrased as a trailing clause. */
const SPOTS: Record<string, string> = {
  chest: 'in a chest',
  locked_chest: 'in a locked chest',
  crystal_chest: 'in a crystal chest',
  ebony_chest: 'in an ebony chest',
  skeleton: 'in skeletal remains',
  remains: 'in remains',
  tomb: 'in a tomb',
  for_sale: 'for sale',
  mimic: 'carried by a mimic',
  golden_mimic: 'carried by a golden mimic',
  crystal_mimic: 'carried by a crystal mimic',
  ebony_mimic: 'carried by an ebony mimic',
  statue: 'carried by an animated statue',
  sacrificial: 'on the sacrificial altar',
  mimic_reward: 'dropped when the mimic is killed',
  hidden_reward: 'in the hidden chamber',
  prize: 'in a tomb',
  gold_tombs: 'in the other tombs',
  cracked_spyglass: 'hidden on the floor (Cracked Spyglass)',
  into_darkness: 'extra torch from Into Darkness',
}

/** Tags that say nothing about where to look. */
const SILENT = new Set([
  'forced',
  'heap',
  'hidden',
  'plant',
  'well',
  'equipment',
  'gold',
  'bomb',
  'guaranteed floor spawn',
])

/**
 * Where to look on the floor, or null when the source adds nothing useful.
 * Quest rewards return null because their quest card already says so.
 */
export function formatItemLocation(
  source: string | null | undefined
): string | null {
  if (!source || source.endsWith('.Quest')) return null
  const parts = source.split(':').filter((part) => !SILENT.has(part))
  const room = parts.find((part) => part in ROOMS)
  const spots = parts
    .filter((part) => part !== room)
    .map((part) => SPOTS[part])
    .filter((spot): spot is string => spot != null)
    .filter((spot, index, all) => spot !== all[index - 1])
  if (room) return [ROOMS[room], ...spots].join(', ')
  if (spots.length === 0) return null
  const text = spots.join(', ')
  return text.charAt(0).toUpperCase() + text.slice(1)
}
