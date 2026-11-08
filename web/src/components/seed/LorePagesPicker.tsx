import { ItemBox } from '@/components/ItemBox'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldLabel,
} from '@/components/ui/field'
import { itemSprite } from '@/lib/item-kind'

/** Whether one seed session's floors show the journal's lore pages. */
export type LoreChoice = {
  shown: boolean
  /** A regenerated report is on its way. */
  busy: boolean
  select: (shown: boolean) => void
}

/**
 * The journal changes generated floors: a complete one spawns no pages, a
 * new one places guide and alchemy pages that also move later loot.
 */
export function LorePagesPicker({ choice }: { choice: LoreChoice }) {
  return (
    <section
      data-lore-picker
      aria-labelledby="lore-picker-title"
      aria-busy={choice.busy}
      className="flex flex-col gap-3"
    >
      <div className="flex items-center gap-2">
        <ItemBox
          scale={2}
          panel={false}
          border={false}
          bg={false}
          sprites={itemSprite({ className: 'GuidePage', category: 'other' })}
        />
        <h3 id="lore-picker-title" className="font-pixel text-px-1">
          Lore pages
        </h3>
      </div>
      <Field orientation="horizontal">
        <Checkbox
          id="lore-pages"
          checked={choice.shown}
          disabled={choice.busy}
          onCheckedChange={(checked) => choice.select(checked === true)}
        />
        <FieldContent>
          <FieldLabel htmlFor="lore-pages">Show lore pages on maps</FieldLabel>
          <FieldDescription aria-live="polite">
            {choice.busy
              ? 'Regenerating floors…'
              : choice.shown
                ? 'Floors are generated for a new journal: guide pages and alchemy pages appear, and later loot on those floors can move.'
                : 'Floors are generated for a completed journal, so no guide or alchemy pages spawn.'}
          </FieldDescription>
        </FieldContent>
      </Field>
    </section>
  )
}
