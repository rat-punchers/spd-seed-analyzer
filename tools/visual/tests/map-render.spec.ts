import { expect, type Locator, type Page, test } from '@playwright/test'
import { GAME_IDS } from './game-ids'
import {
  MAP_RENDER_FIXTURES,
  type MapRenderLevelIdentity,
  mapRenderBranchTitle,
  mapRenderSnapshotName,
  mapRenderTestName,
} from './map-render-fixtures'

const APP_STORAGE = {
  mode: 'spd-analyzer-mode',
  testInitialized: 'spd-analyzer-visual-test-initialized',
  theme: 'spd-analyzer-theme',
} as const

const GENERIC_MAP_SEED = 'VIS-UAL-MAP'
const GENERIC_MAP_FLOOR = 4

type BrowserErrors = {
  console: string[]
  page: string[]
}

async function installSyntheticMapReport(page: Page) {
  await page.addInitScript(({ ItemId, RoomId, Enchantment }) => {
    const width = 20
    const height = 30
    const tiles = Array(width * height).fill(1)
    for (let y = 4; y < height - 4; y++) {
      for (let x = 4; x < width - 4; x++) tiles[y * width + x] = 2
    }
    const floors = Array.from({ length: 4 }, (_, index) => ({
      depth: index + 1,
      feeling: 'none',
      builder: 'loop',
      rooms: [RoomId.StandardRoom],
      guaranteed_appearances:
        index === 2
          ? [
              {
                kind: 'alchemy_pot',
                source: RoomId.LaboratoryRoom,
              },
            ]
          : [],
      items: [],
      quests: [],
      map:
        index === 3
          ? {
              width,
              height,
              tileset: 'sewers',
              tiles,
              tile_variance: [],
              discoverable: Array(width * height).fill(1),
              markers: [],
              heaps: [],
              mobs: [],
              transitions: [
                { cell: 5 * width + 5, type: 'REGULAR_ENTRANCE' },
                {
                  cell: (height - 6) * width + (width - 6),
                  type: 'REGULAR_EXIT',
                },
              ],
              traps: [],
              plants: [],
              blobs: [],
              custom_tiles: [
                {
                  class: 'CenterPieceVisuals',
                  texture: 'halls_special',
                  x: 8,
                  y: 8,
                  width: 1,
                  height: 1,
                  static_data: [8],
                },
              ],
              custom_walls: [
                {
                  class: 'CenterPieceWalls',
                  texture: 'halls_special',
                  x: 8,
                  y: 9,
                  width: 1,
                  height: 1,
                  static_data: [-1],
                },
              ],
            }
          : null,
    }))
    ;(floors[3].items as unknown[]).push({
      variants: [
        {
          quantity: 1,
          item: ItemId.ScrollOfTransmutation,
          category: 'scroll',
          level_range: { min: 0, max: 3 },
          cursed: true,
          enchantment: {
            type: Enchantment.Blazing,
            conditions: [
              {
                type: 'trinket',
                events: [
                  {
                    before_depth: 4,
                    kind: 'acquired',
                    trinket: 'ParchmentScrap',
                  },
                  { before_depth: 4, kind: 'upgraded' },
                ],
              },
            ],
          },
          prediction: 'exact',
          conditions: [
            {
              type: 'state',
              state_id: 'synthetic_upgrade_route',
            },
          ],
          spawn_conditions: [
            {
              all_of: [
                {
                  type: 'challenge',
                  challenge: 'forbidden_runes',
                  enabled: false,
                },
              ],
            },
          ],
        },
      ],
    })
    ;(floors[3].items as unknown[]).push({
      variants: [
        {
          quantity: 1,
          item: ItemId.TrinketCatalyst,
          category: 'trinket',
          candidates: [
            ItemId.MossyClump,
            ItemId.MimicTooth,
            ItemId.RatSkull,
            ItemId.SaltCube,
          ],
          prediction: 'exact',
        },
      ],
    })
    ;(floors[3].items as unknown[]).push({
      variants: [
        {
          quantity: 1,
          item: ItemId.PotionOfHealing,
          category: 'potion',
          prediction: 'exact',
        },
      ],
    })
    ;(floors[3].items as unknown[]).push({
      source: 'SacrificeRoom',
      variants: [
        {
          label: 'weapon reward',
          quantity: 1,
          category: 'weapon',
          tier_range: { min: 2, max: 5 },
          level_range: { min: 0, max: 3 },
          cursed: true,
          prediction: 'constrained',
        },
        {
          quantity: 1,
          item: ItemId.Sickle,
          category: 'weapon',
          level: 2,
          cursed: true,
          enchantment: { type: Enchantment.Friendly, conditions: [] },
          prediction: 'variable',
        },
      ],
    })
    const report = {
      seed: {
        input: 'VIS-UAL-MAP',
        numeric: 0,
        code: null,
        formatted: 'VIS-UAL-MAP',
      },
      spd_version: 'v4.0.2',
      spd_commit: '57a4e06a4',
      floors_requested: 4,
      identities: { potions: [], scrolls: [], rings: [] },
      trinket_selection: {
        catalyst_depth: 2,
        first_alchemy_pot_depth: 3,
        first_alchemy_pot_is_secret: false,
        selection_depth: 3,
        first_effective_depth: 4,
        catalyst_options: [
          ItemId.MossyClump,
          ItemId.MimicTooth,
          ItemId.RatSkull,
          ItemId.SaltCube,
        ],
        catalyst_option_effects: ['modeled', 'modeled', 'modeled', 'none'],
        transmutation_sequence: [
          ItemId.TrapMechanism,
          ItemId.ParchmentScrap,
          ItemId.PetrifiedSeed,
          ItemId.ExoticCrystals,
          ItemId.DimensionalSundial,
          ItemId.ThirteenLeafClover,
          ItemId.WondrousResin,
          ItemId.EyeOfNewt,
          ItemId.VialOfBlood,
          ItemId.ShardOfOblivion,
          ItemId.ChaoticCenser,
          ItemId.FerretTuft,
          ItemId.CrackedSpyglass,
        ],
      },
      floors,
      analysis_notes: ['Synthetic conditional analysis remains partial.'],
      status: 'partial',
      message: 'Synthetic Playwright renderer fixture.',
    }
    class VisualFixtureWorker {
      onmessage: ((event: MessageEvent) => void) | null = null
      onerror: ((event: ErrorEvent) => void) | null = null
      postMessage(message: { type?: string }) {
        if (message.type !== 'analyze') return
        setTimeout(() => {
          this.onmessage?.(
            new MessageEvent('message', {
              data: { type: 'analysis-complete', report },
            })
          )
        }, 0)
      }
      terminate() {}
      addEventListener() {}
      removeEventListener() {}
      dispatchEvent() {
        return true
      }
    }
    Object.assign(window, { Worker: VisualFixtureWorker })
  }, GAME_IDS)
}

