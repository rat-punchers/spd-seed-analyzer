import {
  CopyIcon,
  MagnifyingGlassIcon,
  MapTrifoldIcon,
  PlantIcon,
  PlusIcon,
  SpinnerGapIcon,
  StopIcon,
  TrashIcon,
} from '@phosphor-icons/react'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldLabel,
} from '@/components/ui/field'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from '@/components/ui/input-group'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { Subsection } from '@/preview/section'

function SeedField({ invalid = false }: { invalid?: boolean }) {
  const id = invalid ? 'seed-invalid' : 'seed'
  return (
    <Field data-invalid={invalid || undefined}>
      <FieldLabel htmlFor={id}>Seed</FieldLabel>
      <div className="flex w-full items-stretch gap-2">
        <InputGroup className="min-w-0 flex-1">
          <InputGroupAddon align="inline-start" aria-hidden>
            <PlantIcon />
          </InputGroupAddon>
          <InputGroupInput
            id={id}
            defaultValue={invalid ? 'ABC-DEF-GH' : undefined}
            placeholder="ABC-DEF-GHI or 2026-10-14"
            aria-invalid={invalid || undefined}
            aria-describedby={`${id}-help`}
            autoCapitalize="characters"
            autoComplete="off"
            spellCheck={false}
            maxLength={11}
            className="font-mono"
          />
        </InputGroup>
        <Button type="button">
          <MagnifyingGlassIcon data-icon="inline-start" />
          Analyze
        </Button>
      </div>
      {invalid ? (
        <FieldError id={`${id}-help`}>
          That seed has 8 letters. Seed codes have 9, like ABC-DEF-GHI.
        </FieldError>
      ) : (
        <FieldDescription id={`${id}-help`}>
          Seed codes have 9 letters. Daily runs use the date.
        </FieldDescription>
      )}
    </Field>
  )
}

export function ControlsSpecimen() {
  return (
    <>
      <Subsection
        title="Buttons"
        note="One lantern button per view: the action the page exists for. Labels name the action and keep that name through the flow."
      >
        <div className="flex flex-col gap-5 border-y py-6">
          <div className="flex flex-wrap items-center gap-3">
            <Button>
              <MagnifyingGlassIcon data-icon="inline-start" />
              Analyze seed
            </Button>
            <Button variant="secondary">
              <PlusIcon data-icon="inline-start" />
              Add constraint
            </Button>
            <Button variant="outline">
              <CopyIcon data-icon="inline-start" />
              Copy seed
            </Button>
            <Button variant="ghost">Clear</Button>
            <Button variant="destructive">
              <StopIcon data-icon="inline-start" />
              Stop search
            </Button>
            <Button variant="link">Read the changelog</Button>
          </div>
          <div className="flex flex-wrap items-center gap-3">
            <Button variant="secondary" size="xs">
              Extra small
            </Button>
            <Button variant="secondary" size="sm">
              Small
            </Button>
            <Button variant="secondary">Default</Button>
            <Button variant="secondary" size="lg">
              Large
            </Button>
            <Button disabled>
              <SpinnerGapIcon
                data-icon="inline-start"
                className="animate-spin"
              />
              Analyzing
            </Button>
            <Button variant="secondary" size="icon" aria-label="Open the map">
              <MapTrifoldIcon />
            </Button>
            <Button variant="ghost" size="icon" aria-label="Remove seed">
              <TrashIcon />
            </Button>
          </div>
        </div>
      </Subsection>
      <Subsection
        title="Fields"
        note="Inputs are recessed wells. Errors say what is wrong and how to fix it, right under the field."
      >
        <div className="grid gap-8 border-y py-6 md:grid-cols-2">
          <SeedField />
          <SeedField invalid />
          <Field>
            <FieldLabel htmlFor="category">Item category</FieldLabel>
            <NativeSelect id="category" defaultValue="ring" className="w-full">
              <NativeSelectOption value="weapon">Weapon</NativeSelectOption>
              <NativeSelectOption value="armor">Armor</NativeSelectOption>
              <NativeSelectOption value="ring">Ring</NativeSelectOption>
              <NativeSelectOption value="wand">Wand</NativeSelectOption>
              <NativeSelectOption value="artifact">Artifact</NativeSelectOption>
            </NativeSelect>
          </Field>
          <div className="flex flex-col gap-5">
            <Field orientation="horizontal">
              <Switch id="deep-search" />
              <FieldContent>
                <FieldLabel htmlFor="deep-search">Deep search</FieldLabel>
                <FieldDescription>
                  Also match Imp and Wandmaker rewards. Slower.
                </FieldDescription>
              </FieldContent>
            </Field>
            <Field orientation="horizontal">
              <Checkbox id="lore-pages" />
              <FieldContent>
                <FieldLabel htmlFor="lore-pages">Show lore pages</FieldLabel>
                <FieldDescription>
                  Place guide and alchemy pages, as on a new journal.
                </FieldDescription>
              </FieldContent>
            </Field>
            <Field>
              <FieldLabel id="match-label">Match</FieldLabel>
              <ToggleGroup
                type="single"
                variant="outline"
                spacing={0}
                defaultValue="all"
                aria-labelledby="match-label"
              >
                <ToggleGroupItem value="all">All constraints</ToggleGroupItem>
                <ToggleGroupItem value="any">Any constraint</ToggleGroupItem>
              </ToggleGroup>
            </Field>
          </div>
        </div>
      </Subsection>
    </>
  )
}
