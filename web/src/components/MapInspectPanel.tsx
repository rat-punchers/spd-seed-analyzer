import { forwardRef, type ReactNode, useContext } from 'react'

import { GameText, ItemDetail } from '@/components/item-detail'
import {
  ItemPortrait,
  MobPortrait,
  PlantPortrait,
  StairPortrait,
  TrapPortrait,
} from '@/components/map-inspect-art'
import {
  HeldTrinketContext,
  PICKED_TRINKET_LEVEL,
} from '@/components/seed/trinket-offers'
import { Badge } from '@/components/ui/badge'
import {
  mobClassName,
  plantClassName,
  trapClassName,
} from '@/generated/game-ids'
import { itemLabel } from '@/lib/item-title'
import {
  itemClass,
  itemEnhancement,
  itemTitle,
  type MapInspectEntry,
  type MapInspection,
  mobTitle,
  objectText,
  plantTitle,
  transitionTitle,
  trapTitle,
} from '@/lib/map-inspect'
import type { IdentityMaps, MapMobState } from '@/lib/spd-wasm'
import { cn } from '@/lib/utils'

const MOB_STATE_NOTE: Record<MapMobState, string | null> = {
  sleeping: 'Asleep when the floor is generated.',
  wandering: 'Awake and wandering when the floor is generated.',
  passive: null,
}

function ObjectSection({
  title,
  desc,
  note,
}: {
  title: string
  desc?: string | null
  note?: string | null
}) {
  return (
    <div className="flex flex-col gap-1">
      <p className="font-semibold text-popover-foreground">{title}</p>
      {note ? <p className="text-sm text-muted-foreground">{note}</p> : null}
      {desc ? (
        <p className="whitespace-pre-wrap text-sm text-muted-foreground">
          <GameText text={desc} />
        </p>
      ) : null}
    </div>
  )
}

function PictureRow({
  picture,
  children,
}: {
  picture?: ReactNode
  children: ReactNode
}) {
  if (!picture) return children
  return (
    <div className="flex items-start gap-3">
      <div className="shrink-0">{picture}</div>
      <div className="min-w-0 flex-1">{children}</div>
    </div>
  )
}

function EntrySection({
  entry,
  identities,
  tileset,
}: {
  entry: MapInspectEntry
  identities: IdentityMaps
  tileset: string
}) {
  switch (entry.kind) {
    case 'item': {
      const title = itemTitle(entry.item)
      return (
        <PictureRow
          picture={
            <ItemPortrait
              item={entry.item}
              title={title}
              identities={identities}
            />
          }
        >
          <ItemDetail
            title={title}
            className={itemClass(entry.item)}
            cursed={entry.item.cursed}
            enhancement={itemEnhancement(entry.item)}
            location={entry.place}
          />
        </PictureRow>
      )
    }
    case 'mob': {
      const drops = entry.mob.items ?? []
      const crystal = mobClassName(entry.mob.mob) === 'CrystalMimic'
      const title = mobTitle(entry.mob)
      return (
        <div className="flex flex-col gap-2">
          <PictureRow
            picture={
              <MobPortrait mob={entry.mob.mob} items={drops} label={title} />
            }
          >
            <ObjectSection
              title={title}
              note={MOB_STATE_NOTE[entry.mob.state]}
              desc={objectText(mobClassName(entry.mob.mob))?.desc}
            />
          </PictureRow>
          {drops.map((item, index) => {
            const dropTitle = itemTitle(item)
            return (
              <PictureRow
                key={`${itemClass(item)}-${index}`}
                picture={
                  <ItemPortrait
                    item={item}
                    title={dropTitle}
                    identities={identities}
                  />
                }
              >
                <ItemDetail
                  title={dropTitle}
                  className={itemClass(item)}
                  cursed={item.cursed}
                  enhancement={itemEnhancement(item)}
                  location={
                    crystal
                      ? 'Drops when defeated. Lost if the mimic escapes.'
                      : 'Drops when defeated.'
                  }
                />
              </PictureRow>
            )
          })}
        </div>
      )
    }
    case 'loot':
      return (
        <ObjectSection
          title={entry.label}
          note="Room loot whose exact item is not modeled at this tile."
        />
      )
    case 'trap': {
      const title = trapTitle(entry.trap)
      return (
        <PictureRow picture={<TrapPortrait trap={entry.trap} label={title} />}>
          <ObjectSection
            title={title}
            note={entry.trap.active ? null : 'Inactive.'}
            desc={objectText(trapClassName(entry.trap.trap))?.desc}
          />
        </PictureRow>
      )
    }
    case 'plant': {
      const title = plantTitle(entry.plant)
      return (
        <PictureRow
          picture={<PlantPortrait plant={entry.plant} label={title} />}
        >
          <ObjectSection
            title={title}
            desc={objectText(plantClassName(entry.plant.plant))?.desc}
          />
        </PictureRow>
      )
    }
    case 'transition': {
      const title = transitionTitle(entry.transition)
      return (
        <PictureRow
          picture={
            <StairPortrait
              tileset={tileset}
              type={entry.transition.type}
              label={title}
            />
          }
        >
          <ObjectSection title={title} />
        </PictureRow>
      )
    }
  }
}

type Props = {
  inspection: MapInspection
  identities: IdentityMaps
  tileset: string
  pinned: boolean
  style?: React.CSSProperties
}

/** Description panel for one map tile: everything generated on it. */
export const MapInspectPanel = forwardRef<HTMLDivElement, Props>(
  function MapInspectPanel(
    { inspection, identities, tileset, pinned, style },
    ref
  ) {
    const { entries, uncertain, perRun, impShop, hidden, secretDoor } =
      inspection
    const heldTrinket = useContext(HeldTrinketContext)
    return (
      <div
        ref={ref}
        data-slot="map-inspect"
        role="dialog"
        aria-label="Tile details"
        className={cn(
          'absolute z-20 flex max-h-80 w-80 max-w-[calc(100%-1rem)] flex-col gap-2 overflow-y-auto rounded-lg border bg-popover p-3.5 text-sm text-popover-foreground shadow-window inset-shadow-raised',
          !pinned && 'pointer-events-none'
        )}
        style={style}
      >
        {uncertain || hidden ? (
          <div className="flex flex-wrap gap-1">
            {hidden ? (
              <Badge variant="secondary">
                {secretDoor ? 'Secret door' : 'Hidden'}
              </Badge>
            ) : null}
            {uncertain ? <Badge variant="warning">May differ</Badge> : null}
          </div>
        ) : null}
        {secretDoor && entries.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            Looks like a wall until searched. The room behind it is shown
            pulsing on the map.
          </p>
        ) : null}
        {entries.map((entry, index) => (
          <div
            key={`${entry.kind}-${index}`}
            className={cn(index > 0 && 'border-t pt-2')}
          >
            <EntrySection
              entry={entry}
              identities={identities}
              tileset={tileset}
            />
          </div>
        ))}
        {uncertain ? (
          <p className="border-t pt-2 text-xs text-muted-foreground">
            {perRun
              ? 'The game places these consumables in a random order that is not tied to the seed, so which one lands here changes on every run.'
              : impShop
                ? 'The Imp opens this shop only after you finish his quest and defeat the Dwarf King. Items found or trinkets held before this floor can change the stock.'
                : heldTrinket
                  ? `Shown as generated with ${itemLabel(heldTrinket)} +${PICKED_TRINKET_LEVEL} held. Items found before this floor can change what spawns here.`
                  : 'Shown as generated for a fresh run with no trinkets. Items found or trinkets held before this floor can change what spawns here.'}
          </p>
        ) : null}
      </div>
    )
  }
)
