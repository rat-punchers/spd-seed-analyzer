// GitHub Actions environment variables and secrets through the gh CLI.
// Values go in on stdin, never argv.

export type Repo = { owner: string; name: string; slug: string }

/** The environment `.github/workflows/ci.yaml` deploys from. */
export const ENVIRONMENT = 'prod'

async function run(cmd: string[], stdin?: string) {
  const proc = Bun.spawn(cmd, {
    stdin: stdin === undefined ? 'ignore' : new Blob([stdin]),
    stdout: 'pipe',
    stderr: 'pipe',
  })
  const [out, err, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ])
  return { out: out.trim(), err: err.trim(), code }
}

/** The repo that deploys: origin, not upstream. */
export async function originRepo(): Promise<Repo | null> {
  const { out, code } = await run(['git', 'remote', 'get-url', 'origin'])
  if (code !== 0) return null
  const [, owner, name] =
    out.match(/github\.com[:/]([^/]+)\/(.+?)(\.git)?$/) ?? []
  if (!owner || !name) return null
  return { owner, name, slug: `${owner}/${name}` }
}

export async function ghLogin(): Promise<string | null> {
  if (!Bun.which('gh')) return null
  const { out, code } = await run(['gh', 'api', 'user', '--jq', '.login'])
  return code === 0 ? out : null
}

export type EnvironmentAccess = 'ok' | 'missing' | 'denied'

/**
 * Reading the environment's secrets public key needs the same access as
 * writing its secrets. A 404 means the environment does not exist yet.
 */
export async function environmentAccess(
  repo: Repo
): Promise<EnvironmentAccess> {
  const { code, err } = await run([
    'gh',
    'api',
    `repos/${repo.slug}/environments/${ENVIRONMENT}/secrets/public-key`,
  ])
  if (code === 0) return 'ok'
  return /HTTP 404/.test(err) ? 'missing' : 'denied'
}

/** Creating the environment object needs repository Administration: write. */
export async function createEnvironment(repo: Repo) {
  const { code, err } = await run([
    'gh',
    'api',
    '--method',
    'PUT',
    `repos/${repo.slug}/environments/${ENVIRONMENT}`,
  ])
  if (code !== 0) throw new Error(err || `creating ${ENVIRONMENT} failed`)
}

const scope = (repo: Repo) => ['--env', ENVIRONMENT, '--repo', repo.slug]

export async function existingVariables(
  repo: Repo
): Promise<Map<string, string>> {
  const { out, code } = await run([
    'gh',
    'variable',
    'list',
    ...scope(repo),
    '--json',
    'name,value',
  ])
  if (code !== 0) return new Map()
  const list = JSON.parse(out) as { name: string; value: string }[]
  return new Map(list.map((v) => [v.name, v.value]))
}

export async function existingSecrets(repo: Repo): Promise<Set<string>> {
  const { out, code } = await run([
    'gh',
    'secret',
    'list',
    ...scope(repo),
    '--json',
    'name',
  ])
  if (code !== 0) return new Set()
  return new Set((JSON.parse(out) as { name: string }[]).map((s) => s.name))
}

export async function setVariable(repo: Repo, name: string, value: string) {
  const { code, err } = await run(
    ['gh', 'variable', 'set', name, ...scope(repo)],
    value
  )
  if (code !== 0) throw new Error(err || `gh variable set ${name} failed`)
}

export async function deleteVariable(repo: Repo, name: string) {
  const { code, err } = await run([
    'gh',
    'variable',
    'delete',
    name,
    ...scope(repo),
  ])
  if (code !== 0) throw new Error(err || `gh variable delete ${name} failed`)
}

export async function setSecret(repo: Repo, name: string, value: string) {
  const { code, err } = await run(
    ['gh', 'secret', 'set', name, ...scope(repo)],
    value
  )
  if (code !== 0) throw new Error(err || `gh secret set ${name} failed`)
}
