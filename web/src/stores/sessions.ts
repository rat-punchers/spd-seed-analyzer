/**
 * Seed session lifecycle: open tabs, analyze, rehydrate, close.
 *
 * Persisted: open seed inputs, active tab id, per-seed trinket, challenges,
 * and lore pages.
 * Ephemeral: draft input, session runtime (reports), analyzing, form error.
 */

import { isCompleteCanonicalSeed } from '@/lib/canonical-seed'
import type { Challenge, SeedReport } from '@/lib/spd-wasm'
import { analyzeSeedInWorker, type WorkerTask } from '@/lib/spd-worker-client'
import { challengeChoiceFor, setChallengeChoice } from './challenge-choices'
import { loreChoiceFor, setLoreChoice } from './lore-choices'
import { AppStore, derivedStore, persistentStore } from './store-utils'
import { setTrinketChoice, trinketChoiceFor } from './trinket-choices'

// —— keys / limits ————————————————————————————————————————————————

const SAVED_SEEDS_KEY = 'spd-analyzer-open-seeds'
const ACTIVE_SEED_KEY = 'spd-analyzer-active-seed'

/** Max open/saved seeds (FIFO when exceeded). */
export const MAX_SAVED_SEEDS = 10

/** Full main-path depth range (SPD clamps to 26). */
export const ANALYZE_FLOORS = 26

/** Delay between sequential re-analyzes during session restore (ms). */
const REANALYZE_GAP_MS = 350

// —— types ————————————————————————————————————————————————————————

export type SessionStatus =
  | 'pending'
  | 'loading'
  | 'ready'
  | 'cancelled'
  | 'error'

export type SeedSession = {
  /** Stable tab id (normalized input key). */
  id: string
  /** Original user input used for analyze. */
  input: string
  status: SessionStatus
  report: SeedReport | null
  /** Catalyst trinket class held at +3 for this report; null is no trinket. */
  trinket: string | null
  /** Enabled generation-affecting challenges for this report. */
  challenges: Challenge[]
  /** Floors replay a fresh journal, so guide and alchemy pages spawn. */
  lorePages: boolean
  error: string | null
  startedAt: number | null
  finishedAt: number | null
}

// —— helpers ——————————————————————————————————————————————————————

const stringArrayCodec = {
  encode: (v: string[]) => JSON.stringify(v),
  decode: (v: string): string[] => {
    try {
      const parsed = JSON.parse(v) as unknown
      if (!Array.isArray(parsed)) return []
      return parsed
        .filter((s): s is string => typeof s === 'string')
        .map((s) => s.trim())
        .filter(Boolean)
        .slice(0, MAX_SAVED_SEEDS)
    } catch {
      return []
    }
  },
}

const nullableStringCodec = {
  encode: (v: string | null) => v ?? '',
  decode: (v: string) => (v === '' ? null : v),
}

export function normalizeSeedInput(raw: string): string {
  return raw.trim()
}

export function sessionIdFor(input: string): string {
  return normalizeSeedInput(input).toUpperCase()
}

/** Tab title: always the stored user input (not the canonical code). */
export function tabLabel(session: SeedSession): string {
  return session.input
}

function sleep(ms: number) {
  return new Promise<void>((resolve) => {
    setTimeout(resolve, ms)
  })
}

// —— persistent stores ————————————————————————————————————————————

/** Ordered list of seed inputs to keep open (max {@link MAX_SAVED_SEEDS}). */
export const $savedSeedInputs = persistentStore<string[]>(
  SAVED_SEEDS_KEY,
  [],
  stringArrayCodec
)

export const $activeSeedId = persistentStore<string | null>(
  ACTIVE_SEED_KEY,
  null,
  nullableStringCodec
)

// —— ephemeral stores ——————————————————————————————————————————————

/** Draft seed field in the left menu. */
export const $seedInput = new AppStore('')

/** Runtime sessions (reports not persisted — re-analyzed on load). */
export const $sessions = new AppStore<SeedSession[]>([])

export const $analyzing = new AppStore(false)

export const $formError = new AppStore<string | null>(null)

export const $sessionCount = derivedStore(
  [$sessions] as const,
  () => $sessions.get().length
)

// —— internal rehydrate control ————————————————————————————————————

const closedIds = new Set<string>()
const analyzeTasks = new Map<string, WorkerTask<SeedReport>>()
let rehydrateGen = 0

// —— pure mutations ————————————————————————————————————————————————

