# Component Patterns

All components use `cva` + `tailwind-merge` via the `cn()` utility from `@/lib/utils`. Follow shadcn/ui New York style conventions.

## Buttons

### Primary Button

```tsx
// Gradient background, white text, accent shadow on hover
<Button className="bg-gradient-to-r from-[var(--accent)] to-[var(--accent-secondary)] text-white shadow-sm hover:shadow-[var(--shadow-accent)] hover:-translate-y-0.5 active:scale-[0.98] transition-all duration-200 rounded-xl h-12 px-6">
  Action
</Button>
```

### Button Variants (CVA)

| Variant | Styles |
|:--------|:-------|
| `default` | Gradient accent bg, white text, shadow |
| `outline` | Transparent bg, border, muted hover |
| `secondary` | Muted bg, foreground text |
| `ghost` | No bg/border, muted-foreground text |
| `destructive` | Red bg, white text |
| `link` | Underline, accent color |

### Button Sizes

| Size | Height | Padding | Text |
|:-----|:-------|:--------|:-----|
| `sm` | `h-9` | `px-3` | `text-xs` |
| `default` | `h-10` | `px-4` | `text-sm` |
| `lg` | `h-12` | `px-6` | `text-base` |
| `icon` | `h-10 w-10` | — | — |

### Hover Behavior

- Lift: `hover:-translate-y-0.5`
- Shadow deepens: `shadow-sm` → `shadow-accent` (accent-tinted)
- Active press: `active:scale-[0.98]`
- Transition: `transition-all duration-200`
- Arrow icons: `group-hover:translate-x-1`

## Cards

### Standard Card

```tsx
<Card className="rounded-xl border border-[var(--border)] bg-[var(--card)] shadow-md hover:shadow-xl transition-all duration-300">
  <CardHeader>
    <CardTitle>Title</CardTitle>
    <CardDescription>Description</CardDescription>
  </CardHeader>
  <CardContent>Content</CardContent>
</Card>
```

### Card Hover Effects

- Gradient overlay fades in: `bg-gradient-to-br from-accent/[0.03] to-transparent`
- Shadow deepens: `shadow-md` → `shadow-xl`
- Optional icon scale: `group-hover:scale-110`

### Featured Card (Gradient Border)

```tsx
<div className="rounded-xl bg-gradient-to-br from-[var(--accent)] via-[var(--accent-secondary)] to-[var(--accent)] p-[2px]">
  <div className="h-full w-full rounded-[calc(12px-2px)] bg-[var(--card)]">
    {/* content */}
  </div>
</div>
```

## Section Labels (Badges)

Consistent pill badge at the start of each section:

```tsx
<div className="inline-flex items-center gap-3 rounded-full border border-[var(--accent)]/30 bg-[var(--accent)]/5 px-5 py-2">
  <span className="h-2 w-2 rounded-full bg-[var(--accent)]" />
  <span className="font-mono text-xs uppercase tracking-[0.15em] text-[var(--accent)]">
    Section Name
  </span>
</div>
```

## Inputs

- Height: `h-12` to `h-14`
- Border: `1px` in `border` color
- Border-radius: `rounded-lg` or `rounded-xl`
- Focus: `ring-2 ring-[var(--accent)] ring-offset-2`
- Placeholder: `text-muted-foreground/50`

## Badges & Tags

```tsx
<Badge variant="outline" className="border-[var(--accent)]/30 bg-[var(--accent)]/5 text-[var(--accent)]">
  Label
</Badge>
```

## Icon Containers

Gradient backgrounds for feature icons:

```tsx
<div className="flex h-12 w-12 items-center justify-center rounded-xl bg-gradient-to-br from-[var(--accent)] to-[var(--accent-secondary)] text-white">
  <Icon className="h-6 w-6" />
</div>
```

## Accessibility

- All interactive elements: minimum `44px` touch target
- Focus rings: `ring-2 ring-[var(--accent)] ring-offset-2`
- Color contrast: all text meets WCAG AA
- `prefers-reduced-motion`: respect for continuous animations
