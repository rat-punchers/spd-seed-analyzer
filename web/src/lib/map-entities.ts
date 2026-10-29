import { type ItemId, itemClassName, mobClassName } from '@/generated/game-ids'
import { SHEET_COLS, TILE_PX } from '@/lib/dungeon-tile-visuals'
import { ITEM_SHEET, resolveItemIconIndex } from '@/lib/item-icons'
import { itemSpriteSize } from '@/lib/item-sprite-rects'
import type { MapEntityAssets } from '@/lib/map-assets'
import { mobFrameAt, mobSourceRect, mobSprite } from '@/lib/map-mob-sprites'
import type { FloorMap, IdentityMaps, MapHeap, MapMob } from '@/lib/spd-wasm'

type Visibility = { item: boolean; mob: boolean }

const CONTAINER_FRAMES: Record<string, number> = {
  chest: 36,
  locked_chest: 37,
  crystal_chest: 38,
  tomb: 34,
  skeleton: 32,
  remains: 33,
}

function validCell(map: FloorMap, cell: number) {
  return cell >= 0 && cell < map.tiles.length
}

function isDiscoverable(map: FloorMap, cell: number) {
  return (
    map.discoverable.length !== map.tiles.length || map.discoverable[cell] === 1
  )
}

function drawSheetTile(
  ctx: CanvasRenderingContext2D,
  image: HTMLImageElement,
  visual: number,
  cell: number,
  width: number,
  scale: number
) {
  const size = TILE_PX * scale
  ctx.drawImage(
    image,
    (visual % SHEET_COLS) * TILE_PX,
    Math.floor(visual / SHEET_COLS) * TILE_PX,
    TILE_PX,
    TILE_PX,
    (cell % width) * size,
    Math.floor(cell / width) * size,
    size,
    size
  )
}

export function drawVisibleTraps(
  ctx: CanvasRenderingContext2D,
  assets: MapEntityAssets,
  map: FloorMap,
  scale: number
) {
  for (const trap of map.traps) {
    if (!trap.visible || !validCell(map, trap.cell)) continue
    if (!isDiscoverable(map, trap.cell)) continue
    const color = trap.active ? trap.color : 8
    drawSheetTile(
      ctx,
      assets.terrainFeatures,
      color + trap.shape * SHEET_COLS,
      trap.cell,
      map.width,
      scale
    )
  }
}

function appearanceFor(item: ItemId, identities: IdentityMaps) {
  const className = itemClassName(item)
  if (className.startsWith('PotionOf')) {
    return identities.potions.find((entry) => entry.item === item)?.appearance
  }
  if (className.startsWith('ScrollOf')) {
    return identities.scrolls.find((entry) => entry.item === item)?.appearance
  }
  if (className.startsWith('RingOf')) {
    return identities.rings.find((entry) => entry.item === item)?.appearance
  }
  return undefined
}

function itemFrame(heap: MapHeap, identities: IdentityMaps) {
  const container = CONTAINER_FRAMES[heap.heap_type]
  if (container != null) {
    const [width, height] = itemSpriteSize(container)
    return [container, width, height] as const
  }
  const item = heap.items[0]
  if (!item) {
    const [width, height] = itemSpriteSize(0)
    return [0, width, height] as const
  }
  const index = resolveItemIconIndex(itemClassName(item.item), {
    appearance: appearanceFor(item.item, identities),
  })
  const [width, height] = itemSpriteSize(index)
  return [index, width, height] as const
}

function drawHeap(
  ctx: CanvasRenderingContext2D,
  image: HTMLImageElement,
  map: FloorMap,
  heap: MapHeap,
  identities: IdentityMaps,
  scale: number
) {
  const [index, width, height] = itemFrame(heap, identities)
  const cellX = heap.cell % map.width
  const cellY = Math.floor(heap.cell / map.width)
  const raise = height < 8 ? 13 - height : 5
  const x = cellX * TILE_PX + (TILE_PX - width) / 2
  const y = (cellY + 1) * TILE_PX - height - raise
  ctx.drawImage(
    image,
    (index % ITEM_SHEET.cols) * ITEM_SHEET.size,
    Math.floor(index / ITEM_SHEET.cols) * ITEM_SHEET.size,
    width,
    height,
    x * scale,
    y * scale,
    width * scale,
    height * scale
  )
}