function setSavedInputs(inputs: string[]) {
  const cleaned = inputs
    .map(normalizeSeedInput)
    .filter(Boolean)
    .slice(0, MAX_SAVED_SEEDS)
  // Deduplicate by session id (keep first occurrence order).
  const seen = new Set<string>()
  const unique: string[] = []
  for (const input of cleaned) {
    const id = sessionIdFor(input)
    if (seen.has(id)) continue
    seen.add(id)
    unique.push(input)
  }
  // Cap from the end (keep newest) if still over limit after dedupe.
  $savedSeedInputs.set(
    unique.length > MAX_SAVED_SEEDS
      ? unique.slice(unique.length - MAX_SAVED_SEEDS)
      : unique
  )
}

function patchSession(id: string, patch: Partial<SeedSession>) {
  const prev = $sessions.get()
  if (!prev.some((s) => s.id === id)) return
  $sessions.set(prev.map((s) => (s.id === id ? { ...s, ...patch } : s)))
}

async function runAnalyze(id: string, input: string): Promise<boolean> {
  const trinket = trinketChoiceFor(id)
  const challenges = challengeChoiceFor(id)
  const lorePages = loreChoiceFor(id)
  patchSession(id, {
    status: 'loading',
    trinket,
    challenges,
    lorePages,
    error: null,
    startedAt: Date.now(),
    finishedAt: null,
  })
  const task = analyzeSeedInWorker(
    input,
    ANALYZE_FLOORS,
    trinket,
    challenges,
    lorePages
  )
  analyzeTasks.get(id)?.cancel()
  analyzeTasks.set(id, task)
  try {
    const report = await task.promise
    console.log(`[Analyze] ${input} result`, report)
    if (analyzeTasks.get(id) !== task) return false
    patchSession(id, {
      status: 'ready',
      report,
      error: null,
      finishedAt: Date.now(),
    })
    return true
  } catch (err) {
    if (analyzeTasks.get(id) !== task) return false
    patchSession(id, {
      status: 'error',
      report: null,
      error: err instanceof Error ? err.message : String(err),
      finishedAt: Date.now(),
    })
    return false
  } finally {
    if (analyzeTasks.get(id) === task) analyzeTasks.delete(id)
  }
}

// —— public actions ————————————————————————————————————————————————

export function setSeedInput(value: string) {
  $seedInput.set(value)
}

export function setActiveSeed(id: string | null) {
  $activeSeedId.set(id)
}

/**
 * Close a seed tab. Removes from runtime sessions + persisted list.
 */
export function closeSeedSession(id: string) {
  closedIds.add(id)
  setTrinketChoice(id, null)
  setChallengeChoice(id, [])
  setLoreChoice(id, false)
  analyzeTasks.get(id)?.cancel()
  analyzeTasks.delete(id)
  const prev = $sessions.get()
  const idx = prev.findIndex((s) => s.id === id)
  if (idx < 0) return
  const next = prev.filter((s) => s.id !== id)
  $sessions.set(next)
  setSavedInputs(next.map((s) => s.input))
  if ($activeSeedId.get() === id) {
    const fallback = next[idx] ?? next[idx - 1] ?? next[0] ?? null
    $activeSeedId.set(fallback?.id ?? null)
  }
}

export function cancelSeedAnalysis(id: string) {
  const session = $sessions.get().find((item) => item.id === id)
  if (session?.status !== 'loading' && session?.status !== 'pending') return
  analyzeTasks.get(id)?.cancel()
  analyzeTasks.delete(id)
  patchSession(id, { status: 'cancelled', error: null, finishedAt: Date.now() })
  $analyzing.set(analyzeTasks.size > 0)
}

/**
 * Hold a Catalyst trinket (class name) for later floors of a seed, or clear
 * it with null, and regenerate. The previous report stays visible meanwhile.
 */
export async function selectTrinket(
  id: string,
  trinket: string | null
): Promise<void> {
  const session = $sessions.get().find((item) => item.id === id)
  if (!session || trinketChoiceFor(id) === trinket) return
  setTrinketChoice(id, trinket)
  $analyzing.set(true)
  try {
    await runAnalyze(id, session.input)
  } finally {
    $analyzing.set(analyzeTasks.size > 0)
  }
}

/**
 * Set the run's enabled challenges for a seed and regenerate. The previous
 * report stays visible meanwhile.
 */
export async function selectChallenges(
  id: string,
  challenges: readonly Challenge[]
): Promise<void> {
  const session = $sessions.get().find((item) => item.id === id)
  if (!session) return
  const previous = challengeChoiceFor(id)
  setChallengeChoice(id, challenges)
  if (challengeChoiceFor(id).join() === previous.join()) return
  $analyzing.set(true)
  try {
    await runAnalyze(id, session.input)
  } finally {
    $analyzing.set(analyzeTasks.size > 0)
  }
}

/**
 * Show or hide the journal's lore pages for a seed and regenerate. The
 * previous report stays visible meanwhile.
 */
