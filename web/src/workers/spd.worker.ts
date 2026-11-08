/// <reference lib="webworker" />

import type { SeedSearchMatch, SeedSearchResult } from '@/lib/spd-wasm'
import { analyzeSeed, searchSeeds } from '@/lib/spd-wasm'
import type {
  SpdWorkerRequest,
  SpdWorkerResponse,
} from '@/lib/spd-worker-protocol'

const context = self as DedicatedWorkerGlobalScope
const SEARCH_CHUNK_SIZE = 1

function post(message: SpdWorkerResponse) {
  context.postMessage(message)
}

async function runSearch(
  request: Extract<SpdWorkerRequest, { type: 'search' }>
) {
  const config = request.request
  let scanned = 0
  let cursor: number | null = config.startSeed
  let exhausted = false
  let latest: SeedSearchResult | null = null
  const matches = new Map<number, SeedSearchMatch>()

  while (
    scanned < config.candidateCount &&
    matches.size < config.maxMatches &&
    cursor !== null &&
    !exhausted
  ) {
    post({
      type: 'search-candidate',
      candidateNumber: scanned + 1,
      seed: cursor,
      depth: config.floors,
    })
    const result = await searchSeeds({
      ...config,
      startSeed: cursor,
      candidateCount: Math.min(
        SEARCH_CHUNK_SIZE,
        config.candidateCount - scanned
      ),
      maxMatches: config.maxMatches - matches.size,
    })
    scanned += result.candidatesScanned
    cursor = result.nextSeed
    exhausted = result.exhausted
    for (const match of result.matches) matches.set(match.seed.numeric, match)
    latest = {
      ...result,
      requestedCandidates: config.candidateCount,
      candidatesScanned: scanned,
      nextSeed: cursor,
      exhausted,
      matches: Array.from(matches.values()),
    }
    post({ type: 'search-progress', result: latest })
    if (result.candidatesScanned === 0) break
  }

  if (!latest) {
    latest = await searchSeeds({ ...config, candidateCount: 0 })
  }
  post({ type: 'search-complete', result: latest })
}

context.onmessage = async (event: MessageEvent<SpdWorkerRequest>) => {
  try {
    if (event.data.type === 'analyze') {
      const report = await analyzeSeed(
        event.data.input,
        event.data.floors,
        event.data.trinket,
        event.data.challenges,
        event.data.lorePages
      )
      post({ type: 'analysis-complete', report })
      return
    }
    await runSearch(event.data)
  } catch (error) {
    post({
      type: 'error',
      message: error instanceof Error ? error.message : String(error),
    })
  }
}
