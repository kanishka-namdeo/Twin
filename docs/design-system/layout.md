# Layout, Animation & Interaction Patterns

## Section Structure

Every major section follows this pattern:

```tsx
<section className="py-32">
  <div className="mx-auto max-w-6xl px-6">
    {/* Section label badge */}
    <div className="mb-8 inline-flex items-center gap-3 rounded-full border border-accent/30 bg-accent/5 px-5 py-2">
      <span className="h-2 w-2 rounded-full bg-accent" />
      <span className="font-mono text-xs uppercase tracking-[0.15em] text-accent">
        Section Name
      </span>
    </div>

    {/* Headline — last key word gets gradient text */}
    <h2 className="font-display text-4xl leading-tight tracking-tight md:text-[3.25rem]">
      Section headline with <span className="gradient-text">highlighted word</span>
    </h2>

    {/* Content */}
    <div className="mt-16 grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
      {/* Cards */}
    </div>
  </div>
</section>
```

## Grid Patterns

| Layout | Grid | Usage |
|:-------|:-----|:------|
| Hero | `grid-cols-[1.1fr_0.9fr]` | Asymmetric text + graphic |
| Features | `grid-cols-1 md:grid-cols-2 lg:grid-cols-3` | Responsive card grid |
| Benefits | `grid-cols-[1.2fr_0.8fr]` | Content-heavy left |
| Stats | `grid-cols-2 md:grid-cols-4` | Metric cards |
| Pricing | `grid-cols-1 md:grid-cols-3` | Center card elevated |

## Inverted Sections

Strategic dark sections for visual rhythm:

```tsx
<section className="bg-[var(--foreground)] text-[var(--background)]">
  <div className="relative">
    {/* Dot pattern texture */}
    <div className="absolute inset-0 opacity-[0.03]" style={{
      backgroundImage: 'radial-gradient(circle, currentColor 1px, transparent 1px)',
      backgroundSize: '32px 32px'
    }} />
    {/* Content */}
  </div>
</section>
```

## Radial Glow Decorations

```tsx
<div className="pointer-events-none absolute -top-32 left-1/2 h-[500px] w-[500px] -translate-x-1/2 rounded-full bg-[var(--accent)] opacity-[0.03] blur-[150px]" />
```

## Animation System

### Entrance Animations (Framer Motion)

```typescript
const easeOut = [0.16, 1, 0.3, 1] as const;

const fadeInUp = {
  hidden: { opacity: 0, y: 28 },
  visible: { opacity: 1, y: 0, transition: { duration: 0.7, ease: easeOut } }
};

const fadeIn = {
  hidden: { opacity: 0 },
  visible: { opacity: 1, transition: { duration: 0.7, ease: easeOut } }
};

const stagger = {
  hidden: {},
  visible: { transition: { staggerChildren: 0.1, delayChildren: 0.1 } }
};
```

### Viewport Options

```typescript
{ once: true, amount: 0.15, margin: "-60px" }
```

### Continuous Animations

| Animation | Duration | Curve | Usage |
|:----------|:---------|:------|:------|
| Rotating ring | `60s` | `linear` | Decorative hero element |
| Floating card | `4-5s` | `ease-in-out` | Hero cards, ±10px bob |
| Pulsing dot | `2s` | `ease-in-out` | Live indicators |
| Activity pulse | `3s` | `ease-in-out` | Status indicators |

### Transition Defaults

| Context | Duration | Easing |
|:--------|:---------|:-------|
| Standard | `200ms` | `ease-out` |
| Entrance | `700ms` | custom easeOut |
| Hover lift | `300ms` | `ease-out` |
| Button active | `200ms` | `ease-out` |

### CSS Keyframes (globals.css)

- `vibrate` — shake effect (0.3s)
- `fade-in` — opacity + translateY (0.4s)
- `fade-in-up` — staggered entrance (0.5s)

## Responsive Strategy

This is a **Tauri desktop application**, not a web app. Viewport responsiveness uses CSS media queries and the `useMediaQuery` hook for state synchronization.

### Viewport Breakpoints

| Breakpoint | Width | Behavior |
|:-----------|:------|:---------|
| Compact | `<900px` | Sidebar auto-collapses to icon-only mode |
| Standard | `900-1199px` | Two-panel layout, standard spacing |
| Comfortable | `1200-1599px` | Expanded content areas, larger max-widths |
| Extended | `≥1600px` | Maximum content width, full feature set |

### Implementation Pattern

The sidebar uses a **user-preference-first** approach:

