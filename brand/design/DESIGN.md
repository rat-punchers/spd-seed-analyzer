# SPD Seed Analyzer design system

**Lantern on stone.** The analyzer is a reference for Shattered Pixel Dungeon
players reading one seed floor by floor, or searching many seeds for the items
they want. The interface is built from the game's own materials: stone for
surfaces, the yellow of the game's window titles for action, one hue per
dungeon region, and the game's pixel font for places and numbers.

Built on **shadcn/ui** (`radix-lyra` style, Phosphor icons) and **Tailwind CSS
v4**. Tokens live in [`src/theme/theme.css`](src/theme/theme.css). Every rule
below is shown live in the preview app (`bun run design`, port 5174).

## Principles

1. **Hue is information.** Color marks depth (region), loot (upgrade tier), or
   status. It never decorates. Everything else is stone.
2. **One lantern per view.** The primary color goes on the action the view
   exists for, the selected floor, and keyboard focus. Nothing else.
3. **Carved, not floated.** State is a bevel: raised, pressed, recessed. Only
   overlays cast a shadow, and it is a hard 4px drop.
4. **Respect the pixel grid.** Game art and the pixel font render at whole
   multiples only, with smoothing off. Pixel surfaces keep square corners.
5. **Report what the seed decides.** Copy states guaranteed facts plainly and
   names the conditions when player choices change an outcome.

## Color

All values are OKLCH in `theme.css`; hex here is the sRGB rendering.

### Stone and lantern

Neutrals carry a faint green cast sampled from the game's window chrome
(`interfaces/chrome.png`) and floor tiles. Surfaces step lighter as they rise
toward the player.

| Token | Name | Limestone (light) | Torchlit (dark) | Use |
|---|---|---|---|---|
| `--background` | Ground | `#e4e3dc` | `#151411` | Page |
| `--card` | Slab | `#f2f1ed` | `#1d1c18` | Windows, report frame |
| `--popover` | Raised slab | `#f7f7f4` | `#23221d` | Dialogs, popovers |
| `--muted` | Hollow | `#dad9d2` | `#252420` | Slots, trays, hover |
| `--secondary` | Stone | `#d6d4cc` | `#2c2b26` | Secondary buttons |
| `--border` | Mortar | `#bfbeb5` | `#363630` | Dividers, frames |
| `--input` | Chisel | `#828078` | `#6d6c65` | Control edges |
| `--muted-foreground` | Lichen | `#5c5b53` | `#a3a299` | Secondary text |
| `--foreground` | Ink | `#191814` | `#e6e5dc` | Body text |
| `--primary` | Lantern | `#7c5500` | `#e9c85e` | Primary action |
| `--accent` | Lantern wash | `#e1d7be` | `#36301f` | Pressed toggles, quest band |
| `--ring` | Ring | `#a67700` | `#e9c85e` | Keyboard focus |

The dark lantern is the game's `Window.TITLE_COLOR` (`0xFFFF44`) softened;
the light lantern is the same hue deepened to brass so chalk text reads on it.

### Regions

One hue per region, **identical in both themes**. They fill floor numbers,
banner depth blocks, the descent rail, and region badges, and appear nowhere
else. Text on a region fill is always `--region-foreground` (`#100f0c`).
Tailwind: `bg-sewers`, `bg-prison`, `bg-caves`, `bg-city`, `bg-halls`,
`text-region-foreground`. They double as `--chart-1` … `--chart-5`.

| Token | Region | Floors | Hex | Source in the game | Ink contrast |
|---|---|---|---|---|---|
| `--region-sewers` | Sewers | 1–5 | `#94b072` | Moss on the sewer flagstones | 7.95:1 |
| `--region-prison` | Prison | 6–10 | `#de9b52` | Torchlight in the prison halls | 8.14:1 |
| `--region-caves` | Caves | 11–15 | `#7aa3c8` | Cold night air over the mines | 7.21:1 |
| `--region-city` | Dwarven City | 16–20 | `#ba85a8` | Rose stone of the dwarven walls | 6.40:1 |
| `--region-halls` | Demon Halls | 21–26 | `#d86353` | Lava in the hex floor | 5.32:1 |

### Loot tiers and status

The `+N` on an item is colored by tier: `+1`, `+2`, `+3`, then `+4` and
above. Negative levels and cursed frames use `--degraded`, the game's
`ItemSlot.DEGRADED` red. Labels drawn over sprite art carry the `dark` class
so they use the Torchlit values in both themes: the game always outlines them
in black (`ink-outline`), so they need the bright colors.

