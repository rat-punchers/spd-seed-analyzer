// Walks through the `prod` GitHub environment values that deploy web/ to
// Cloudflare: brief what each is for, infer what the machine already knows,
// prompt for the rest, check it against Cloudflare, then store it.
// Run: bun run credentials. --dry-run does every read and check but stores nothing.

import {
  createEnvironment,
  deleteVariable,
  ENVIRONMENT,
  environmentAccess,
  existingSecrets,
  existingVariables,
  ghLogin,
  originRepo,
  setSecret,
  setVariable,
} from './github'
import {
  ACCOUNT,
  accountStep,
  type Context,
  DEPLOY_URL,
  DOMAIN,
  domainStep,
  TOKEN,
  type TokenResult,
  tokenStep,
  urlStep,
  WORKER,
  workerStep,
} from './steps'
import {
  c,
  confirm,
  fail,
  heading,
  installCleanup,
  note,
  ok,
  restoreTerminal,
  say,
  step,
} from './tty'

const dryRun = process.argv.includes('--dry-run')

type Outcome = 'stored' | 'unverified' | 'kept' | 'cleared' | 'skipped'
const outcomes: [string, Outcome][] = []

async function save(name: string, write: () => Promise<void>, verb: string) {
  if (dryRun) ok(`Would ${verb} ${name}`)
  else {
    await step(`Saving ${name}`, write)
    ok(`${verb[0]?.toUpperCase()}${verb.slice(1)}d ${name}`)
  }
}

async function storeVariable(
  ctx: Context,
  name: string,
  value: string,
  verified: boolean
) {
  const current = ctx.variables.get(name) ?? ''
  if (value === current) {
    outcomes.push([name, value ? 'kept' : 'skipped'])
    return
  }
  if (!value) {
    await save(name, () => deleteVariable(ctx.repo, name), 'clear')
    outcomes.push([name, 'cleared'])
    return
  }
  await save(name, () => setVariable(ctx.repo, name, value), 'store')
  outcomes.push([name, verified ? 'stored' : 'unverified'])
}

/** Whether the token probe passed the check whose label starts with `prefix`. */
function vouched(result: TokenResult, prefix: string) {
  if (result.outcome !== 'entered') return false
  return result.checks.some((ch) => ch.label.startsWith(prefix) && ch.ok)
}

async function preflight(): Promise<Context | null> {
  heading('Preflight')
  const repo = await step('Reading the git remote', originRepo)
  if (!repo) {
    fail('origin is not a GitHub repository.')
    return null
  }
  const login = await step('Checking the GitHub CLI', ghLogin)
  if (!login) {
    fail(
      'The GitHub CLI is not signed in. Run `gh auth login`, then try again.'
    )
    return null
  }
  ok(`gh is signed in as ${login}`)
  let access = await step(`Checking the ${ENVIRONMENT} environment`, () =>
    environmentAccess(repo)
  )
  if (access === 'missing') {
    note(
      `${repo.slug} has no ${ENVIRONMENT} environment; the deploy job needs one.`
    )
    if (!dryRun && (await confirm(`Create ${ENVIRONMENT}?`, true))) {
      try {
        await step(`Creating ${ENVIRONMENT}`, () => createEnvironment(repo))
        access = 'ok'
      } catch {
        fail('Creating an environment needs repository Administration: write.')
      }
    }
  }
  if (access !== 'ok') {
    fail(
      `${login} can't manage ${ENVIRONMENT} secrets on ${repo.slug}; a repo admin has to run this.`
    )
    return null
  }
  ok(`Can manage ${ENVIRONMENT} variables and secrets on ${repo.slug}`)
  const [variables, secrets] = await step('Listing current values', () =>
    Promise.all([existingVariables(repo), existingSecrets(repo)])
  )
  const set = [ACCOUNT, WORKER, DOMAIN, DEPLOY_URL, TOKEN].filter(
    (name) => variables.has(name) || secrets.has(name)
  )
  note(
    set.length ? `Already set: ${set.join(', ')}` : 'No deploy values set yet.'
  )
  return { repo, variables, secrets }
}

async function main() {
  if (!process.stdin.isTTY) {
    say('Run this in a terminal: it asks for a secret.')
    process.exitCode = 1
    return
  }
  installCleanup()
  say(
    c.bold(
      `🔐 SPD Seed Analyzer deploy credentials${dryRun ? ' (dry run: nothing is stored)' : ''}`
    )
  )
  say(
    `Sets the ${ENVIRONMENT} environment values the deploy job in .github/workflows/ci.yaml reads.`
  )
  const ctx = await preflight()
  if (!ctx) {
    process.exitCode = 1
    return
  }

  heading('Required')
  const accountId = await accountStep(ctx)
  const worker = await workerStep(ctx)
  const domain = await domainStep(ctx)
  const token = await tokenStep(ctx, accountId, worker, domain)

  heading('Optional')
  const url = await urlStep(ctx, domain)
  if (url === null)
    note(`${DEPLOY_URL} is unset; CI derives the link from ${DOMAIN}.`)

  heading('Saving')
  await storeVariable(
    ctx,
    ACCOUNT,
    accountId,
    vouched(token, 'Workers Scripts')
  )
  await storeVariable(ctx, WORKER, worker, true)
  await storeVariable(ctx, DOMAIN, domain, !domain || vouched(token, 'Zone'))
  if (url !== null) await storeVariable(ctx, DEPLOY_URL, url, true)
  if (token.outcome === 'entered') {
    const verified = token.checks.every((ch) => ch.ok)
    await save(TOKEN, () => setSecret(ctx.repo, TOKEN, token.token), 'store')
    outcomes.push([TOKEN, verified ? 'stored' : 'unverified'])
  } else outcomes.push([TOKEN, token.outcome])

  heading('Summary')
  const label: Record<Outcome, string> = {
    stored: c.green(dryRun ? '✓ would store' : '✓ stored'),
    unverified: c.yellow(
      dryRun ? '! would store, unverified' : '! stored, unverified'
    ),
    kept: c.dim('· kept'),
    cleared: c.dim(dryRun ? '· would clear' : '· cleared'),
    skipped: c.yellow('· skipped'),
  }
  for (const [name, outcome] of outcomes) say(`  ${label[outcome]}  ${name}`)
  if (
    outcomes.some(
      ([n, o]) => o === 'unverified' || (n === TOKEN && o === 'skipped')
    )
  ) {
    note('Deploys fail until the token is set and passes its checks.')
  }
}

try {
  await main()
} finally {
  restoreTerminal()
}
