# UI Component Sizing Audit Report
## Meetily Tauri App — Minimum Window Size: 900×600px

### Executive Summary
The Meetily UI has **good responsive design patterns** overall, with most components using flexible layouts and relative sizing. However, several components have **fixed dimensions or layout patterns that risk overflow or poor usability** at the minimum 900×600px window size.

---

## ✅ COMPLIANT COMPONENTS

### Core Layout
- **Root Layout** (`app/layout.tsx`): Uses `flex h-screen overflow-hidden` — correct full-viewport sizing
- **MainContent** (`components/MainContent/index.tsx`): Uses `flex-1` with responsive padding (`p-4 sm:p-5 md:p-6 lg:p-8 xl:p-10`) — adapts well
- **Sidebar** (`components/Sidebar/index.tsx`): Collapsible with `w-16` (collapsed) / `w-64` (expanded), uses `h-screen` correctly
- **SidebarProvider**: Auto-collapses on compact viewports via media query — good responsive behavior

### Pages (General)
- **Home Page** (`app/page.tsx`): Uses `flex flex-col h-screen` with `overflow-hidden` — proper containment
- **Settings Page** (`app/settings/page.tsx`): Uses `flex flex-col` with `overflow-y-auto` content area — scrolls correctly
- **Action Items Page** (`app/action-items/page.tsx`): Uses `flex flex-col h-screen` with sticky header — good
- **Notes Page** (`app/notes/page.tsx`): Uses `flex flex-col` with `overflow-y-auto` — proper

### Modals & Dialogs
- **Dialog Component** (`components/ui/dialog.tsx`): Base `max-w-lg` (512px) — fits comfortably
- **RecordingBlockedModal**: `sm:max-w-md` (448px) — safe
- **UpdateDialog**: `sm:max-w-[500px]` — fits within 900px width
- **ImportAudioDialog**: `sm:max-w-[500px]` — safe
- **RetranscribeDialog**: `sm:max-w-[450px]` — safe
- **Edit Meeting Title Dialog**: `sm:max-w-[425px]` — safe
- **ConfirmationModal**: `max-w-md w-full mx-4` — safe with margin

### UI Primitives
- **Buttons, Inputs, Cards**: All use relative sizing or sensible max-widths
- **Popover/Command**: `max-h-[300px]` for scrollable areas — prevents overflow
- **Select/Dropdowns**: Use `w-full` or reasonable fixed widths (`w-[80px]`, `w-[250px]`)

---

## ⚠️ PROBLEMATIC COMPONENTS & ISSUES

### 1. Sidebar Header Height (CRITICAL)
**File**: `frontend/src/components/Sidebar/index.tsx:705`
```tsx
<div className="flex-shrink-0 h-[88px] flex items-center">
```
**Issue**: Fixed 88px height for sidebar header. At 600px window height, this consumes 14.7% of vertical space.
**Impact**: Combined with footer buttons (~200px), leaves ~312px for meeting list — may truncate with many meetings.
**Recommendation**: Reduce to `h-[64px]` or use `min-h` with flexible content.

### 2. Meeting Details — Two-Panel Layout (CRITICAL)
**File**: `frontend/src/app/meeting-details/page-content.tsx:204-265`
```tsx
<div className="flex flex-1 overflow-hidden w-full">
  <TranscriptPanel ... />   // md:w-1/4 lg:w-1/3
  <SummaryPanel ... />       // flex-1
</div>
```
**TranscriptPanel Width** (`components/MeetingDetails/TranscriptPanel.tsx:138`):
```tsx
<div className="hidden md:flex md:w-1/4 lg:w-1/3 min-w-0 border-r border-gray-200 bg-white flex-col relative shrink-0">
```
**Issue**: At 900px window with 256px sidebar (expanded), remaining width = 644px.
- TranscriptPanel at `lg:w-1/3` = ~215px (too narrow for readable text)
- TranscriptPanel at `md:w-1/4` = ~161px (severely cramped)
- SummaryPanel gets rest, but BlockNote editor needs minimum width for toolbar + content
**Impact**: Transcript text will wrap excessively; editor may feel cramped.
**Recommendation**: 
- Use `w-72` (288px) or `w-80` (320px) fixed width for TranscriptPanel instead of fractions
- Add `min-w-[280px]` to prevent over-squeezing
- Consider making TranscriptPanel collapsible within the meeting view

### 3. Empty State Components (MODERATE)
**Files**: 
- `components/EmptyStateNudge.tsx:29-30`
- `components/EmptyStateSummary.tsx:25`

```tsx
// EmptyStateNudge
<div className="flex flex-col items-center justify-center min-h-[400px] p-8 animate-in fade-in duration-300">
  <div className="max-w-md w-full bg-[var(--card)] rounded-lg border border-[var(--border)] shadow-sm p-8 space-y-6">

// EmptyStateSummary
<div className="flex flex-col items-center justify-center min-h-[300px] p-8 text-center">
```
**Issue**: `min-h-[400px]` and `min-h-[300px]` force vertical space. At 600px window with header/footer chrome (~100px), only ~500px available — empty states consume 60-80% of usable height.
**Impact**: On short windows, empty states dominate the view; scrolling may be required to see action buttons.
**Recommendation**: Reduce to `min-h-[200px]` and `min-h-[150px]` respectively, or use `max-h` with scroll.

