// Cloudflare: the Worker that deploys, and what a deploy token must be able to do.
import { join } from 'node:path'

const root = join(import.meta.dir, '../..')
const API = 'https://api.cloudflare.com/client/v4'

/** The Worker name in web/wrangler.toml; CI overrides it with WEB_WORKER_NAME. */
export async function configuredWorker(): Promise<string> {
  const text = await Bun.file(join(root, 'web/wrangler.toml')).text()
  return (Bun.TOML.parse(text) as { name: string }).name
}

export type Account = { id: string; name: string }

export const isAccountId = (value: string) => /^[0-9a-f]{32}$/i.test(value)
export const isWorkerName = (value: string) =>
  /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$/.test(value)
export const isHostname = (value: string) =>
  /^([a-z0-9]([a-z0-9-]*[a-z0-9])?\.)+[a-z]{2,}$/i.test(value)

/**
 * Accounts the local wrangler login can see. CLOUDFLARE_API_TOKEN would win
 * over the login, so it is stripped from wrangler's environment.
 */
export async function loginAccounts(): Promise<Account[]> {
  const wrangler = Bun.which('wrangler', {
    PATH: `${join(root, 'node_modules/.bin')}:${join(root, 'web/node_modules/.bin')}:${process.env.PATH}`,
  })
  if (!wrangler) return []
  const env = { ...process.env }
  delete env.CLOUDFLARE_API_TOKEN
  delete env.CLOUDFLARE_API_KEY
  delete env.CLOUDFLARE_EMAIL
  delete env.CLOUDFLARE_ACCOUNT_ID
  const proc = Bun.spawn([wrangler, 'whoami', '--json'], {
    env,
    stdout: 'pipe',
    stderr: 'ignore',
  })
  const out = await new Response(proc.stdout).text()
  if ((await proc.exited) !== 0) return []
  try {
    const who = JSON.parse(out) as { accounts?: Account[] }
    return (who.accounts ?? []).map(({ id, name }) => ({ id, name }))
  } catch {
    return []
  }
}

const permissions = [
  { key: 'workers_scripts', type: 'edit' },
  { key: 'workers_routes', type: 'edit' },
  { key: 'zone', type: 'read' },
]

/** Dashboard template URLs that pre-fill the token form. */
export function tokenUrls(name: string) {
  const keys = encodeURIComponent(JSON.stringify(permissions))
  const label = encodeURIComponent(name)
  return {
    account: `https://dash.cloudflare.com/?to=/:account/api-tokens&permissionGroupKeys=${keys}&name=${label}`,
    user: `https://dash.cloudflare.com/profile/api-tokens?permissionGroupKeys=${keys}&accountId=*&zoneId=all&name=${label}`,
  }
}

/** Cloudflare error codes, turned into what to do about them. */
function explain(errors: { code: number; message: string }[]) {
  const codes = new Set(errors.map((e) => e.code))
  if (codes.has(6003) || codes.has(6111)) {
    return "Cloudflare didn't read that as a token; paste the whole token value"
  }
  if (codes.has(1000) || codes.has(9106) || codes.has(9109))
    return 'Cloudflare rejected the token as invalid'
  if (codes.has(10000)) {
    return 'the token lacks this permission, or the account ID is wrong'
  }
  return errors.map((e) => `${e.code} ${e.message}`).join('; ')
}

type ApiResult<T> = {
  ok: boolean
  status: number
  result: T | null
  error: string
}

async function api<T>(token: string, route: string): Promise<ApiResult<T>> {
  try {
    const res = await fetch(`${API}${route}`, {
      headers: { Authorization: `Bearer ${token}` },
    })
    const body = (await res.json()) as {
      success?: boolean
      result?: T
      errors?: { code: number; message: string }[]
    }
    return {
      ok: res.ok && body.success !== false,
      status: res.status,
      result: body.result ?? null,
      error: explain(body.errors ?? []),
    }
  } catch (error) {
    return { ok: false, status: 0, result: null, error: String(error) }
  }
}

export type Check = { label: string; ok: boolean; detail: string }

type Zone = { id: string; name: string; account: { id: string } }

/** The zone that owns `hostname`: the longest suffix the token can see. */
async function findZone(token: string, hostname: string) {
  const labels = hostname.split('.')
  for (let i = 0; i < labels.length - 1; i++) {
    const name = labels.slice(i).join('.')
    const res = await api<Zone[]>(token, `/zones?name=${name}`)
    if (!res.ok) return { zone: null, error: res.error }
    if (res.result?.[0]) return { zone: res.result[0], error: '' }
  }
  return { zone: null, error: '' }
}

/**
 * Probes capability, not ownership: each check reads with the permission the
 * deploy will use. Token verify endpoints only prove the token exists, and
 * `/user/tokens/verify` rejects account-owned tokens outright.
 */
export async function probeToken(
  token: string,
  accountId: string,
  domain: string
): Promise<Check[]> {
  const checks: Check[] = []
  const scripts = await api(token, `/accounts/${accountId}/workers/scripts`)
  checks.push({
    label: 'Workers Scripts on the account',
    ok: scripts.ok,
    detail: scripts.ok
      ? 'can read and deploy Workers'
      : scripts.error || `HTTP ${scripts.status}`,
  })
  if (!domain) return checks

  const { zone, error } = await findZone(token, domain)
  const sameAccount = zone?.account.id === accountId
  checks.push({
    label: `Zone that owns ${domain}`,
    ok: Boolean(zone && sameAccount),
    detail: !zone
      ? error || 'no zone visible to this token; grant Zone › Zone › Read on it'
      : sameAccount
        ? `${zone.name}, on this account`
        : `${zone.name} belongs to another account; custom domains need the same account`,
  })

  if (zone) {
    const routes = await api(token, `/zones/${zone.id}/workers/routes`)
    checks.push({
      label: `Workers Routes on ${zone.name}`,
      ok: routes.ok,
      detail: routes.ok
        ? 'deploys can attach the custom domain'
        : routes.error || `HTTP ${routes.status}`,
    })
  }
  return checks
}

/** Which Worker already serves `domain` on the account, if any. */
export async function domainOwner(
  token: string,
  accountId: string,
  domain: string
) {
  const res = await api<{ service: string }[]>(
    token,
    `/accounts/${accountId}/workers/domains?hostname=${domain}`
  )
  return res.ok ? (res.result?.[0]?.service ?? null) : null
}