| Token | Limestone | Torchlit | Use |
|---|---|---|---|
| `--tier-1` | `#1c6d26` | `#6fda75` | `+1` |
| `--tier-2` | `#2266a4` | `#62bbf5` | `+2` |
| `--tier-3` | `#7347af` | `#b995f6` | `+3` |
| `--tier-4` | `#7e5904` | `#efca55` | `+4` and above |
| `--degraded` | `#be2323` | `#f66d67` | Negative levels, cursed frames |
| `--destructive` | `#be2323` | `#f36358` | Errors, removal |
| `--warning` | `#994a00` | `#f19a4b` | Partial results |
| `--info` | `#296898` | `#74b3de` | Version and scope notes |

### Contrast

Measured with the WCAG 2 formula on the hex values above.

| Pair | Limestone | Torchlit |
|---|---|---|
| Ink on ground | 13.81:1 | 14.57:1 |
| Lichen on ground | 5.30:1 | 7.18:1 |
| Primary text on lantern | 6.20:1 | 11.30:1 |
| Warning on ground | 4.90:1 | 8.30:1 |
| Destructive on ground | 4.72:1 | 5.91:1 |
| Lowest tier on ground | 4.66:1 | 7.62:1 |
| Lowest tier on card | 5.30:1 | 7.06:1 |
| Chisel (input edge) on ground | 3.07:1 | 3.49:1 |
| Chisel on card | 3.50:1 | 3.23:1 |
| Ring on ground | 3.10:1 | 11.30:1 |

## Type