async function openAnalyzer(
  page: Page,
  seed: string,
  reducedMotion: 'reduce' | 'no-preference' = 'reduce'
): Promise<BrowserErrors> {
  const errors: BrowserErrors = { console: [], page: [] }
  page.on('console', (message) => {
    if (message.type() === 'error') errors.console.push(message.text())
  })
  page.on('pageerror', (error) => errors.page.push(error.message))

  await page.emulateMedia({ colorScheme: 'light', reducedMotion })
  await page.addInitScript((storage) => {
    if (sessionStorage.getItem(storage.testInitialized) !== 'true') {
      localStorage.clear()
      localStorage.setItem(storage.mode, 'analyze')
      localStorage.setItem(storage.theme, 'light')
      sessionStorage.setItem(storage.testInitialized, 'true')
    }
  }, APP_STORAGE)

  await page.goto('/')
  await expect(page).toHaveTitle('SPD Seed Analyzer')
  await expect(
    page.getByRole('heading', { name: 'No seeds analyzed yet' })
  ).toBeVisible()

  await page.getByLabel('Enter your seed').fill(seed)
  await page.getByRole('button', { name: 'Analyze', exact: true }).click()
  await expect(page.getByRole('tab', { name: seed, exact: true })).toBeVisible()
  await expect(
    page.getByRole('heading', { name: 'Floor 1', exact: true })
  ).toBeVisible({
    timeout: 60_000,
  })

  return errors
}

async function waitForCanvasPaint(canvas: Locator) {
  await expect(canvas).toBeVisible()
  await expect
    .poll(async () =>
      canvas.evaluate((node) => {
        const mapCanvas = node as HTMLCanvasElement
        const context = mapCanvas.getContext('2d')
        if (!context || mapCanvas.width === 0 || mapCanvas.height === 0) {
          return false
        }

        const pixels = context.getImageData(
          0,
          0,
          mapCanvas.width,
          mapCanvas.height
        ).data
        for (let alpha = 3; alpha < pixels.length; alpha += 4) {
          if (pixels[alpha] !== 0) return true
        }
        return false
      })
    )
    .toBe(true)
}

