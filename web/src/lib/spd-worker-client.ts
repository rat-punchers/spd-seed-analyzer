import type {
  Challenge,
  SeedReport,
  SeedSearchRequest,
  SeedSearchResult,
} from '@/lib/spd-wasm'
import type { SpdWorkerRequest, SpdWorkerResponse } from './spd-worker-protocol'

export type WorkerTask<T> = { promise: Promise<T>; cancel: () => void }

function createWorker() {
  return new Worker(new URL('../workers/spd.worker.ts', import.meta.url), {
    type: 'module',
  })
}

export function analyzeSeedInWorker(
  input: string,
  floors: number,
  trinket: string | null,
  challenges: readonly Challenge[],
  lorePages: boolean
): WorkerTask<SeedReport> {
  const worker = createWorker()
  let rejectTask: (reason: Error) => void = () => {}
  const promise = new Promise<SeedReport>((resolve, reject) => {
    rejectTask = reject
    worker.onmessage = (event: MessageEvent<SpdWorkerResponse>) => {
      if (event.data.type === 'analysis-complete') resolve(event.data.report)
      else if (event.data.type === 'error')
        reject(new Error(event.data.message))
      else return
      worker.terminate()
    }
    worker.onerror = (event) => {
      reject(new Error(event.message || 'Analysis worker failed.'))
      worker.terminate()
    }
    worker.postMessage({
      type: 'analyze',
      input,
      floors,
      trinket,
      challenges: [...challenges],
      lorePages,
    } satisfies SpdWorkerRequest)
  })
  return {
    promise,
    cancel: () => {
      worker.terminate()
      rejectTask(new Error('Cancelled'))
    },
  }
}

export function searchSeedsInWorker(
  request: SeedSearchRequest,
  onProgress: (result: SeedSearchResult) => void,
  onCandidate: (progress: {
    candidateNumber: number
    seed: number
    depth: number
  }) => void
): WorkerTask<SeedSearchResult> {
  const worker = createWorker()
  let rejectTask: (reason: Error) => void = () => {}
  const promise = new Promise<SeedSearchResult>((resolve, reject) => {
    rejectTask = reject
    worker.onmessage = (event: MessageEvent<SpdWorkerResponse>) => {
      if (event.data.type === 'search-candidate') onCandidate(event.data)
      else if (event.data.type === 'search-progress')
        onProgress(event.data.result)
      else if (event.data.type === 'search-complete') {
        resolve(event.data.result)
        worker.terminate()
      } else if (event.data.type === 'error') {
        reject(new Error(event.data.message))
        worker.terminate()
      }
    }
    worker.onerror = (event) => {
      reject(new Error(event.message || 'Search worker failed.'))
      worker.terminate()
    }
    worker.postMessage({ type: 'search', request } satisfies SpdWorkerRequest)
  })
  return {
    promise,
    cancel: () => {
      worker.terminate()
      rejectTask(new Error('Cancelled'))
    },
  }
}