1. **First visit**: Sidebar auto-collapses on compact viewports (`<900px`), stays expanded on larger viewports
2. **After user toggles**: Their preference is persisted to `localStorage` and respected regardless of viewport
3. **JS sync**: Use `useMediaQuery` hook only for initial state and CSS class synchronization

```typescript
import { useMediaQuery, VIEWPORT_BREAKPOINTS } from '@/hooks/useMediaQuery';

// In SidebarProvider:
const isCompact = useMediaQuery(VIEWPORT_BREAKPOINTS.compact);

// Initialize from localStorage, fallback to viewport
const [isCollapsed, setIsCollapsed] = useState(() => {
  const saved = localStorage.getItem('sidebar-collapsed');
  if (saved !== null) return saved === 'true';
  return window.matchMedia(VIEWPORT_BREAKPOINTS.compact).matches;
});

// Toggle with persistence
const toggleCollapse = () => {
  const newValue = !isCollapsed;
  setIsCollapsed(newValue);
  localStorage.setItem('sidebar-collapsed', String(newValue));
};

// Auto-collapse only on first visit (no saved preference)
useEffect(() => {
  const saved = localStorage.getItem('sidebar-collapsed');
  if (saved === null && isCompact) {
    setIsCollapsed(true);
  }
}, [isCompact]);
```

### Content Width Tokens

Use Tailwind's responsive max-width utilities to scale content with viewport:

```tsx
// Standard pattern: fluid width with breakpoint-specific max-widths
<div className="w-full max-w-3xl lg:max-w-4xl">
  {/* Content adapts: 768px on standard, 896px on large+ */}
</div>
```

| Token | Value | Usage |
|:------|:------|:------|
| `max-w-3xl` | `48rem` (768px) | Transcript content, recording controls |
| `max-w-4xl` | `56rem` (896px) | Same elements on large viewports |
| `max-w-6xl` | `72rem` (1152px) | Settings pages, full-width content |
| `max-w-7xl` | `80rem` (1280px) | Main content wrapper |

### Key Adaptations

- Sidebar: Auto-collapses at `<900px` via `useMediaQuery` hook
- Content padding: `p-4` → `p-6` → `p-8` → `p-10` across breakpoints
- Content max-width: `max-w-3xl` → `max-w-4xl` on large viewports
- Button widths: `w-full sm:w-auto` on mobile
- Touch targets: 44px minimum on all interactive elements

## Asymmetry Patterns

- Hero: `grid-cols-[1.1fr_0.9fr]` — left-heavy for text dominance
- Benefits: `grid-cols-[1.2fr_0.8fr]` — content over visual
- Use negative margins and overlapping elements for Z-depth
- Featured pricing tier floats above siblings

## Textures & Depth

### Dot Pattern (on dark sections)

```css
background-image: radial-gradient(circle, currentColor 1px, transparent 1px);
background-size: 32px 32px;
opacity: 0.03;
```

### Radial Glow (at section corners)

```css
/* Large blurred accent circle */
.blur-[150px] bg-[var(--accent)] opacity-[0.03]
```

### Layered Shadows

Cards use multi-layered shadows for realistic depth, not just a single shadow value.

---

## Desktop App Layout Principles

This is a Tauri desktop application. The following principles govern window management, panel layout, component placement, and resize behavior.

### Window Management

#### Window Configuration (`tauri.conf.json`)

| Property | Value | Rationale |
|:---------|:------|:----------|
| `width` | `1100` | Useful default, not maximized |
| `height` | `700` | Fits comfortably on 1080p with taskbar |
| `minWidth` | `900` | Minimum usable width for sidebar + content |
| `minHeight` | `600` | Minimum effective resolution |
| `resizable` | `true` | Always allow resizing |
| `decorations` | `true` | Use native title bar for platform consistency |

#### Window State Persistence

Save and restore window size/position across sessions using the Tauri window-state plugin or a custom Rust implementation:

- Save on: move, resize, scale-factor change, close
- Persist: logical inner size, logical outer position
- Restore size first (clamped to minimums), then position (only if fits on an available monitor)
- Throttle saves during rapid resize/move events

### Panel Layout (Sidebar + Content)

The app uses a two-panel layout: collapsible sidebar + main content area.

#### Sidebar Rules

- **Resizable**: Use `react-resizable-panels` (shadcn `Resizable` component) for drag-to-resize
- **Min width**: `240px` (enough for meeting titles + icons)
- **Max width**: `400px` (prevents sidebar from consuming too much space)
- **Default width**: `256px` (16rem, current `ml-64`)
- **Collapsed width**: `64px` (4rem, current `ml-16`) — icon-only mode
- **Persistence**: Save sidebar width to `localStorage` via `autoSaveId`
- **Mobile fallback**: At `<768px`, sidebar becomes a Sheet drawer