export async function selectLorePages(
  id: string,
  lorePages: boolean
): Promise<void> {
  const session = $sessions.get().find((item) => item.id === id)
  if (!session || loreChoiceFor(id) === lorePages) return
  setLoreChoice(id, lorePages)
  $analyzing.set(true)
  try {
    await runAnalyze(id, session.input)
  } finally {
    $analyzing.set(analyzeTasks.size > 0)
  }
}

async function analyzeSeedInputInternal(
  rawInput: string,
  clearDraftOnCreate: boolean
): Promise<void> {
  const input = normalizeSeedInput(rawInput)
  if (!input) return

  $formError.set(null)
  const id = sessionIdFor(input)
  const sessions = $sessions.get()
  const existing = sessions.find((s) => s.id === id)

  if (existing) {
    $activeSeedId.set(id)
    if (existing.status === 'ready' || existing.status === 'loading') {
      return
    }
    $analyzing.set(true)
    try {
      await runAnalyze(id, existing.input)
    } finally {
      $analyzing.set(false)
    }
    return
  }

  // Cap: drop oldest session(s) so we stay within MAX after adding.
  let nextSessions = [...sessions]
  while (nextSessions.length >= MAX_SAVED_SEEDS) {
    const oldest = nextSessions[0]
    closedIds.add(oldest.id)
    setTrinketChoice(oldest.id, null)
    setChallengeChoice(oldest.id, [])
    setLoreChoice(oldest.id, false)
    nextSessions = nextSessions.slice(1)
  }

  const session: SeedSession = {
    id,
    input,
    status: 'loading',
    report: null,
    trinket: null,
    challenges: [],
    lorePages: false,
    error: null,
    startedAt: Date.now(),
    finishedAt: null,
  }
  nextSessions = [...nextSessions, session]
  $sessions.set(nextSessions)
  setSavedInputs(nextSessions.map((s) => s.input))
  $activeSeedId.set(id)
  if (clearDraftOnCreate) {
    $seedInput.set('')
  }
  $analyzing.set(true)
  try {
    await runAnalyze(id, input)
  } finally {
    $analyzing.set(false)
  }
}

/**
 * Open and analyze any seed input through the normal persisted session lifecycle.
 * Used by entry points such as seed-finder results that do not own the draft field.
 */
export async function analyzeSeedInput(input: string): Promise<void> {
  await analyzeSeedInputInternal(input, false)
}

/**
 * Analyze the draft seed input (or re-focus existing tab).
 * Enforces {@link MAX_SAVED_SEEDS} by dropping oldest sessions.
 */
export async function analyzeDraftSeed(): Promise<void> {
  const input = $seedInput.get()
  if (!isCompleteCanonicalSeed(input)) return
  await analyzeSeedInputInternal(input, true)
}

/**
 * Rebuild runtime sessions from persisted seed inputs and re-analyze slowly.
 * Safe under React Strict Mode (cleanup aborts via generation counter).
 *
 * @returns cleanup function that aborts the in-flight loop
 */
export function startSessionRehydrate(): () => void {
  const saved = $savedSeedInputs
    .get()
    .map(normalizeSeedInput)
    .filter(Boolean)
    .slice(0, MAX_SAVED_SEEDS)

  // Normalize / re-cap persisted list if it grew before the limit existed.
  if (
    saved.length !== $savedSeedInputs.get().length ||
    $savedSeedInputs.get().some((s, i) => normalizeSeedInput(s) !== saved[i])
  ) {
    setSavedInputs(saved)
  }

  if (saved.length === 0) {
    $sessions.set([])
    return () => {}
  }

  const restored: SeedSession[] = saved.map((input) => {
    const id = sessionIdFor(input)
    return {
      id,
      input,
      status: 'pending' as const,
      report: null,
      trinket: null,
      challenges: [],
      lorePages: false,
      error: null,
      startedAt: null,
      finishedAt: null,
    }
  })

  closedIds.clear()
  $sessions.set(restored)

  const preferred = $activeSeedId.get()
  const active =
    preferred && restored.some((s) => s.id === preferred)
      ? preferred
      : (restored[0]?.id ?? null)
  $activeSeedId.set(active)

  const gen = ++rehydrateGen
  let cancelled = false

  ;(async () => {
    $analyzing.set(true)
    try {
      for (let i = 0; i < restored.length; i++) {
        if (cancelled || rehydrateGen !== gen) break
        const s = restored[i]
        // Skip only if closed during rehydrate — do not consult $sessions
        // for membership (race with React paint).
        if (closedIds.has(s.id)) continue
        await runAnalyze(s.id, s.input)
        if (i < restored.length - 1 && !cancelled && rehydrateGen === gen) {
          await sleep(REANALYZE_GAP_MS)
        }
      }
    } finally {
      if (!cancelled && rehydrateGen === gen) {
        $analyzing.set(false)
      }
    }
  })()

  return () => {
    cancelled = true
    rehydrateGen += 1
  }
}
