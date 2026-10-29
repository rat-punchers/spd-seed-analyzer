import { useEffect, useState } from 'react'

import { GameSprite } from '@/components/GameSprite'
import { ItemBox } from '@/components/ItemBox'
import type { MobId } from '@/generated/game-ids'
import { filmFrames, type SpriteSheet } from '@/lib/game-objects'
import { itemAppearance } from '@/lib/identity'
import { itemSprite } from '@/lib/item-kind'
import { itemClass, itemEnhancement } from '@/lib/map-inspect'
import { mobSheetUrl, mobSprite } from '@/lib/map-mob-sprites'
import type {
  IdentityMaps,
  MapHeapItem,
  MapPlant,
  MapTrap,
} from '@/lib/spd-wasm'

/** Longer edge of a portrait, in CSS pixels. Matches the quest-card mobs. */
const PORTRAIT = 36
const TILE = 16
const TILE_COLS = 16

const FEATURES: SpriteSheet = {
  url: '/assets/environment/terrain_features.png',
  width: 256,
  height: 256,
}

const TILE_SHEETS = new Set([
  'sewers',
  'prison',
  'caves',
  'caves_crystal',
  'caves_gnoll',
  'city',
  'halls',
])

const sheets = new Map<string, SpriteSheet>()

function useLoadedSheet(url: string | null) {
  const [sheet, setSheet] = useState<SpriteSheet | null>(() =>
    url ? (sheets.get(url) ?? null) : null
  )
  useEffect(() => {
    if (!url) {
      setSheet(null)
      return
    }
    const known = sheets.get(url)
    if (known) {
      setSheet(known)
      return
    }
    setSheet(null)
    let cancel = false
    const image = new Image()
    image.onload = () => {
      const next = {
        url,
        width: image.naturalWidth,
        height: image.naturalHeight,
      }
      sheets.set(url, next)
      if (!cancel) setSheet(next)
    }
    image.src = url
    return () => {
      cancel = true
    }
  }, [url])
  return sheet
}

function tileFrame(index: number) {
  return {
    x: (index % TILE_COLS) * TILE,
    y: Math.floor(index / TILE_COLS) * TILE,
    w: TILE,
    h: TILE,
  }
}

function TilePortrait({
  sheet,
  index,
  label,
}: {
  sheet: SpriteSheet
  index: number
  label: string
}) {
  return (
    <GameSprite
      sheet={sheet}
      frames={[tileFrame(index)]}
      size={PORTRAIT}
      label={label}
    />
  )
}

/** `TerrainFeaturesTilemap.getTrapVisual`: inactive traps use the black row. */
export function TrapPortrait({
  trap,
  label,
}: {
  trap: MapTrap
  label: string
}) {
  const color = trap.active ? trap.color : 8
  return (
    <TilePortrait
      sheet={FEATURES}
      index={color + trap.shape * TILE_COLS}
      label={label}
    />
  )
}

/** `TerrainFeaturesTilemap.getPlantVisual`: plant row starts at image + 7×16. */
export function PlantPortrait({
  plant,
  label,
}: {
  plant: MapPlant
  label: string
}) {
  return (
    <TilePortrait
      sheet={FEATURES}
      index={plant.image + 7 * TILE_COLS}
      label={label}
    />
  )
}

/** Entrance and exit cells on `tiles_{region}.png`. The index is the same cell on every sheet. */
export function StairPortrait({
  tileset,
  type,
  label,
}: {
  tileset: string
  type: string
  label: string
}) {
  if (!TILE_SHEETS.has(tileset)) return null
  const index =
    type === 'REGULAR_EXIT' || type === 'BRANCH_EXIT'
      ? 17
      : type === 'SURFACE' ||
          type === 'REGULAR_ENTRANCE' ||
          type === 'BRANCH_ENTRANCE'
        ? 16
        : null
  if (index == null) return null
  return (
    <TilePortrait
      sheet={{
        url: `/assets/environment/tiles_${tileset}.png`,
        width: 256,
        height: 256,
      }}
      index={index}
      label={label}
    />
  )
}

export function MobPortrait({
  mob,
  items,
  label,
}: {
  mob: MobId
  items?: readonly MapHeapItem[]
  label: string
}) {
  const def = mobSprite(mob, items)
  const sheet = useLoadedSheet(def ? mobSheetUrl(def.sheet) : null)
  if (!def) return null
  if (!sheet) {
    return <span className="inline-block size-9 shrink-0" aria-hidden />
  }
  return (
    <GameSprite
      sheet={sheet}
      frames={filmFrames(sheet.width, def.width, def.height, def.frames)}
      fps={def.fps}
      size={PORTRAIT}
      label={label}
    />
  )
}

function appearanceCategory(className: string) {
  if (className.startsWith('PotionOf')) return 'potion'
  if (className.startsWith('ScrollOf')) return 'scroll'
  if (className.startsWith('RingOf')) return 'ring'
  return 'other'
}

export function ItemPortrait({
  item,
  title,
  identities,
}: {
  item: MapHeapItem
  title: string
  identities: IdentityMaps
}) {
  const className = itemClass(item)
  return (
    <ItemBox
      scale={2}
      panel={false}
      title={title}
      cursed={item.cursed}
      enhancement={itemEnhancement(item)}
      level={item.level}
      sprites={itemSprite({
        className,
        appearance: itemAppearance(
          { category: appearanceCategory(className), class_name: className },
          identities
        ),
      })}
    >
      {item.quantity > 1 ? (
        <span className="ink-outline font-pixel text-px-1 pointer-events-none absolute -top-0.5 left-0.5 leading-[20px] text-[#f0c040]">
          {item.quantity}
        </span>
      ) : null}
    </ItemBox>
  )
}
