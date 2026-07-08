# Design System

## Purpose

Visual design specification for the Meetily frontend. Defines design tokens (colors, typography, spacing), component patterns, and layout rules to ensure consistency across the application.

## Ownership

This module owns:
- Design token definitions (CSS custom properties)
- Component pattern specifications (buttons, cards, inputs, badges)
- Layout and spacing rules
- Animation and transition guidelines
- Responsive design patterns
- Dark mode support

## Local Contracts

### Documentation Structure

| File | Scope |
|------|-------|
| `tokens.md` | Colors, typography, spacing, shadows, gradients |
| `components.md` | Button, card, input, badge, form patterns |
| `layout.md` | Section structure, grids, animation, responsive rules |

### Color System

#### Core Colors

| Token | Light | Dark | Usage |
|-------|-------|------|-------|
| `background` | `#FAFAFA` | `#0F172A` | Primary canvas |
| `foreground` | `#0F172A` | `#F8FAFC` | Primary text |
| `muted` | `#F1F5F9` | `#1E293B` | Secondary surfaces |
| `muted-foreground` | `#64748B` | `#94A3B8` | Secondary text |
| `card` | `#FFFFFF` | `#0F172A` | Elevated surfaces |
| `border` | `#E2E8F0` | `#1E293B` | Structural borders |
| `ring` | `#0052FF` | `#4D7CFF` | Focus rings |

#### Accent Colors

| Token | Value | Usage |
|-------|-------|-------|
| `accent` | `#0052FF` | Primary action (CTAs, links) |
| `accent-secondary` | `#4D7CFF` | Gradient endpoint |
| `accent-foreground` | `#FFFFFF` | Text on accent |

#### Semantic Colors

| Token | Light | Usage |
|-------|-------|-------|
| `primary` | `#0F172A` | Primary button bg |
| `secondary` | `#F1F5F9` | Secondary button bg |
| `destructive` | `#DC2626` | Destructive actions |

### Signature Gradient

```css
background: linear-gradient(135deg, #0052FF 0%, #4D7CFF 100%);
```

Used for primary CTAs and brand highlights.

### Typography

| Element | Size | Weight | Line Height |
|---------|------|--------|-------------|
| H1 | 30px | 700 | 1.2 |
| H2 | 24px | 600 | 1.3 |
| H3 | 20px | 600 | 1.4 |
| Body | 14px | 400 | 1.5 |
| Small | 12px | 400 | 1.5 |

### Spacing Scale

Base unit: 4px

| Token | Value | Usage |
|-------|-------|-------|
| `spacing-1` | 4px | Tight spacing |
| `spacing-2` | 8px | Icon gaps, inline elements |
| `spacing-3` | 12px | Form field gaps |
| `spacing-4` | 16px | Card padding, section gaps |
| `spacing-5` | 20px | Component margins |
| `spacing-6` | 24px | Section padding |
| `spacing-8` | 32px | Large section gaps |

### Component Patterns

#### Buttons

| Variant | Background | Text | Border | Usage |
|---------|------------|------|--------|-------|
| Default | `accent` | `accent-foreground` | None | Primary actions |
| Secondary | `secondary` | `secondary-foreground` | None | Secondary actions |
| Outline | Transparent | `foreground` | `border` | Tertiary actions |
| Ghost | Transparent | `foreground` | None | Minimal emphasis |
| Destructive | `destructive` | `destructive-foreground` | None | Delete/remove |

#### Cards

- Background: `card`
- Border: `border` (1px)
- Border radius: 8px
- Padding: `spacing-4` (16px)
- Shadow: `0 1px 3px rgba(0, 0, 0, 0.1)`

#### Inputs

- Border: `input` (1px)
- Border radius: 6px
- Padding: `spacing-2` `spacing-3` (8px 12px)
- Focus: `ring` (2px outline)

### Animation

| Token | Duration | Easing | Usage |
|-------|----------|--------|-------|
| `fast` | 150ms | `ease-out` | Hover states, toggles |
| `normal` | 250ms | `ease-in-out` | Modals, dropdowns |
| `slow` | 400ms | `ease-in-out` | Page transitions |

### Responsive Breakpoints

| Name | Width | Usage |
|------|-------|-------|
| `sm` | 640px | Mobile landscape |
| `md` | 768px | Tablet |
| `lg` | 1024px | Desktop |
| `xl` | 1280px | Large desktop |

### Cursor Rules

Design system rules are enforced via Cursor rules:

| Rule File | Scope |
|-----------|-------|
| `design-tokens.mdc` | All frontend files |
| `design-components.mdc` | `components/ui/` |
| `design-layout.mdc` | Pages and feature components |

### Deterministic Core

- Token values (colors, spacing, typography)
- Component pattern specifications
- Animation durations and easings
- Responsive breakpoints

### Non-Deterministic Edges

- Component composition and layout choices
- Icon selection
- Micro-interaction details

## Work Guidance

### Using Tokens

All UI work must use design tokens via CSS custom properties:

```css
background: var(--background);
color: var(--foreground);
border: 1px solid var(--border);
```

### Dark Mode

Dark mode is automatic via `prefers-color-scheme`. Token values switch automatically.

### Component Implementation

Use shadcn/ui components as the base. Extend with design tokens:

```tsx
<Button className="bg-accent hover:bg-accent/90">
  Click me
</Button>
```

### Integration Points

- Frontend components in `frontend/src/components/`
- shadcn/ui components in `frontend/src/components/ui/`
- Global styles in `frontend/src/app/globals.css`

## Verification

```bash
# Check design token usage
cd frontend
pnpm run dev

# Visual inspection in browser
# Open http://localhost:3118
```

## Child DOX Index

This module has no child docs.
