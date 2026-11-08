/**
 * Per-seed lore page choice, keyed by session id. Persisted so a restored tab
 * replays the same journal. Only seeds that show lore pages are stored; the
 * default is a complete journal with no pages on the floors.
 */

import { persistentStore } from './store-utils'

const LORE_CHOICES_KEY = 'spd-analyzer-lore-choices'

type LoreChoices = Record<string, true>

const choicesCodec = {
  encode: (v: LoreChoices) => JSON.stringify(v),
  decode: (v: string): LoreChoices => {
    try {
      const parsed = JSON.parse(v) as unknown
      if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
        return {}
      }
      const choices: LoreChoices = {}
      for (const [id, value] of Object.entries(parsed)) {
        if (value === true) choices[id] = true
      }
      return choices
    } catch {
      return {}
    }
  },
}

/** Session ids whose floors show the journal's lore pages. */
export const $loreChoices = persistentStore<LoreChoices>(
  LORE_CHOICES_KEY,
  {},
  choicesCodec
)

export function loreChoiceFor(id: string): boolean {
  return $loreChoices.get()[id] === true
}

export function setLoreChoice(id: string, lorePages: boolean) {
  const { [id]: _previous, ...rest } = $loreChoices.get()
  $loreChoices.set(lorePages ? { ...rest, [id]: true } : rest)
}