async function expectSyntheticCustomTile(canvas: Locator) {
  const matchesSource = await canvas.evaluate(async (node) => {
    const mapCanvas = node as HTMLCanvasElement
    const source = new Image()
    source.src = '/assets/environment/custom_tiles/halls_special.png'
    await source.decode()

    const sourceCanvas = document.createElement('canvas')
    sourceCanvas.width = 16
    sourceCanvas.height = 16
    const sourceContext = sourceCanvas.getContext('2d')
    const mapContext = mapCanvas.getContext('2d')
    if (!sourceContext || !mapContext) return false
    // Synthetic layer starts at atlas visual 8 and map cell (8, 8).
    sourceContext.drawImage(source, 0, 16, 16, 16, 0, 0, 16, 16)
    const sourcePixels = sourceContext.getImageData(0, 0, 16, 16).data
    const sourcePixel = Array.from(
      { length: 16 * 16 },
      (_, pixel) => pixel
    ).find((pixel) => sourcePixels[pixel * 4 + 3] === 255)
    if (sourcePixel == null) return false

    const scale = mapCanvas.width / (20 * 16)
    const sourceX = sourcePixel % 16
    const sourceY = Math.floor(sourcePixel / 16)
    const mapPixel = mapContext.getImageData(
      (8 * 16 + sourceX) * scale,
      (8 * 16 + sourceY) * scale,
      1,
      1
    ).data
    return [0, 1, 2, 3].every(
      (channel) => mapPixel[channel] === sourcePixels[sourcePixel * 4 + channel]
    )
  })
  expect(matchesSource).toBe(true)
}

async function snapshotCanvas(canvas: Locator, snapshot: string) {
  const dataUrl = await canvas.evaluate(async (node) => {
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
    })
    return (node as HTMLCanvasElement).toDataURL('image/png')
  })
  const png = Buffer.from(dataUrl.slice(dataUrl.indexOf(',') + 1), 'base64')
  expect(png).toMatchSnapshot(snapshot, {
    maxDiffPixels: 0,
    threshold: 0,
  })
}

async function captureLevel(
  page: Page,
  level: MapRenderLevelIdentity,
  snapshot: string
) {
  const floor = level.depth
  await page
    .getByRole('heading', { name: `Floor ${floor}`, exact: true })
    .scrollIntoViewIfNeeded()

  const mapLabel =
    level.kind === 'main'
      ? `floor ${floor} map`
      : `${mapRenderBranchTitle(level.objective)} floor ${floor}, branch ${level.branch} map`
  const dialogTitle =
    level.kind === 'main'
      ? `Floor ${floor}`
      : `${mapRenderBranchTitle(level.objective)} - Floor ${floor}, branch ${level.branch}`

  if (level.kind === 'branch') {
    const questType =
      level.objective === 'Vault' ? 'ambitious_imp' : 'troll_blacksmith'
    const questName =
      level.objective === 'Vault' ? 'Ambitious Imp' : 'Troll Blacksmith'
    const questCard = page.locator(`[data-quest-type="${questType}"]`)
    await expect(
      questCard.getByRole('heading', { name: questName, exact: true })
    ).toBeVisible()
    await expect(questCard).not.toContainText('Objective:')
    await expect(questCard).not.toContainText('Access:')
    if (level.objective === 'Crystal' || level.objective === 'Gnoll') {
      await expect(
        questCard.getByRole('img', { name: level.objective, exact: true })
      ).toBeVisible()
    }
    if (level.objective === 'Vault') {
      await expect(
        questCard.getByRole('img', { name: 'Imp', exact: true })
      ).toBeVisible()
      await expect(questCard).not.toContainText('Spawned on depth')
    }
  }

  await page
    .getByRole('button', { name: `Expand ${mapLabel}`, exact: true })
    .click()

  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible()
  await expect(
    dialog.getByRole('heading', { name: dialogTitle, exact: true })
  ).toBeVisible()

  const canvas = dialog.getByRole('img', {
    name: /Shattered Pixel Dungeon floor map/,
  })
  await expect(canvas).toHaveAttribute('data-water-animation', 'paused')
  await waitForCanvasPaint(canvas)
  await expect(dialog.getByRole('button', { name: /^Show items/ })).toHaveCount(
    0
  )
  await expect(
    dialog.getByRole('button', { name: /^Show known mobs/ })
  ).toHaveCount(0)

  await snapshotCanvas(canvas, snapshot)
}

