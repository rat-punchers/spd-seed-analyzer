import {
  type ChallengeChoice,
  ChallengePicker,
} from '@/components/seed/ChallengePicker'
import { IdentitiesPanel } from '@/components/seed/IdentitiesPanel'
import {
  type LoreChoice,
  LorePagesPicker,
} from '@/components/seed/LorePagesPicker'
import { SeedTitle } from '@/components/seed/SeedTitle'
import type { SeedReport } from '@/lib/spd-wasm'

/** Top of a report: the seed, the run settings that change its floors, and its identities. */
export function RunHeader({
  report,
  loreChoice,
  challengeChoice,
}: {
  report: SeedReport
  loreChoice: LoreChoice
  challengeChoice: ChallengeChoice
}) {
  return (
    <header data-run-header className="flex flex-col gap-6 pb-6">
      <SeedTitle seed={report.seed} />
      <LorePagesPicker choice={loreChoice} />
      <ChallengePicker
        choice={challengeChoice}
        identities={report.identities}
      />
      <IdentitiesPanel identities={report.identities} />
    </header>
  )
}
