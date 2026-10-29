import { expect, type Page, test } from '@playwright/test'

const TEST_STORAGE_KEY = 'spd-analyzer-encounter-test-initialized'

async function openFloorOne(page: Page, seed: string) {
  await page.addInitScript((storageKey) => {
    if (sessionStorage.getItem(storageKey) !== 'true') {
      localStorage.clear()
      localStorage.setItem('spd-analyzer-mode', 'analyze')
      localStorage.setItem('spd-analyzer-theme', 'light')
      sessionStorage.setItem(storageKey, 'true')
    }
  }, TEST_STORAGE_KEY)

  await page.goto('/')
  await page.getByLabel('Enter your seed').fill(seed)
  await page.getByRole('button', { name: 'Analyze', exact: true }).click()
  const heading = page.getByRole('heading', { name: 'Floor 1', exact: true })
  await expect(heading).toBeVisible({ timeout: 60_000 })
  return heading.locator('xpath=ancestor::section[1]')
}

test('floor one shows exact room rewards and non-positional encounters', async ({
  page,
}) => {
  const browserErrors: string[] = []
  page.on('console', (message) => {
    if (message.type() === 'error') browserErrors.push(message.text())
  })
  page.on('pageerror', (error) => browserErrors.push(error.message))

  const floor = await openFloorOne(page, 'AAA-AAA-AFU')
  await expect(floor.getByText('Initial encounters')).toHaveCount(0)
  await expect(floor.getByLabel(/stone of blast/i)).toBeVisible()
  await expect(floor.getByLabel(/stone of deep sleep/i)).toBeVisible()
  await expect(floor.getByText('2–3 runestone-room rewards')).toHaveCount(0)

  await page.setViewportSize({ width: 390, height: 844 })
  await floor.scrollIntoViewIfNeeded()
  const viewport = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }))
  expect(viewport.scrollWidth).toBe(viewport.clientWidth)
  expect(browserErrors).toEqual([])
})

test('Shattered Honeypot uses its in-game item sprite', async ({ page }) => {
  await openFloorOne(page, 'MWH-KAE-DHG')
  const floor = page
    .getByRole('heading', { name: 'Floor 2', exact: true })
    .locator('xpath=ancestor::section[1]')
  const icon = floor.getByLabel('Shattered Honeypot', { exact: true })

  await expect(icon).toBeVisible()
  await expect(icon.locator('[aria-hidden="true"]')).toHaveCSS(
    'background-position',
    '-192px -96px'
  )
})