for (const fixture of MAP_RENDER_FIXTURES) {
  test(mapRenderTestName(fixture), async ({ page }) => {
    const browserErrors = await openAnalyzer(page, fixture.seed)
    await captureLevel(page, fixture.level, mapRenderSnapshotName(fixture))

    expect(browserErrors.console, 'browser console errors').toEqual([])
    expect(browserErrors.page, 'uncaught page errors').toEqual([])
  })
}

test('mobile map dialog fills the viewport and supports pan, zoom, and close', async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 })
  await installSyntheticMapReport(page)
  const browserErrors = await openAnalyzer(page, GENERIC_MAP_SEED)
  await page
    .getByRole('button', { name: `Expand floor ${GENERIC_MAP_FLOOR} map` })
    .first()
    .click()

  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible()
  await expect
    .poll(() => dialog.boundingBox())
    .toEqual({ x: 0, y: 0, width: 390, height: 844 })

  const canvas = dialog.getByRole('img', {
    name: /Shattered Pixel Dungeon floor map/,
  })
  await waitForCanvasPaint(canvas)
  await expectSyntheticCustomTile(canvas)
  const stage = dialog.getByTestId('map-scroll-container')
  await expect(stage).toHaveAttribute('data-zoom')
  const maxZoom = Number(await stage.getAttribute('data-max-zoom'))
  const minZoom = Number(await stage.getAttribute('data-min-zoom'))
  await stage.hover()
  for (let step = 0; step < 24; step++) await page.mouse.wheel(0, -240)
  await expect
    .poll(async () => Number(await stage.getAttribute('data-zoom')))
    .toBeGreaterThan(maxZoom - 0.01)
  const zoomedPan = Number(await stage.getAttribute('data-pan-x'))
  const box = await stage.boundingBox()
  if (!box) throw new Error('map stage has no box')
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(box.x + 24, box.y + box.height / 2 + 36, { steps: 8 })
  await page.mouse.up()
  await expect
    .poll(async () => Number(await stage.getAttribute('data-pan-x')))
    .not.toBe(zoomedPan)
  for (let step = 0; step < 24; step++) await page.mouse.wheel(0, 240)
  await expect
    .poll(async () => Number(await stage.getAttribute('data-zoom')))
    .toBeLessThan(minZoom + 0.01)

  const settingsPanel = dialog.getByTestId('map-settings-panel')
  await expect(settingsPanel).toHaveClass(/\bdark\b/)
  await expect(settingsPanel).toHaveClass(/bg-background\/30/)
  const panelButtons = settingsPanel.getByRole('button')
  await expect(panelButtons).toHaveCount(2)
  for (const button of await panelButtons.all()) {
    await expect(button).toHaveAttribute('data-variant', 'ghost')
  }
  const panelBounds = await settingsPanel.boundingBox()
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(box.x + box.width - 20, box.y + 80, { steps: 8 })
  await page.mouse.up()
  expect(await settingsPanel.boundingBox()).toEqual(panelBounds)

  const close = dialog.getByRole('button', { name: 'Close' })
  await expect(close).toBeVisible()
  await expect
    .poll(() =>
      close.evaluate((node) => {
        const rect = node.getBoundingClientRect()
        const hit = document.elementFromPoint(
          rect.left + rect.width / 2,
          rect.top + rect.height / 2
        )
        return hit === node || node.contains(hit)
      })
    )
    .toBe(true)
  const touchBlocked = await close.evaluate((node) => {
    const rect = node.getBoundingClientRect()
    const touch = new Touch({
      identifier: 1,
      target: node,
      clientX: rect.left + rect.width / 2,
      clientY: rect.top + rect.height / 2,
    })
    const event = new TouchEvent('touchmove', {
      bubbles: true,
      cancelable: true,
      touches: [touch],
      changedTouches: [touch],
    })
    node.dispatchEvent(event)
    return event.defaultPrevented
  })
  expect(touchBlocked).toBe(false)
  await close.click()
  await expect(dialog).toBeHidden()

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})