```tsx
// Preferred pattern (react-resizable-panels)
<PanelGroup direction="horizontal" autoSaveId="app-layout">
  <Panel defaultSize={25} minSize={20} maxSize={35} collapsible>
    <Sidebar />
  </Panel>
  <PanelResizeHandle className="w-px bg-border hover:bg-accent transition-colors" />
  <Panel>
    <MainContent>{children}</MainContent>
  </Panel>
</PanelGroup>
```

#### Content Area Rules

- **Fills remaining space**: `flex-1` or `flex-grow`
- **Scrollable**: Content scrolls vertically within its panel, not the whole window
- **Responsive padding**: `p-4` (mobile) → `p-6` (md) → `p-8` (lg)
- **Max content width**: Use `max-w-6xl` for reading comfort on wide monitors

### Component Placement Rules

#### Fixed vs. Flexible Regions

| Region | Behavior | Rationale |
|:-------|:---------|:----------|
| Title bar | Fixed height, full width | Platform convention |
| Sidebar | Fixed width, full height | Navigation must always be accessible |
| Main content | Flexible, fills remaining space | Scales with window size |
| Toolbars (if any) | Fixed height, full width of content area | Above content, below sidebar top |
| Status indicators | Fixed position (bottom or top-right) | Always visible without scrolling |

#### Content Scaling

As the window grows larger, the content area should show more — not just stretch existing content wider:

- **Lists and tables**: Show more rows, allow columns to expand
- **Card grids**: Add columns at breakpoints (1 → 2 → 3)
- **Text content**: Cap line width at ~65 characters for readability
- **Detail panels**: Consider showing side-by-side detail views at wide widths

#### Placement Conventions

- **Primary actions**: Top-right of content area or top of sidebar
- **Secondary actions**: In overflow menus or context menus
- **Destructive actions**: Require confirmation, never in a toolbar without a guard
- **Navigation**: Left sidebar (current pattern), never buried in a dropdown
- **Search**: Top of sidebar or a global command palette (`Cmd+K`)

### Resize Behavior

#### Breakpoint Strategy (Desktop-Specific)

Unlike web responsive design, desktop windows resize fluidly. Use these thresholds:

| Window Width | Layout Behavior |
|:-------------|:----------------|
| `<900px` | Sidebar collapses to icon-only, content takes full width |
| `900-1200px` | Standard two-panel layout |
| `1200-1600px` | Comfortable layout, more content visible |
| `>1600px` | Consider showing additional panels (detail sidebars, previews) |

#### Resize Rules

- **Debounce resize handlers**: Use `requestAnimationFrame` or a 150ms debounce for layout calculations
- **No hardcoded pixel widths**: Use percentages, `flex`, `grid`, or panel constraints
- **Min sizes on all resizable elements**: Prevent content from becoming unusable
- **Transitions on layout shifts**: `transition-all duration-300` for sidebar collapse/expand
- **No jank**: Avoid layout thrashing — batch DOM reads and writes

#### What NOT to Do

- Never open the window maximized by default — choose a useful default size
- Never use fixed pixel widths for the main layout
- Never let panels resize to zero or below their minimum usable size
- Never hide critical navigation behind a resize-dependent collapse
- Never use `transform: translate` for sidebar sliding — it doesn't trigger reflow (use `margin` or panel library instead)

### Keyboard Shortcuts

Desktop users expect keyboard shortcuts. Register these globally:

| Action | Shortcut | Notes |
|:-------|:---------|:------|
| New meeting | `Cmd/Ctrl+N` | Primary action |
| Search | `Cmd/Ctrl+K` | Command palette or search focus |
| Toggle sidebar | `Cmd/Ctrl+B` | Common desktop pattern |
| Settings | `Cmd/Ctrl+,` | Standard preferences shortcut |
| Close/Cancel | `Escape` | Dismiss dialogs, cancel actions |
| Delete selected | `Delete` / `Backspace` | With confirmation |

### Touch Targets & Accessibility

- Minimum interactive element size: `44px` × `44px`
- Focus rings: `ring-2 ring-[var(--accent)] ring-offset-2`
- All interactive elements must be keyboard-accessible
- Respect `prefers-reduced-motion` for all animations
- Use `aria-label` on icon-only buttons (collapsed sidebar)