### 4. Recording Controls Floating Bar (MODERATE)
**File**: `app/page.tsx:235-259`
```tsx
<div className="fixed bottom-12 left-0 right-0 z-10 px-4">
  <div className={`... ${sidebarCollapsed ? 'ml-16' : 'ml-64'}`}>
    <div className="w-full max-w-3xl lg:max-w-4xl flex justify-center">
```
**Issue**: `max-w-3xl` (768px) / `lg:max-w-4xl` (896px) controls width. At 900px window with expanded sidebar (256px), available width = 644px. The `ml-64` margin pushes content, but `max-w-3xl` still allows up to 768px — could overflow if sidebar is expanded.
**Impact**: Recording controls may extend beyond visible area on narrow windows.
**Recommendation**: Change to `max-w-full` or `max-w-[calc(100%-16rem)]` to respect sidebar width dynamically.

### 5. Settings Page Tabs (MODERATE)
**File**: `app/settings/page.tsx:88-102`
```tsx
<TabsList className="... p-0 h-auto">
  {TABS.map((tab, index) => (
    <TabsTrigger
      className="flex items-center gap-2 px-6 py-4 ... relative z-10"
    >
```
**Issue**: 5 tabs × `px-6` (24px each side = 48px) + label + icon ≈ 80-90px per tab = 400-450px total. At 900px window with sidebar (256px) + MainContent padding (32px each side), available width ≈ 580px. Tabs fit but are tight.
**Impact**: On smaller screens or with longer labels, tabs may wrap or truncate.
**Recommendation**: Use `px-4` instead of `px-6`, or make tabs scrollable with `overflow-x-auto`.

### 6. Onboarding Flow (MODERATE)
**File**: `components/onboarding/OnboardingContainer.tsx:46-47`
```tsx
<div className="fixed inset-0 bg-gray-50 flex items-center justify-center z-50 overflow-hidden">
  <div className={cn('w-full max-w-2xl h-full max-h-screen flex flex-col px-6 py-6', className)}>
```
**Issue**: `max-w-2xl` (672px) is fine for width, but `h-full max-h-screen` with `py-6` padding means content must fit in 600px - 48px padding = 552px. Onboarding steps have multiple sections (progress, title, content, navigation).
**Impact**: DownloadProgressStep and PermissionsStep may require vertical scrolling at 600px.
**Recommendation**: Ensure each step's content is concise; test at 600px height.

### 7. Model Settings Modal — Popover Width (LOW)
**File**: `components/ModelSettingsModal.tsx:781-784`
```tsx
<PopoverContent className="w-[250px] p-0" align="start">
  <Command>
    <CommandList className="max-h-[300px]">
```
**Issue**: Popover width 250px is reasonable, but long model names may truncate. Not a sizing issue per se, but worth noting.
**Recommendation**: Consider `min-w-[250px]` instead of fixed width to allow expansion.

### 8. SummaryPanel — Streaming Preview (LOW)
**File**: `components/MeetingDetails/SummaryPanel.tsx:726`
```tsx
<div className="sticky bottom-0 left-0 right-0 bg-white shadow-lg p-4 max-h-[33vh] overflow-y-auto">
```
**Issue**: `max-h-[33vh]` at 600px = 198px. This is reasonable but could feel cramped with 4 columns of summary cards.
**Impact**: Summary preview may require scrolling within the sticky area.
**Recommendation**: Acceptable as-is; monitor user feedback.

---

## 📐 LAYOUT PATTERNS — WIDTH ANALYSIS

### Available Width at 900px Window
| State | Sidebar | Main Content | Usable Width |
|-------|---------|--------------|--------------|
| Sidebar Expanded | 256px (w-64) | 644px | ~580px (after padding) |
| Sidebar Collapsed | 64px (w-16) | 836px | ~772px (after padding) |

### Available Height at 600px Window
| Component | Height | Remaining |
|-----------|--------|-----------|
| Window | 600px | — |
| Browser chrome (est.) | ~28px | 572px |
| Sidebar header | 88px | 484px |
| Sidebar footer (buttons) | ~200px | 284px |
| MainContent top padding | 32px | 252px |
| MainContent bottom padding | 32px | 220px |

**Conclusion**: Vertical space is tight with expanded sidebar. Collapsed sidebar improves this significantly.

---

## 🔧 RECOMMENDED FIXES (Priority Order)

### High Priority
1. **Reduce Sidebar Header Height** from 88px to 64px → saves 24px vertical space
2. **Fix Meeting Details Two-Panel Layout**:
   - Change TranscriptPanel from `md:w-1/4 lg:w-1/3` to `w-72 md:w-80` (fixed width)
   - Add `min-w-[280px]` to prevent over-squeezing
   - Consider making TranscriptPanel collapsible within the meeting view
3. **Adjust Recording Controls Max-Width**:
   - Change `max-w-3xl lg:max-w-4xl` to `max-w-full md:max-w-3xl` with dynamic margin

### Medium Priority
4. **Reduce Empty State Heights**:
   - EmptyStateNudge: `min-h-[400px]` → `min-h-[200px]`
   - EmptyStateSummary: `min-h-[300px]` → `min-h-[150px]`
5. **Optimize Settings Tabs**:
   - Reduce tab padding from `px-6` to `px-4`
   - Or enable horizontal scroll: `overflow-x-auto`

### Low Priority
6. **Review Onboarding Step Content** — ensure each step fits in ~500px height
7. **Test BlockNote Editor** at narrow widths — may need `min-w` constraint

---

## ✅ SUMMARY

**Overall Assessment**: The app is **mostly compliant** with the 900×600px minimum. The responsive design system (Tailwind) is well-implemented. The primary concerns are:

1. **Vertical space pressure** with expanded sidebar (88px header + footer buttons)
2. **Meeting details two-panel layout** — transcript panel width fractions don't work well at 900px
3. **Empty state components** are too tall for short windows

**None of these are showstoppers**, but addressing the high-priority items will significantly improve usability at the minimum window size.

---

*Audit conducted via code review only — no builds or browser testing performed.*