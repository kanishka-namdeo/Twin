# Design Tokens

## Color Palette

### Core Colors

| Token | Light Value | Dark Value | Usage |
|:------|:------------|:-----------|:------|
| `background` | `#FAFAFA` | `#0F172A` | Primary canvas |
| `foreground` | `#0F172A` | `#F8FAFC` | Primary text. Also inverted section backgrounds. |
| `muted` | `#F1F5F9` | `#1E293B` | Secondary surfaces, card backgrounds |
| `muted-foreground` | `#64748B` | `#94A3B8` | Secondary text, descriptions, metadata |
| `card` | `#FFFFFF` | `#0F172A` | Elevated surfaces |
| `card-foreground` | `#0F172A` | `#F8FAFC` | Card text |
| `border` | `#E2E8F0` | `#1E293B` | Structural borders |
| `input` | `#E2E8F0` | `#1E293B` | Input borders |
| `ring` | `#0052FF` | `#4D7CFF` | Focus rings |

### Accent Colors (Signature Gradient)

| Token | Value | Usage |
|:------|:------|:------|
| `accent` | `#0052FF` (Electric Blue) | Primary action color. CTAs, links, highlights. |
| `accent-secondary` | `#4D7CFF` | Gradient endpoint. Used with accent for gradients. |
| `accent-foreground` | `#FFFFFF` | Text on accent backgrounds. Always white. |

### Semantic Colors

| Token | Light Value | Usage |
|:------|:------------|:------|
| `primary` | `#0F172A` | Primary button bg (dark) |
| `primary-foreground` | `#F8FAFC` | Primary button text |
| `secondary` | `#F1F5F9` | Secondary button bg |
| `secondary-foreground` | `#0F172A` | Secondary button text |
| `destructive` | `#DC2626` | Destructive actions |
| `destructive-foreground` | `#FFFFFF` | Text on destructive |

### Chart Colors

| Token | Light Value |
|:------|:------------|
| `chart-1` | `hsl(12, 76%, 61%)` |
| `chart-2` | `hsl(173, 58%, 39%)` |
| `chart-3` | `hsl(197, 37%, 24%)` |
| `chart-4` | `hsl(43, 74%, 66%)` |
| `chart-5` | `hsl(27, 87%, 67%)` |

## The Signature Gradient

```css
background: linear-gradient(to right, var(--accent), var(--accent-secondary));
/* Diagonal: */
background: linear-gradient(135deg, var(--accent), var(--accent-secondary));
```

Appears on: primary buttons, featured badges, icon backgrounds, pricing tier borders, testimonial accent bars, trend indicators, and text highlights.

## Gradient Text Effect

```css
.gradient-text {
  background: linear-gradient(to right, var(--accent), var(--accent-secondary));
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
}
```

## Typography

### Font Pairing

| Role | Font | Source |
|:-----|:-----|:-------|
| Display (h1/h2) | Calistoga | Google Fonts |
| UI & Body | Inter (mapped as `--font-sans`) | Google Fonts |
| Monospace (labels/badges) | JetBrains Mono | Google Fonts |

### Type Scale

| Element | Size | Font | Weight | Tracking | Line Height |
|:--------|:-----|:-----|:-------|:---------|:------------|
| Hero Headline | `text-5xl` / `text-[5.25rem]` | Calistoga | Normal | `-0.02em` | `1.05` |
| Section Headline | `text-3xl` / `text-[3.25rem]` | Calistoga | Normal | Normal | `1.15` |
| Card Title | `text-lg` / `text-2xl` | Inter | Semibold (600) | `-0.01em` | `1.3` |
| Body Text | `text-base` / `text-lg` | Inter | Normal (400) | Normal | `1.625-1.75` |
| Section Label | `text-xs` (12px) | JetBrains Mono | Normal | `0.15em` | Normal |

### Font Loading

In `layout.tsx`:
```typescript
import { Calistoga, Inter, JetBrains_Mono } from 'next/font/google';

const calistoga = Calistoga({ weight: '400', subsets: ['latin'], variable: '--font-display' });
const inter = Inter({ subsets: ['latin'], variable: '--font-sans' });
const jetbrainsMono = JetBrains_Mono({ subsets: ['latin'], variable: '--font-mono' });
```

## Spacing & Layout

| Token | Value | Usage |
|:------|:------|:------|
| Section padding | `py-28` to `py-44` | Major section vertical spacing |
| Container max-width | `max-w-6xl` (72rem) | Primary content width |
| Grid gap | `gap-5` to `gap-8` | Between grid items |
| Card padding | `p-6` to `p-10` | Internal card spacing |

## Border Radius

| Token | Value | Usage |
|:------|:------|:------|
| `--radius` | `0.5rem` | Base radius |
| `rounded-lg` | `0.5rem` | Cards, inputs |
| `rounded-xl` | `0.75rem` | Buttons, elevated cards |
| `rounded-2xl` | `1rem` | Feature cards |

## Shadows

| Token | Value | Usage |
|:------|:------|:------|
| `shadow-sm` | `0 1px 3px rgba(0,0,0,0.06)` | Subtle lift |
| `shadow-md` | `0 4px 6px rgba(0,0,0,0.07)` | Standard cards |
| `shadow-lg` | `0 10px 15px rgba(0,0,0,0.08)` | Elevated cards |
| `shadow-xl` | `0 20px 25px rgba(0,0,0,0.1)` | Hero elements |
| `shadow-accent` | `0 4px 14px rgba(0,82,255,0.25)` | Accent-tinted lift |
| `shadow-accent-lg` | `0 8px 24px rgba(0,82,255,0.35)` | Featured elements |

## Textures

- **Dot Pattern:** `radial-gradient(circle, white 1px, transparent 1px)` at `32px` intervals, `opacity: 0.03` — on dark inverted sections
- **Radial Glows:** Large blurred circles (`blur-[150px]`) of accent color at `3-6%` opacity — at section corners
