/**
 * App-wide state via TanStack Store — public façade.
 *
 * Implementation is split by concern:
 * - `sessions` — seed tabs, analyze, rehydrate
 * - `meta` — SPD version/commit from wasm
 * - `theme` — light / dark / system preference
 * - `mode` — current mode (analyze / finder), persisted
 *
 * Consumers should keep importing from `@/stores/app`.
 */

import { persistentStore } from './store-utils'

export {
  $activeFinderId,
  $activeFinderSession,
  $finderCloseConfirmationId,
  $finderRunning,
  $finderSessions,
  cancelFinderSearch,
  closeFinderSession,
  type FinderSession,
  MAX_FINDER_SESSIONS,
  setActiveFinder,
  setFinderCloseConfirmation,
  startFinderRehydrate,
  startFinderSearch,
} from './finder'
export { $meta, loadSpdMeta, type SpdMeta } from './meta'
export {
  $screenshotScan,
  analyzeScreenshotSeeds,
  dismissScreenshotScan,
  dropScreenshot,
  hideScreenshotDropTarget,
  type ScreenshotScan,
  scanScreenshot,
  showScreenshotDropTarget,
} from './screenshot'
export {
  $activeSeedId,
  $analyzing,
  $formError,
  $savedSeedInputs,
  $seedInput,
  $sessionCount,
  $sessions,
  ANALYZE_FLOORS,
  analyzeDraftSeed,
  analyzeSeedInput,
  cancelSeedAnalysis,
  closeSeedSession,
  MAX_SAVED_SEEDS,
  normalizeSeedInput,
  type SeedSession,
  type SessionStatus,
  selectChallenges,
  selectLorePages,
  selectTrinket,
  sessionIdFor,
  setActiveSeed,
  setSeedInput,
  startSessionRehydrate,
  tabLabel,
} from './sessions'
export {
  $theme,
  applyTheme,
  cycleTheme,
  initTheme,
  resolvedTheme,
  setTheme,
  type Theme,
} from './theme'
export {
  $dismissedVersionAlert,
  $finderForm,
  dismissVersionAlert,
  type FinderFormState,
} from './ui'

/**
 * Mode: analyze or find-seed. Persisted to localStorage.
 */
export type AppMode = 'analyze' | 'finder'

const MODE_KEY = 'spd-analyzer-mode'
const modeCodec = {
  encode: (v: AppMode) => v,
  decode: (v: string): AppMode =>
    v === 'analyze' || v === 'finder' ? v : 'analyze',
}

export const $mode = persistentStore<AppMode>(MODE_KEY, 'analyze', modeCodec)

export function setMode(value: AppMode) {
  $mode.set(value)
}
