// One function per deploy value: brief it, infer or prompt for it, check it.
// Values are collected here and stored by main.ts once the token has vouched
// for the account and the domain.
import {
  type Check,
  configuredWorker,
  domainOwner,
  isAccountId,
  isHostname,
  isWorkerName,
  loginAccounts,
  probeToken,
  tokenUrls,
} from './cloudflare'
import type { Repo } from './github'
import {
  ask,
  askOr,
  askSecret,
  c,
  confirm,
  fail,
  heading,
  note,
  offerLink,
  ok,
  say,
  step,
  warn,
} from './tty'

export const ACCOUNT = 'CLOUDFLARE_ACCOUNT_ID'
export const TOKEN = 'CLOUDFLARE_API_TOKEN'
export const WORKER = 'WEB_WORKER_NAME'
export const DOMAIN = 'WEB_DOMAIN'
export const DEPLOY_URL = 'WEB_URL'

export type Context = {
  repo: Repo
  variables: Map<string, string>
  secrets: Set<string>
}

function brief(usedFor: string, needs: { name: string; why: string }[]) {
  say(`  ${c.dim('Used for:')} ${usedFor}`)
  if (!needs.length) return
  say(`  ${c.dim('Needs:')}`)
  for (const need of needs) say(`    • ${need.name} ${c.dim(`— ${need.why}`)}`)
}

const zoneOf = (domain: string) => domain.split('.').slice(-2).join('.')

export async function accountStep(ctx: Context) {
  heading(`1. 🏢 Cloudflare account ID ${c.dim(ACCOUNT)}`)
  brief('tells wrangler which account owns the Worker.', [
    {
      name: 'The account that holds the custom domain zone',
      why: "custom domains only attach to zones on the Worker's own account",
    },
  ])
  const current = ctx.variables.get(ACCOUNT) ?? ''
  if (current) note(`Currently ${current}.`)
  const accounts = await step('Reading your wrangler login', loginAccounts)
  if (accounts.length) {
    note('From `wrangler login`. Pick the account that now holds the zone.')
    for (const [i, a] of accounts.entries()) {
      const mark = a.id === current ? c.dim(' (current)') : ''
      say(`    ${i + 1}. ${a.name} ${c.dim(a.id)}${mark}`)
    }
  } else {
    note('Dashboard: Workers & Pages › the account ID is in the sidebar.')
  }
  while (true) {
    const answer = await askOr('Number or account ID:', current)
    const id = accounts[Number(answer) - 1]?.id ?? answer
    if (isAccountId(id)) return id.toLowerCase()
    fail('An account ID is 32 hexadecimal characters.')
  }
}

export async function workerStep(ctx: Context) {
  heading(`2. 🧩 Worker name ${c.dim(WORKER)}`)
  brief('the Worker `wrangler deploy --name` uploads the built SPA to.', [])
  const fallback = ctx.variables.get(WORKER) || (await configuredWorker())
  while (true) {
    const name = await askOr('Worker name:', fallback)
    if (isWorkerName(name)) return name
    fail('Lowercase letters, digits and dashes; at most 63 characters.')
  }
}

export async function domainStep(ctx: Context) {
  heading(`3. 🌐 Custom domain ${c.dim(DOMAIN)}`)
  brief('`wrangler deploy --domain` attaches this hostname to the Worker.', [
    {
      name: 'A zone on the account from step 1',
      why: 'Cloudflare creates the DNS record and certificate there',
    },
  ])
  note('Enter "-" to deploy on workers.dev only.')
  const current = ctx.variables.get(DOMAIN) ?? ''
  while (true) {
    const domain = (await askOr('Hostname:', current)).toLowerCase()
    if (domain === '-' || !domain) return ''
    if (isHostname(domain)) return domain
    fail('Enter a hostname such as spd.example.com, without https://.')
  }
}

export type TokenResult =
  | { outcome: 'kept' | 'skipped' }
  | { outcome: 'entered'; token: string; checks: Check[] }

