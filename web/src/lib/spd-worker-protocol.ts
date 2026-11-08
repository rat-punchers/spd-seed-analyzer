import type {
  Challenge,
  SeedReport,
  SeedSearchRequest,
  SeedSearchResult,
} from '@/lib/spd-wasm'

export type SpdWorkerRequest =
  | {
      type: 'analyze'
      input: string
      floors: number
      trinket: string | null
      challenges: Challenge[]
      lorePages: boolean
    }
  | { type: 'search'; request: SeedSearchRequest }

export type SpdWorkerResponse =
  | { type: 'analysis-complete'; report: SeedReport }
  | {
      type: 'search-candidate'
      candidateNumber: number
      seed: number
      depth: number
    }
  | { type: 'search-progress'; result: SeedSearchResult }
  | { type: 'search-complete'; result: SeedSearchResult }
  | { type: 'error'; message: string }