test('desktop maps use a large dialog', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 1200 })
  await installSyntheticMapReport(page)
  const browserErrors = await openAnalyzer(page, GENERIC_MAP_SEED)

  await expect(
    page.getByRole('button', { name: /^Rooms \(\d+\)$/ })
  ).toHaveCount(0)

  await page
    .getByRole('button', { name: `Expand floor ${GENERIC_MAP_FLOOR} map` })
    .first()
    .click()
  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible()
  await expect
    .poll(async () => (await dialog.boundingBox())?.width)
    .toBeGreaterThan(1200)
  await expect
    .poll(async () => (await dialog.boundingBox())?.height)
    .toBeGreaterThan(1100)
  await expect(
    dialog.getByRole('button', { name: 'Zoom map in' })
  ).toBeVisible()
  await expect(
    dialog.getByRole('button', { name: 'Zoom map out' })
  ).toBeVisible()
  const stage = dialog.getByTestId('map-scroll-container')
  await stage.hover()
  const beforeZoom = Number(await stage.getAttribute('data-zoom'))
  for (let step = 0; step < 6; step++) await page.mouse.wheel(0, -400)
  await expect
    .poll(async () => Number(await stage.getAttribute('data-zoom')))
    .toBeGreaterThan(beforeZoom)
  const beforePan = Number(await stage.getAttribute('data-pan-x'))
  const box = await stage.boundingBox()
  if (!box) throw new Error('map stage has no box')
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
  await page.mouse.down()
  await page.mouse.move(box.x + 40, box.y + box.height / 2 + 28, { steps: 6 })
  await page.mouse.up()
  await expect
    .poll(async () => Number(await stage.getAttribute('data-pan-x')))
    .not.toBe(beforePan)

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})

test('alchemy pots render in the floor banner', async ({ page }) => {
  await installSyntheticMapReport(page)
  const browserErrors = await openAnalyzer(page, GENERIC_MAP_SEED)

  const floor3 = page
    .locator('section')
    .filter({
      has: page.getByRole('heading', { name: 'Floor 3', exact: true }),
    })
    .last()
  await expect(
    floor3.getByRole('img', { name: 'Alchemy pot, Laboratory' })
  ).toBeVisible()
  await expect(page.getByText('Guaranteed appearances')).toHaveCount(0)
  await expect(page.getByText('Alchemy pot', { exact: true })).toHaveCount(0)
  await expect(
    page
      .locator('section')
      .filter({
        has: page.getByRole('heading', { name: 'Floor 4', exact: true }),
      })
      .last()
      .getByRole('img', { name: /Alchemy pot/ })
  ).toHaveCount(0)

  const pot = floor3.getByRole('img', { name: 'Alchemy pot, Laboratory' })
  await pot.hover()
  const detail = page.locator('[data-slot="item-detail"]')
  await expect(detail).toContainText('Alchemy pot')
  await expect(detail).toContainText(
    'This pot is filled with magical water. Items can be mixed into the pot to create something new!'
  )
  await floor3.getByRole('heading', { name: 'Floor 3', exact: true }).hover()
  await expect(detail).toHaveCount(0)
  await pot.click()
  await floor3.getByRole('heading', { name: 'Floor 3', exact: true }).hover()
  await expect(detail).toContainText('magical water')
  await page.keyboard.press('Escape')
  await expect(detail).toHaveCount(0)

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})

test('map dialog initially focuses its container instead of a control', async ({
  page,
}) => {
  await installSyntheticMapReport(page)
  const browserErrors = await openAnalyzer(page, GENERIC_MAP_SEED)
  await page
    .getByRole('button', { name: `Expand floor ${GENERIC_MAP_FLOOR} map` })
    .first()
    .click()

  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible()
  await expect(dialog).toBeFocused()
  await expect(
    dialog.getByRole('button', { name: 'Zoom map out' })
  ).not.toBeFocused()
  await expect(dialog.getByRole('button', { name: 'Close' })).not.toBeFocused()

  await page.keyboard.press('Tab')
  await expect(
    dialog.getByRole('button', { name: 'Zoom map out' })
  ).toBeFocused()

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})

