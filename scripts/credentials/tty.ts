// Terminal helpers for guided scripts: color, spinner, prompts.
// Colors and the spinner only run on a real terminal outside CI.

const interactive = Boolean(process.stdout.isTTY) && !process.env.CI
const colored = interactive && !process.env.NO_COLOR

const paint = (code: string) => (text: string) =>
  colored ? `\x1b[${code}m${text}\x1b[0m` : text

export const c = {
  bold: paint('1'),
  dim: paint('2'),
  green: paint('32'),
  yellow: paint('33'),
  red: paint('31'),
  cyan: paint('36'),
}

export function say(text = '') {
  process.stdout.write(`${text}\n`)
}

export function heading(text: string) {
  say()
  say(c.bold(text))
}

let spinning: ReturnType<typeof setInterval> | null = null

/** Runs `task` behind a spinner and prints ✓ or ✗ with `label`. */
export async function step<T>(
  label: string,
  task: () => Promise<T>
): Promise<T> {
  const frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
  let frame = 0
  if (interactive) {
    process.stdout.write('\x1b[?25l')
    spinning = setInterval(() => {
      process.stdout.write(
        `\r${c.cyan(frames[frame++ % frames.length] ?? '')} ${label}`
      )
    }, 80)
  }
  try {
    return await task()
  } finally {
    if (spinning) clearInterval(spinning)
    spinning = null
    if (interactive) process.stdout.write('\r\x1b[2K\x1b[?25h')
  }
}

export const ok = (text: string) => say(`  ${c.green('✓')} ${text}`)
export const warn = (text: string) => say(`  ${c.yellow('!')} ${text}`)
export const fail = (text: string) => say(`  ${c.red('✗')} ${text}`)
export const note = (text: string) => say(`  ${c.dim(text)}`)

/** Reads one line in cooked mode; the answer echoes, for public values. */
export function ask(question: string): Promise<string> {
  process.stdout.write(`  ${question} `)
  return new Promise((resolve) => {
    let buffer = ''
    const onData = (chunk: Buffer) => {
      buffer += chunk.toString('utf8')
      const end = buffer.indexOf('\n')
      if (end === -1) return
      process.stdin.off('data', onData)
      process.stdin.pause()
      resolve(buffer.slice(0, end).replace(/\r$/, '').trim())
    }
    process.stdin.on('data', onData)
    process.stdin.resume()
  })
}

/** Like `ask`, with `fallback` shown and returned on Enter. */
export async function askOr(question: string, fallback: string) {
  const hint = fallback ? ` ${c.dim(`[${fallback}]`)}` : ''
  return (await ask(`${question}${hint}`)) || fallback
}

export async function confirm(question: string, fallback: boolean) {
  const hint = fallback ? '[Y/n]' : '[y/N]'
  const answer = (await ask(`${question} ${c.dim(hint)}`)).toLowerCase()
  if (!answer) return fallback
  return answer === 'y' || answer === 'yes'
}

/**
 * Reads a secret in raw mode, echoing one * per character (paste included)
 * so the prompt never looks hung. Backspace erases a star; Ctrl-C quits.
 */
export function askSecret(question: string): Promise<string> {
  // Raw mode first: typing or pasting before it would echo in the clear.
  process.stdin.setRawMode(true)
  process.stdout.write(`  ${question} `)
  return new Promise((resolve) => {
    let value = ''
    const finish = () => {
      process.stdin.off('data', onData)
      process.stdin.setRawMode(false)
      process.stdin.pause()
      process.stdout.write('\n')
      resolve(value.trim())
    }
    const onData = (chunk: Buffer) => {
      for (const char of chunk.toString('utf8')) {
        if (char === '\r' || char === '\n') return finish()
        if (char === '\u0003') {
          restoreTerminal()
          process.stdout.write('\n')
          process.exit(130)
        }
        if (char === '\u007f' || char === '\b') {
          if (value) {
            value = value.slice(0, -1)
            process.stdout.write('\b \b')
          }
        } else if (char >= ' ') {
          value += char
          process.stdout.write('*')
        }
      }
    }
    process.stdin.on('data', onData)
    process.stdin.resume()
  })
}

/** Shows the cursor again however the script ends, Ctrl-C included. */
export function installCleanup() {
  process.on('exit', () => {
    if (interactive) process.stdout.write('\x1b[?25h')
  })
  process.on('SIGINT', () => {
    restoreTerminal()
    process.stdout.write('\n')
    process.exit(130)
  })
}

/** Cooked mode, cursor back, stdin released so the process can end. */
export function restoreTerminal() {
  if (spinning) clearInterval(spinning)
  if (process.stdin.isTTY) process.stdin.setRawMode(false)
  process.stdin.pause()
  // Only TTY streams have unref; a redirected stdin doesn't hold the process.
  process.stdin.unref?.()
  if (interactive) process.stdout.write('\x1b[?25h')
}

/** Asks before opening a labeled URL; never opens one unasked. */
export async function offerLink(label: string, url: string) {
  if (await confirm(`Open ${label} in the browser?`, false)) {
    const opener = process.platform === 'darwin' ? 'open' : 'xdg-open'
    Bun.spawn([opener, url], { stdout: 'ignore', stderr: 'ignore' })
  }
}