/** `MimicSprite.Ebony.hideMimic` alpha; revealed, it shows fully. */
export const LURKING_OPACITY = 0.2

/**
 * Hidden ebony mimics are drawn over walls so a door's sideways top does
 * not cover them.
 */
export function isLurkingMob(mob: MapMob) {
  return mobClassName(mob.mob) === 'EbonyMimic' && mob.state === 'passive'
}

function drawMob(
  ctx: CanvasRenderingContext2D,
  assets: MapEntityAssets,
  map: FloorMap,
  mob: MapMob,
  scale: number,
  seconds: number | null
) {
  const spriteDef = mobSprite(mob.mob, mob.items)
  const image = spriteDef && assets.mobs[spriteDef.sheet]
  if (!spriteDef || !image) return false
  const faint = mob.opacity != null && mob.opacity < 1
  const frame =
    faint && spriteDef.hidingFrame != null
      ? spriteDef.hidingFrame
      : seconds == null
        ? undefined
        : mobFrameAt(spriteDef, seconds, mob.cell * 7)
  const source = mobSourceRect(spriteDef, image, frame)
  const cellX = mob.cell % map.width
  const cellY = Math.floor(mob.cell / map.width)
  // CharSprite.worldToCamera: centered, raised by perspectiveRaise.
  const x = cellX * TILE_PX + (TILE_PX - spriteDef.width) / 2
  const y = (cellY + 1) * TILE_PX - spriteDef.height - spriteDef.raise
  ctx.save()
  if (mob.opacity != null) ctx.globalAlpha *= mob.opacity
  ctx.drawImage(
    image,
    source.x,
    source.y,
    spriteDef.width,
    spriteDef.height,
    x * scale,
    y * scale,
    spriteDef.width * scale,
    spriteDef.height * scale
  )
  ctx.restore()
  return true
}

/** Sprite sheets needed to draw this map's mobs. */
export function mapMobSheets(map: FloorMap): string[] {
  return map.mobs.flatMap((mob) => mobSprite(mob.mob)?.sheet ?? [])
}

export function drawKnownEntities(
  ctx: CanvasRenderingContext2D,
  assets: MapEntityAssets,
  map: FloorMap,
  identities: IdentityMaps,
  scale: number,
  visibility: Visibility
) {
  if (visibility.item) {
    for (const heap of map.heaps) {
      if (!validCell(map, heap.cell) || !isDiscoverable(map, heap.cell))
        continue
      drawHeap(ctx, assets.items, map, heap, identities, scale)
    }
  }
  if (visibility.mob) drawMobs(ctx, assets, map, scale, null)
}

/**
 * Mobs on their idle clip `seconds` in, or on the clip's first frame when
 * `seconds` is null (static renders and reduced motion).
 */
export function drawMobs(
  ctx: CanvasRenderingContext2D,
  assets: MapEntityAssets,
  map: FloorMap,
  scale: number,
  seconds: number | null
) {
  for (const mob of map.mobs) {
    if (!validCell(map, mob.cell) || !isDiscoverable(map, mob.cell)) continue
    if (isLurkingMob(mob)) continue
    drawMob(ctx, assets, map, mob, scale, seconds)
  }
}

/** Lurking mobs on their still frame, drawn over the wall layer. */
export function drawLurkingMobs(
  ctx: CanvasRenderingContext2D,
  assets: MapEntityAssets,
  map: FloorMap,
  scale: number
) {
  for (const mob of map.mobs) {
    if (!validCell(map, mob.cell) || !isDiscoverable(map, mob.cell)) continue
    if (isLurkingMob(mob)) drawMob(ctx, assets, map, mob, scale, null)
  }
}

export function hasAnimatedMobs(map: FloorMap) {
  return map.mobs.some((mob) => {
    const spriteDef = mobSprite(mob.mob)
    return spriteDef != null && spriteDef.fps > 0 && spriteDef.frames.length > 1
  })
}

export function exactEntityCells(map: FloorMap) {
  return {
    item: new Set(map.heaps.map((heap) => heap.cell)),
    mob: new Set(
      map.mobs
        .filter((mob) => mobSprite(mob.mob) != null)
        .map((mob) => mob.cell)
    ),
  }
}