test('Sad Ghost card shows only its resolved target', async ({ page }) => {
  const browserErrors = await openAnalyzer(page, 'AAA-AAA-AAA')

  await expect(
    page.getByRole('alert').filter({ hasText: 'Other Ghost options' })
  ).toHaveCount(0)
  await expect(
    page.getByRole('img', {
      name: /^(Fetid Rat|Gnoll Trickster|Great Crab)$/,
    })
  ).toBeVisible()
  await expect(page.getByText(/^Target:/)).toHaveCount(0)

  await expect(
    page.getByRole('img', { name: /^(Crystal|Gnoll)$/ })
  ).toBeVisible()
  await expect(
    page.getByText('target follows spawn floor', { exact: false })
  ).toHaveCount(0)

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})

test('Wandmaker outlook shows the seed-draw pair inside the deck window', async ({
  page,
}) => {
  const browserErrors = await openAnalyzer(page, 'AAA-AAA-AAA')
  const card = page.locator('[data-quest-type="old_wandmaker"]')
  await expect(card).toBeVisible()
  const questItem = card.getByLabel(/^(Corpse Dust|Elemental Embers|Rotberry)$/)
  await expect(questItem).toBeVisible()
  await questItem.hover()
  const examine = page.locator('[data-slot="item-detail"]')
  await expect(examine).toContainText(
    /malevolent energy|thermal energy|sweet, sweet death/
  )
  await page.keyboard.press('Escape')
  await expect(examine).toHaveCount(0)
  await card.getByRole('button', { name: 'Wand reward outlook' }).click()
  await expect(page.getByText('The player keeps one.')).toBeVisible()
  await expect(
    page.getByText("Wand 1's +N is fixed by the seed.")
  ).toBeVisible()
  await expect(page.getByText('seed draw').first()).toBeVisible()

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})

test('Other items explains gameplay-variable equipment rewards', async ({
  page,
}) => {
  const browserErrors = await openAnalyzer(page, 'AAA-AAA-AAA')
  const info = page
    .getByRole('button', { name: 'About items that vary with gameplay' })
    .first()
  await info.click()
  await expect(
    page.getByText('Varies with gameplay', { exact: true })
  ).toBeVisible()
  await expect(
    page.getByText('not seed-wide guarantees', { exact: false })
  ).toBeVisible()

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})

test('analyzer displays variable items in their ordinary item group', async ({
  page,
}) => {
  const browserErrors = await openAnalyzer(page, 'RZN-LKU-EFS')
  const variableItem = page.getByRole('listitem').filter({
    has: page.getByLabel(/dried Rose/i),
  })
  const notable = page
    .getByText(/^Notable/)
    .first()
    .locator('../..')

  await expect(page.getByText(/^Varies with gameplay/)).toHaveCount(0)
  await expect(notable.getByLabel(/dried Rose/i)).toBeVisible()
  await expect(variableItem).toBeVisible()
  await expect(variableItem.getByLabel(/cursed/i)).toBeVisible()
  await expect(page.getByText('planning only', { exact: true })).toHaveCount(0)

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})

test('finder result displays only matched constraints', async ({ page }) => {
  const errors: BrowserErrors = { console: [], page: [] }
  page.on('console', (message) => {
    if (message.type() === 'error') errors.console.push(message.text())
  })
  page.on('pageerror', (error) => errors.page.push(error.message))
  await page.emulateMedia({ colorScheme: 'light', reducedMotion: 'reduce' })
  await page.addInitScript((storage) => {
    localStorage.clear()
    localStorage.setItem(storage.mode, 'finder')
    localStorage.setItem(storage.theme, 'light')
  }, APP_STORAGE)

  await page.goto('/')
  await expect(page).toHaveTitle('SPD Seed Analyzer')
  await expect(
    page.getByRole('heading', { name: 'No searches yet' })
  ).toBeVisible()
  await page
    .getByRole('spinbutton', { name: 'Start seed' })
    .fill('3755006876613')
  await page.getByRole('spinbutton', { name: 'Candidates' }).fill('10')
  await page.getByLabel('Item 1 depth').selectOption('17')
  await page.getByRole('spinbutton', { name: 'Results' }).fill('1')
  await page.getByLabel('Item 1 name').selectOption('RingOfForce')
  await page.getByLabel('Item 1 upgrade level').selectOption('4')
  await page.getByRole('button', { name: 'Find' }).click()

  await expect(page.getByText('RZN-LKU-EIF', { exact: true })).toBeVisible({
    timeout: 60_000,
  })
  await expect(
    page.getByText('Matched constraints', { exact: true })
  ).toBeVisible()
  const resultItem = page
    .locator('[data-slot="item"]')
    .filter({ hasText: /ring of force/i })
  await expect(resultItem).toBeVisible()
  await expect(page.getByText('planning only', { exact: true })).toHaveCount(0)
  await expect(page.getByText('Ring of Wealth +2')).toHaveCount(0)

  expect(errors.console, 'browser console errors').toEqual([])
  expect(errors.page, 'uncaught page errors').toEqual([])
})