| Role | Family | Tailwind | Use |
|---|---|---|---|
| Display | SPD Pixel (the game's `pixel_font.ttf`, Apache-2.0) | `font-pixel`, `font-heading` | Seed codes on reports, headings, depth numbers, `+N` and stack counts on slots |
| Text | Atkinson Hyperlegible Next Variable | `font-sans` (default) | Everything read: body, controls, tables |
| Entry | Atkinson Hyperlegible Mono Variable | `font-mono` | Seed codes and dates in inputs and result lists, code tokens |

Atkinson Hyperlegible keeps `I l 1`, `O 0` and `5 S` distinct, which matters
for nine-letter seed codes.

### Pixel scale

The pixel font is drawn on 100-unit steps in a 2048-unit em, so one font
pixel is `2048 / 100 = 20.48px` per step. It renders crisp **only** at these
sizes. Never set it smaller, never add letter spacing, never use it for
running text.

| Utility | Size | Line height | Use |
|---|---|---|---|
| `text-px-4` | 81.92px | 80px | Hero display only; nothing in the report uses it |
| `text-px-3` | 61.44px | 60px | Page titles, the seed at the top of a report (`text-px-2` on phones) |
| `text-px-2` | 40.96px | 48px | Section titles, depth numbers |
| `text-px-1` | 20.48px | 24px | Subsections, dialog and card titles, slot labels |

`cn()` registers the custom scales with tailwind-merge: `text-px-*` as font
sizes, `inset-shadow-raised|pressed|recessed` as inset shadows and
`shadow-window` as a shadow. tailwind-merge reads unknown `text-*` and
`inset-shadow-*` values as colors, so without this `cn('text-px-1',
'text-tier-3')` drops the size and `cn('inset-shadow-recessed',
'inset-shadow-none')` keeps both. For the same reason no custom utility uses
the `text-` prefix unless it is a size or a color: the game's black label
outline is `ink-outline`. Add any new custom scale to `cn()` in the same
change.

### Text scale

Tailwind's defaults, which sit on the classical scale: `text-xs` 12/16 for
captions under slots, `text-sm` 14/20 for controls and item groups (the
smallest control text), `text-base` 16/24 for reading, `text-lg` 18/28 for
introductions. Keep prose under about 65 characters per line. Labels are
sentence case; counts sit next to the label in `tabular-nums`, never in
brackets or capitals.

## Shape and depth

| Surface | Radius | Examples |
|---|---|---|
| Pixel surface | `rounded-none` | Item slots, sprites, maps, region fills, rail cells |
| Control | `rounded-sm` (2px) | Buttons, inputs, toggles, badges, tooltips |
| Window | `rounded-lg` (4px) | Cards, dialogs, popovers |

`--radius` is 3px; `sm`, `md`, `lg`, `xl` derive from it.

Bevels are Tailwind `inset-shadow` tokens, so they compose with focus rings:

| Utility | Recipe | Use |
|---|---|---|
| `inset-shadow-raised` | 1px top highlight, 2px base shadow | Buttons at rest, active tab, region fills |
| `inset-shadow-pressed` | 2px top shadow | Held buttons, toggles that are on |
| `inset-shadow-recessed` | 2px top shadow, 1px base highlight | Inputs, slots, tab trays, gauges |
| `shadow-window` | Hard 4px drop | Dialogs, popovers, tooltips only |

**Pixel art.** Sprites and tiles use the `pixelated` utility and scale by
whole numbers: 2× in lists (a slot is the game's 17px journal cell at 2×),
3× for quest givers and detail views. The `Sprite` component rounds any
fractional scale.

## Motion

- Motion answers an action: a pressed button drops 1px, a rail cell lifts on
  hover and stays lifted while selected, overlays fade and scale in 100ms.
- No ambient or entrance animation. The game's enchantment glow and the idle
  clips of game objects (mobs, NPCs, bosses) are the exceptions, because the
  game itself shows them.
- `prefers-reduced-motion` collapses every animation and transition
  (`theme.css` base layer).

## Components

`src/components/ui/` holds the shadcn components, restyled from `radix-lyra`:
14px controls, 2px corners, bevel states, recessed inputs, and these variants
added to the stock set:

- `Badge`: `warning`, and one variant per region (`sewers` … `halls`).
- `Alert`: `info` and `warning`; severity is a 4px left rule plus the title
  color, while the description stays in body ink.
- `Tabs`: `default` is a recessed tray with a raised active tab; `line` marks
  the active tab with a 2px lantern bar.
- `Switch`: a square thumb on a recessed track.
- `Checkbox`: a recessed well that fills with lantern and a bold check when
  checked.
- `Progress`: a recessed, square-ended gauge.

`src/components/spd/` holds the game pieces:

| Component | Role |
|---|---|
| `Sprite` | One crop from a game sprite sheet at a whole-number scale |
| `ItemSlot` | Recessed 2× journal cell with tier-colored `+N`, stack count, enchantment frame, cursed frame, caption |
| `FloorBanner` | Region-filled depth block, then the region's floor tiles darkened behind outlined white text; a boss floor shows its boss sprite. A quest band reuses it without the depth block: NPC, quest name, target, and a map button when the quest has its own floor |
| `DescentRail` | All 26 floors in five region bands; skull for a boss, square for a quest, one pip per notable item |
| `FloorReport` | Banner, quest band, notable items, other items |

Cards are windows for content that stands apart from the flow, such as item
details or the finder. The report itself uses bands and rules, not cards.

## Voice

| Write | Not | Why |
|---|---|---|
| Analyze seed | Submit | Buttons name what happens; the same verb names the result |
| That seed has 8 letters. Seed codes have 9, like ABC-DEF-GHI. | Invalid input! | Errors say what is wrong and how to fix it, without apology |
| No seeds analyzed yet. Enter a seed to see what each floor holds. | Nothing to see here | Empty screens point to the next step |
| Guaranteed on floor 4 | Probably on floor 4 | Claim only what the seed decides |
| One of three wands, each +1 | Random wand | Keep every fact the seed fixes when part of a result varies |
| Scroll of Upgrade | ScrollOfUpgrade, deck index 3 | Use the game's names |
| Notable 2 | NOTABLE (2) | Sentence case; plain counts |

## In the web app

- `web/src/index.css` imports `@spd-seed-analyzer/design/theme.css` after
  Tailwind, `tw-animate-css` and `shadcn/tailwind.css`; the import brings the
  fonts, `@custom-variant dark`, tokens, base layer and utilities. Tokens are
  single-sourced here.
- The shadcn components are copies: `web/src/components/ui/` owns the app's
  set (plus the app-only `item` and `combobox`), and `src/components/ui/`
  here mirrors the shared ones for the preview. Change a shared component in
  both places. `cn()` is identical in both `lib/utils.ts` files.
- The game pieces are the app's own data-driven components, styled to this
  spec: `ItemBox` (item slot), `seed/FloorBanner`, `seed/DescentRail`, and
  `GameWindow`, the game's `Chrome.Type.WINDOW` nine-patch at 2× around the
  report's seed. The frame is game art, so its contents use Torchlit tokens.
  In the app the descent rail is a compact sticky strip under the seed tabs:
  region names move into each floor's tooltip, clicking a floor scrolls to
  it, and the floor being read is outlined. Floor banners stick beneath it
  via `--descent-rail-height`.
- Floor banners draw `web/public/assets/environment/strip_<tileset>.png`,
  picked by the map's own tileset so mine branches show their tiles.
