/**
 * Pure grouping of a floor's items: what the report lists, and which of it
 * is notable. Shared by the floor item sections and the floor strip.
 */
import { isNotableSpawn } from '@/lib/item-notable'
import type { FloorReport, ItemGroup, ItemVariant } from '@/lib/spd-wasm'

/** Floor/quest group after preferring a variable sample over its contract. */
export type VisibleItemGroup = ItemGroup & {
  /** Constrained sibling kept for the item description panel. */
  contract?: ItemVariant
}

function splitNotableItems(items: VisibleItemGroup[]) {
  const notable: VisibleItemGroup[] = []
  const other: VisibleItemGroup[] = []
  for (const group of items) {
    const notableVariants = group.variants.filter(isNotableSpawn)
    const otherVariants = group.variants.filter((item) => !isNotableSpawn(item))
    if (notableVariants.length > 0) {
      notable.push({ ...group, variants: notableVariants })
    }
    if (otherVariants.length > 0) {
      other.push({ ...group, variants: otherVariants })
    }
  }
  return { notable, other }
}

function itemSection(
  item: ItemGroup
): 'shop' | 'quest' | 'guaranteed' | 'loot' | 'general' {
  const source = item.source ?? ''
  if (source.includes('ShopRoom')) return 'shop'
  if (
    /Ghost\.Quest|Wandmaker\.Quest|Blacksmith\.Quest|Imp\.Quest/.test(source)
  ) {
    return 'quest'
  }
  if (source === 'guaranteed floor spawn' || source === 'forced') {
    return 'guaranteed'
  }
  if (source === 'heap') return 'loot'
  return 'general'
}

export function visibleItemGroups(items: ItemGroup[]): VisibleItemGroup[] {
  return items
    .map((group): VisibleItemGroup => {
      // Room contracts ship constrained + variable siblings for one spawn.
      // Show only the sample; keep the contract for the examine panel.
      const contract = group.variants.find(
        (item) => item.prediction === 'constrained'
      )
      const hasVariable = group.variants.some(
        (item) => item.prediction === 'variable'
      )
      return {
        ...group,
        contract: hasVariable ? contract : undefined,
        variants: group.variants.filter(
          (item) => !(item.prediction === 'constrained' && hasVariable)
        ),
      }
    })
    .map((group) => ({
      ...group,
      // Drop blank constrained stubs that never grew a concrete sample.
      // Quest cards keep them as their universal reward fallback.
      variants:
        itemSection(group) === 'quest'
          ? group.variants
          : group.variants.filter(
              (item) =>
                item.item != null ||
                (item.candidates?.length ?? 0) > 0 ||
                item.prediction !== 'constrained'
            ),
    }))
    .filter((group) => group.variants.length > 0)
}

export function partitionFloorItems(items: VisibleItemGroup[]) {
  return {
    general: items.filter((item) => itemSection(item) === 'general'),
    guaranteed: items.filter((item) => itemSection(item) === 'guaranteed'),
    loot: items.filter((item) => itemSection(item) === 'loot'),
    shop: items.filter((item) => itemSection(item) === 'shop'),
    quest: items.filter((item) => itemSection(item) === 'quest'),
  }
}

/**
 * Notable and other items as the floor list shows them. Shop stock stays off
 * the list and quest rewards stay on the quest card.
 */
export function floorItemGroups(floor: FloorReport) {
  const groups = partitionFloorItems(visibleItemGroups(floor.items))
  return splitNotableItems([
    ...groups.guaranteed,
    ...groups.loot,
    ...groups.general,
  ])
}