test('finder includes Wandmaker gameplay-variable items by default', async ({
  page,
}) => {
  const errors: BrowserErrors = { console: [], page: [] }
  page.on('console', (message) => {
    if (message.type() === 'error') errors.console.push(message.text())
  })
  page.on('pageerror', (error) => errors.page.push(error.message))
  await page.emulateMedia({ colorScheme: 'light', reducedMotion: 'reduce' })
  await page.addInitScript((storage) => {
    localStorage.clear()
    localStorage.setItem(storage.mode, 'finder')
    localStorage.setItem(storage.theme, 'light')
  }, APP_STORAGE)

  await page.goto('/')
  await page
    .getByRole('spinbutton', { name: 'Start seed' })
    .fill('3755006876548')
  await page.getByRole('spinbutton', { name: 'Candidates' }).fill('10')
  await page.getByLabel('Item 1 depth').selectOption('7')
  await page.getByRole('spinbutton', { name: 'Results' }).fill('1')
  await page.getByLabel('Item 1 category').selectOption('Wands')
  await page.getByLabel('Item 1 name').selectOption('WandOfDisintegration')
  await page.getByLabel('Item 1 upgrade level').selectOption('1')
  await expect(
    page.getByRole('switch', { name: /include .*variable/i })
  ).toHaveCount(0)
  await page.getByRole('button', { name: 'Find' }).click()

  await expect(page.getByText('RZN-LKU-EFS', { exact: true })).toBeVisible({
    timeout: 60_000,
  })
  const result = page
    .locator('[data-slot="item"]')
    .filter({ hasText: /wand of disintegration/i })
  await expect(result).toContainText(/wand of disintegration/i)
  await expect(
    result.getByText('Varies with gameplay', { exact: true })
  ).toBeVisible()

  expect(errors.console, 'browser console errors').toEqual([])
  expect(errors.page, 'uncaught page errors').toEqual([])
})