test('floor lists include map drops and trinket picks only affect later floors', async ({
  page,
}) => {
  const browserErrors: string[] = []
  page.on('console', (message) => {
    if (message.type() === 'error') browserErrors.push(message.text())
  })
  page.on('pageerror', (error) => browserErrors.push(error.message))

  await openFloorOne(page, 'CHL-XBM-ZWD')
  const floor2Zwd = page.locator('section[data-floor-depth="2"]')
  const floor3Zwd = page.locator('section[data-floor-depth="3"]')

  await expect(
    floor2Zwd.getByLabel('Scroll of Remove Curse', { exact: true })
  ).toBeVisible()
  await expect(
    floor2Zwd.getByLabel('Ration of Food', { exact: true })
  ).toBeVisible()

  const ghostZwd = floor2Zwd.locator('[data-quest-type="sad_ghost"]')
  const ghostRewardsZwd = ghostZwd.locator('ul li button[aria-label]')
  await expect(ghostRewardsZwd).toHaveCount(2)
  const beforeLabels = await ghostRewardsZwd.evaluateAll((buttons) =>
    buttons.map((button) => button.getAttribute('aria-label'))
  )

  const pickerZwd = floor2Zwd.locator('[data-trinket-choice]')
  await pickerZwd.getByRole('radio', { name: 'Parchment Scrap' }).click()
  await expect(pickerZwd).toContainText(
    'Floors 3+ are generated with Parchment Scrap +3.'
  )

  await expect(
    floor2Zwd.getByRole('button', { name: /Generated with Parchment Scrap/ })
  ).toHaveCount(0)
  await expect(
    floor3Zwd.getByRole('button', { name: /Generated with Parchment Scrap/ })
  ).toBeVisible()
  await expect(ghostRewardsZwd).toHaveCount(2)
  const afterLabels = await ghostRewardsZwd.evaluateAll((buttons) =>
    buttons.map((button) => button.getAttribute('aria-label'))
  )
  expect(afterLabels).toEqual(beforeLabels)

  await page.getByLabel('Enter your seed').fill('CHL-XBM-ZDD')
  await page.getByRole('button', { name: 'Analyze', exact: true }).click()
  const floor2Zdd = page.locator('section[data-floor-depth="2"]')
  const floor3Zdd = page.locator('section[data-floor-depth="3"]')
  await expect(
    floor2Zdd.getByLabel('Scroll of Lullaby', { exact: true })
  ).toBeVisible({ timeout: 60_000 })
  await expect(floor2Zdd.getByLabel('Pasty', { exact: true })).toBeVisible()

  const ghostRewardsZdd = floor2Zdd
    .locator('[data-quest-type="sad_ghost"]')
    .locator('ul li button[aria-label]')
  await expect(ghostRewardsZdd).toHaveCount(2)
  const zddBefore = await ghostRewardsZdd.evaluateAll((buttons) =>
    buttons.map((button) => button.getAttribute('aria-label'))
  )

  const pickerZdd = floor3Zdd.locator('[data-trinket-choice]')
  await pickerZdd.getByRole('radio', { name: 'Trap Mechanism' }).click()
  await expect(pickerZdd).toContainText(
    'Floors 4+ are generated with Trap Mechanism +3.'
  )
  const zddAfter = await ghostRewardsZdd.evaluateAll((buttons) =>
    buttons.map((button) => button.getAttribute('aria-label'))
  )
  expect(zddAfter).toEqual(zddBefore)
  expect(browserErrors).toEqual([])
})

async function clickMapFraction(page: Page, fx: number, fy: number) {
  const canvas = page.locator('canvas[data-map-canvas]')
  await expect(canvas).toBeVisible()
  const box = await canvas.boundingBox()
  if (!box) throw new Error('missing map canvas box')
  await page.mouse.click(box.x + box.width * fx, box.y + box.height * fy)
}

test('animated and armored statues report their drops in floor lists and map inspect', async ({
  page,
}) => {
  const browserErrors: string[] = []
  page.on('console', (message) => {
    if (message.type() === 'error') browserErrors.push(message.text())
  })
  page.on('pageerror', (error) => browserErrors.push(error.message))

  await openFloorOne(page, 'CHL-XBM-ZWD')
  const floor2 = page.locator('section[data-floor-depth="2"]')
  const staff = floor2.getByRole('button', {
    name: /Kinetic Quarterstaff/i,
  })
  await expect(staff).toBeVisible()
  await staff.click()
  await expect(page.locator('[data-slot="item-detail"]')).toContainText(
    'carried by an animated statue'
  )
  await page.keyboard.press('Escape')

  await floor2.getByRole('button', { name: 'Expand floor 2 map' }).click()
  await clickMapFraction(page, 0.9390243902439024, 0.5972222222222222)
  const inspect = page.locator('[data-slot="map-inspect"]')
  await expect(inspect).toContainText('Animated statue')
  await expect(inspect).toContainText('Kinetic Quarterstaff')
  await expect(inspect).toContainText('Drops when defeated.')
  await page.getByRole('button', { name: 'Close' }).click()

  await page.getByLabel('Enter your seed').fill('AAA-AAA-AAJ')
  await page.getByRole('button', { name: 'Analyze', exact: true }).click()
  const floor1 = page.locator('section[data-floor-depth="1"]')
  await expect(
    floor1.getByRole('button', { name: /Corrupting Katana/i })
  ).toBeVisible({ timeout: 60_000 })
  await expect(
    floor1.getByRole('button', { name: /Leather Armor of Entanglement \+1/i })
  ).toBeVisible()

  await floor1.getByRole('button', { name: 'Expand floor 1 map' }).click()
  await clickMapFraction(page, 0.4852941176470588, 0.421875)
  await expect(inspect).toContainText('Armored statue')
  await expect(inspect).toContainText('Corrupting Katana')
  await expect(inspect).toContainText('Leather Armor of Entanglement +1')
  expect(browserErrors).toEqual([])
})