export async function tokenStep(
  ctx: Context,
  accountId: string,
  worker: string,
  domain: string
): Promise<TokenResult> {
  heading(`4. 🔑 Cloudflare API token ${c.dim(TOKEN)}`)
  const zone = domain ? zoneOf(domain) : ''
  brief(
    `wrangler deploy in GitHub Actions: uploads the Worker${domain ? ` and attaches ${domain}` : ''}.`,
    [
      {
        name: 'Account › Workers Scripts › Edit',
        why: 'upload the Worker and its static assets',
      },
      ...(zone
        ? [
            {
              name: `Zone › Workers Routes › Edit, on ${zone}`,
              why: "attach the custom domain; every deploy reads the zone's routes",
            },
            {
              name: `Zone › Zone › Read, on ${zone}`,
              why: 'find the zone that owns the hostname',
            },
          ]
        : []),
    ]
  )
  const accountChanged = ctx.variables.get(ACCOUNT) !== accountId
  if (ctx.secrets.has(TOKEN)) {
    if (accountChanged) {
      warn('The account changed: the stored token belongs to the old one.')
    }
    if (
      !(await confirm(
        `${TOKEN} is already set. Enter a new one?`,
        accountChanged
      ))
    ) {
      note(
        "Kept. GitHub doesn't return secret values, so it can't be checked here."
      )
      return { outcome: 'kept' }
    }
  }

  let token = ''
  if (process.env.CLOUDFLARE_API_TOKEN) {
    warn('CLOUDFLARE_API_TOKEN is set in this shell. It is checked either way,')
    note('but use it only if it was minted for this deploy.')
    if (
      await confirm('Use CLOUDFLARE_API_TOKEN from the environment?', false)
    ) {
      token = process.env.CLOUDFLARE_API_TOKEN
    }
  }
  if (!token) {
    const urls = tokenUrls(`SPD Seed Analyzer deploy (${ctx.repo.slug})`)
    say('  Mint one with the access above pre-filled:')
    say(
      `  🏢 Account-owned ${c.dim("(preferred: outlives any one person's access)")}`
    )
    say(`     ${c.cyan(urls.account)}`)
    say(`  👤 User-owned ${c.dim('(if you are not an account admin)')}`)
    say(`     ${c.cyan(urls.user)}`)
    note(
      `${zone ? `In the form, limit Zone resources to ${zone}. ` : ''}wrangler's OAuth login is not a token.`
    )
    await offerLink('🏢 Account-owned', urls.account)
    token = await askSecret('API token (Enter to skip):')
  }
  if (!token) return { outcome: 'skipped' }

  while (true) {
    const checks = await step('Checking what the token can do', () =>
      probeToken(token, accountId, domain)
    )
    for (const check of checks)
      (check.ok ? ok : fail)(`${check.label}: ${check.detail}`)
    if (checks.every((check) => check.ok)) {
      const owner = domain && (await domainOwner(token, accountId, domain))
      if (owner && owner !== worker) {
        warn(
          `${domain} serves the ${owner} Worker; detach it before deploying.`
        )
      }
      return { outcome: 'entered', token, checks }
    }
    note(
      'A wrong account ID or domain fails the same way; re-run to change them.'
    )
    // A failed check still stores: the operator chose to keep this token.
    if (!(await confirm('Enter a different token?', true))) {
      return { outcome: 'entered', token, checks }
    }
    token = await askSecret('API token:')
  }
}

/** WEB_URL only overrides the deploy link; returns '' to clear it. */
export async function urlStep(ctx: Context, domain: string) {
  const current = ctx.variables.get(DEPLOY_URL)
  if (!current) return null
  heading(`5. 🔗 Deploy URL override ${c.dim(DEPLOY_URL)}`)
  brief(
    `the environment link on each deploy. Without it CI uses ${domain ? `https://${domain}` : 'the workers.dev URL'}.`,
    []
  )
  note(`Currently ${current}.`)
  if (await confirm('Keep it?', current === `https://${domain}`)) return current
  return await ask('New URL (Enter to clear):')
}