test('floor items render as icon slots', async ({ page }) => {
  await installSyntheticMapReport(page)
  const browserErrors = await openAnalyzer(page, GENERIC_MAP_SEED)

  await expect(page.getByLabel('Modeled outcome')).toHaveCount(0)
  await expect(
    page.getByRole('switch', { name: 'Forbidden Runes' })
  ).toHaveCount(0)
  await expect(page.getByRole('switch', { name: 'Identities' })).toHaveCount(0)
  await expect(page.getByRole('switch', { name: 'Floor maps' })).toHaveCount(0)
  await expect(page.getByRole('heading', { name: 'Identities' })).toBeVisible()
  await expect(page.getByRole('tab', { name: 'Potions' })).toHaveCount(0)
  for (const group of ['Potions', 'Scrolls', 'Rings']) {
    await expect(page.getByRole('heading', { name: group })).toHaveCount(0)
  }
  await expect(page.getByRole('heading', { name: 'Floor 1' })).toBeVisible()
  const syntheticItem = page.getByRole('listitem').filter({
    has: page.getByLabel(/Scroll of Transmutation/i),
  })
  await expect(syntheticItem).toBeVisible()
  await expect(syntheticItem.getByLabel(/cursed/i)).toBeVisible()
  await expect(syntheticItem.getByLabel(/Blazing/i)).toBeVisible()
  await expect(
    page.getByRole('button', { name: 'Show upgrade conditions' })
  ).toHaveCount(0)
  await expect(
    page.getByRole('button', { name: 'Forbidden Runes disabled' })
  ).toHaveCount(0)
  await expect(
    page.getByRole('button', { name: 'Show enchantment conditions' })
  ).toHaveCount(0)
  await expect(
    page.getByRole('button', { name: 'Show trinket transmutation rotation' })
  ).toHaveCount(0)
  await expect(
    page.getByLabel('Magical Catalyst', { exact: true })
  ).toBeVisible()
  const floor3 = page
    .getByRole('heading', { name: 'Floor 3', exact: true })
    .locator('xpath=ancestor::section[1]')
  const floor4 = page
    .getByRole('heading', { name: 'Floor 4', exact: true })
    .locator('xpath=ancestor::section[1]')
  const choice = floor3.locator('[data-trinket-choice]')
  const mossy = choice.getByRole('radio', { name: 'Mossy Clump' })
  await expect(mossy).toBeVisible()
  await expect(mossy).toHaveAttribute('aria-checked', 'false')
  await expect(mossy.locator('[data-slot="item-box"]')).toHaveCSS(
    'width',
    '59px'
  )
  await expect(choice.getByRole('radio', { name: 'Mimic Tooth' })).toBeVisible()
  await expect(choice.getByRole('radio', { checked: true })).toHaveCount(0)
  await expect(floor4.locator('[data-trinket-choice]')).toHaveCount(0)
  const order = choice.locator(
    '[data-transmutation-order] [data-slot="item-box"]'
  )
  await expect(order).toHaveCount(13)
  await expect(order.first()).toHaveAttribute('aria-label', 'Trap Mechanism')
  await expect(order.nth(1)).toHaveAttribute('aria-label', 'Parchment Scrap')
  await mossy.click()
  await expect(mossy).toHaveAttribute('aria-checked', 'true')
  await expect(
    floor4.getByRole('button', {
      name: 'Generated with Mossy Clump +3. Change trinket',
    })
  ).toBeVisible()
  await expect(
    floor3.getByRole('button', { name: /^Generated with/ })
  ).toHaveCount(0)
  await mossy.click()
  await expect(mossy).toHaveAttribute('aria-checked', 'false')
  await expect(
    page.getByRole('button', { name: /^Generated with/ })
  ).toHaveCount(0)
  const sickle = page.getByRole('listitem').filter({
    has: page.getByLabel(/sickle \+2/i),
  })
  await expect(sickle).toHaveCount(1)
  await expect(sickle).toContainText('+2')
  await expect(page.getByLabel(/weapon reward/i)).toHaveCount(0)
  await sickle.getByLabel(/sickle \+2/i).click()
  const sickleDetail = page.locator('[data-slot="item-detail"]')
  await expect(sickleDetail).toContainText('Always true')
  await expect(sickleDetail).toContainText(
    'Gameplay can change the concrete item; these seed properties stay fixed.'
  )
  await expect(sickleDetail).toContainText('Tier 2–5')
  await page.keyboard.press('Escape')
  await expect(sickleDetail).toHaveCount(0)
  await expect(page.getByText('Guaranteed spawns')).toHaveCount(0)
  await expect(page.getByText('Floor loot')).toHaveCount(0)
  const emptyNotable = page.getByText('No notable items on this floor.')
  await expect(emptyNotable).toHaveCount(3)
  await expect(emptyNotable.first()).toHaveCSS('font-style', 'normal')
  await expect(page.getByText(/^Other items/)).toBeVisible()
  await expect(
    floor4.getByLabel('Potion of Healing', { exact: true })
  ).toBeVisible()
  await expect(
    page.getByText('Varies with gameplay', { exact: true })
  ).toHaveCount(0)
  const floor1Banner = page
    .getByRole('heading', { name: 'Floor 1', exact: true })
    .locator('xpath=ancestor::section[1]')
    .locator('> div')
    .first()
  const floor1Box = await floor1Banner.boundingBox()
  expect(floor1Box?.height ?? 0).toBeLessThan(64)
  await expect(
    page.getByRole('heading', { name: 'Floor 1', exact: true })
  ).toHaveCSS('color', 'rgb(255, 255, 255)')
  await expect(
    page.getByRole('heading', { name: 'Floor 1', exact: true })
  ).toHaveCSS('font-family', /SPD Pixel/)
  await expect(
    page.getByRole('heading', { name: 'Floor 1', exact: true })
  ).toHaveCSS('-webkit-text-stroke-width', '4px')
  await floor4.getByRole('heading', { name: 'Floor 4', exact: true }).click()
  await expect(page.getByRole('dialog')).toHaveCount(0)
  await expect(
    page.getByRole('button', { name: 'Expand floor 4 map' })
  ).toHaveCount(1)

  expect(browserErrors.console, 'browser console errors').toEqual([])
  expect(browserErrors.page, 'uncaught page errors').toEqual([])
})
