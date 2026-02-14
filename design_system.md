# Vectorless Design System

Reference for the Vectorless desktop application UI. All values are sourced from `src/styles/tokens.css` and `src/styles/base.css`.

---

## 1. Color Palette

### Backgrounds

| Token | Value | Usage |
|-------|-------|-------|
| `--bg-0` | `#070b16` | Deepest background (body fallback) |
| `--bg-1` | `#0d1324` | Scrollbar track, secondary surfaces |
| `--bg-2` | `#111c35` | Elevated surface |
| `--bg-3` | `#15244a` | Highest elevation surface |
| `--bg-card` | `rgba(10, 20, 38, 0.95)` | Card/panel backgrounds |
| `--bg-input` | `rgba(10, 20, 40, 0.92)` | Form input backgrounds |
| `--bg-code` | `rgba(8, 16, 31, 0.95)` | Code snippet backgrounds |
| `--bg-pane` | `linear-gradient(180deg, rgba(12,20,39,0.97), rgba(7,12,25,0.98))` | Pane gradient |

**Body background:** Radial gradient glow + dark linear gradient:
```css
radial-gradient(1200px 560px at 82% -20%, rgba(46, 103, 255, 0.2), transparent 55%),
linear-gradient(160deg, #030812, #060f20 58%, #02060d)
```

### Lines & Borders

| Token | Value | Usage |
|-------|-------|-------|
| `--line` | `#1e2f5f` | Scrollbar thumb, hard dividers |
| `--line-soft` | `rgba(61, 105, 203, 0.35)` | Soft dividers, graph edges |
| `--border-pane` | `rgba(58, 99, 196, 0.4)` | Pane container borders |
| `--border-card` | `rgba(78, 125, 232, 0.32)` | Card borders |
| `--border-input` | `rgba(63, 109, 212, 0.4)` | Input field borders |
| `--border-focus` | `rgba(72, 129, 234, 0.48)` | Focus section borders |

### Text

| Token | Value | Usage |
|-------|-------|-------|
| `--text-0` | `#f4f8ff` | Primary text (body, paragraphs) |
| `--text-1` | `#b8c8e9` | Secondary text (descriptions, labels) |
| `--text-2` | `#84a0d6` | Muted text (placeholders, meta) |
| `--text-heading` | `#edf5ff` | Heading text, query display |
| `--text-subtle` | `#dcecff` | Node titles, softer headings |
| `--text-link` | `#9fc2ff` | Pane heading labels, links |
| `--text-kicker` | `#91bdff` | Kicker labels (uppercase tags) |
| `--text-code` | `#9bc5ff` | Code/snippet text |
| `--text-meta` | `#89b9ff` | Trace meta, timestamps |

### Accents

| Token | Value | Usage |
|-------|-------|-------|
| `--accent-0` | `#2c8dff` | Primary blue accent |
| `--accent-1` | `#48c2ff` | Cyan accent, focus rings |
| `--accent-2` | `#70f5ff` | Teal accent, highlights |
| `--accent-purple` | `#9e60ff` | Retrieval step accent |
| `--accent-purple-soft` | `rgba(158, 96, 255, 0.58)` | Retrieval card borders |

### Status

| Token | Value | Usage |
|-------|-------|-------|
| `--ok` | `#32d296` | Success, grounded answers |
| `--warn` | `#ffb454` | Warnings, medium confidence |
| `--danger` | `#ff5f77` | Errors, failed runs |

### Step & Embedding Colors

| Token | Value | Usage |
|-------|-------|-------|
| `--step-scan` | `#32d296` | Scan step indicator dot |
| `--step-navigate` | `#ffb454` | Navigate/drill step indicator dot |
| `--step-retrieve` | `#9e60ff` | Retrieval step indicator dot |
| `--embed-dot` | `rgba(44, 141, 255, 0.6)` | Embedding scatter plot background dots |
| `--embed-match` | `rgba(50, 210, 150, 0.8)` | Embedding scatter plot matched dots |
| `--bg-sidebar` | `rgba(8, 14, 30, 0.98)` | Graph insights sidebar background |

### Logo Glow

| Token | Value | Usage |
|-------|-------|-------|
| `--logo-glow` | `rgba(88, 206, 255, 0.7)` | Logo dot box-shadow glow |

---

## 2. Typography

### Font Stacks

| Token | Value | Usage |
|-------|-------|-------|
| `--font-sans` | `"Plus Jakarta Sans", "Segoe UI Variable", system-ui, sans-serif` | All UI text |
| `--font-mono` | `"JetBrains Mono", "Cascadia Code", Consolas, monospace` | Code snippets, node IDs |

### Type Scale

| Size | Usage |
|------|-------|
| `10px` | Uppercase labels, node type badges, step indices |
| `11px` | Chips, meta text, footer counts, search inputs, status bar |
| `12px` | Breadcrumbs, table cells, code snippets |
| `13px` | Body text, form inputs, settings labels |
| `15px` | Settings section headings |
| `16px` | Answer card heading |
| `18px` | App title (h1) |
| `22px` | Settings card heading, reader subtitle |
| `24px` | Focus section heading (responsive) |
| `28px` | Reader section heading |
| `30px` | Query card text |
| `42px` | Document preview hero title |

### Letter Spacing

| Value | Usage |
|-------|-------|
| `0.01em` | Buttons |
| `0.08em` | Chips, trace meta |
| `0.09em` | Node type labels |
| `0.12em` | Query kicker labels |
| `0.14em` | Pane heading labels |

### Font Weights

| Weight | Usage |
|--------|-------|
| `400` (normal) | Body text, descriptions |
| `600` (semi-bold) | Buttons, strong labels |
| `700` (bold) | Headings (browser default for h1-h3) |

---

## 3. Spacing

### Layout Grid

| Context | Gap |
|---------|-----|
| App shell rows | `8px` |
| Workspace columns | `8px` |
| Within panes | `10-12px` |
| Settings card sections | `16px` |

### Padding Scale

| Size | Value | Usage |
|------|-------|-------|
| Compact | `4-6px` | Chips, small buttons |
| Default | `7-12px` | Inputs, cards, pane headers |
| Spacious | `14-18px` | Settings cards, reader sections |

### Indent

Tree nodes are indented by `depth * 16px` plus a `12px` base padding.

---

## 4. Border Radius

| Token | Value | Usage |
|-------|-------|-------|
| `--radius-sm` | `10px` | Inputs, buttons, cards, tree nodes |
| `--radius-md` | `14px` | (available for medium elements) |
| `--radius-lg` | `18px` | (available for large containers) |
| Pane | `12px` | Pane containers, answer cards, settings |
| Pill | `999px` | Chips, logo dot, reader rule |
| Small card | `8px` | Breadcrumbs, code snippets, tree nodes |

---

## 5. Shadows

| Token | Value | Usage |
|-------|-------|-------|
| `--shadow-1` | `0 10px 30px rgba(8, 14, 29, 0.55)` | General drop shadow |
| `--shadow-2` | `0 0 0 1px rgba(73, 133, 255, 0.2), 0 18px 32px rgba(7, 12, 24, 0.7)` | Elevated containers with ring |
| Logo glow | `0 0 12px rgba(88, 206, 255, 0.7)` | Logo dot glow effect |

---

## 6. Component Patterns

### Pane

The 3-pane layout uses a consistent pane component:

```
┌─────────────────────────┐
│ HEADER (title + action) │  border-bottom
├─────────────────────────┤
│                         │
│  CONTENT (scrollable)   │
│                         │
├─────────────────────────┤
│ FOOTER (counts/meta)    │  border-top
└─────────────────────────┘
```

- Border: `1px solid var(--border-pane)`
- Border radius: `12px`
- Background: `var(--bg-pane)` (gradient)
- Grid: `auto 1fr auto` rows

### Card

Cards are used for trace steps, query display, answer, and settings sections:

- Border: `1px solid var(--border-card)`
- Border radius: `10-12px`
- Background: `var(--bg-card)`
- Padding: `9-12px`

### Chip

Small pill-shaped status indicators:

- Border: `1px solid rgba(82, 168, 255, 0.5)`
- Border radius: `999px`
- Padding: `4px 8px`
- Font size: `10px`
- Text transform: `uppercase`
- Letter spacing: `0.08em`

### Button (Primary)

- Background: `linear-gradient(180deg, #2f88ff, #2155f2)`
- Border: `1px solid transparent`
- Border radius: `10px`
- Padding: `9px 13px`
- Font weight: `600`
- Hover: `translateY(-1px)` + `brightness(1.06)`
- Disabled: `saturate(0.5)`, no transform

### Button (Ghost/Link)

- Background: `rgba(13, 29, 58, 0.88)`
- Border: `1px solid rgba(78, 136, 238, 0.45)`
- Padding: `6px 9px`
- Font size: `11px`

### Input

- Background: `var(--bg-input)`
- Border: `1px solid var(--border-input)`
- Border radius: `10px`
- Padding: `9px 11px`
- Color: `var(--text-0)`

### Timeline

Vertical gradient line connecting trace step cards:

- Width: `2px`
- Gradient: `blue 20% → purple 45% → cyan 20%`
- Step dots: `9px` circles with `2px` border
- Retrieval dots: filled purple

### Table (Extracted Data)

- Full width, collapsed borders
- Header: dark bg `rgba(20, 35, 67, 0.95)`, text `var(--text-link)`
- Cells: `rgba(10, 18, 35, 0.95)` bg, `8px` padding
- Highlighted cells: accent-colored background

### Embedding Proximity

SVG scatter plot showing semantic proximity between query and document nodes:

- Container: `.embedding-proximity` with border-top separator
- Header: "EMBEDDING SPACE PROXIMITY" kicker + "Match: X.XXX" badge
- SVG viewport: `300 x 150`, responsive via `viewBox`
- Background dots: `var(--embed-dot)`, radius `2.5px`, low opacity
- Matched dots: `var(--embed-match)`, radius `4px`, high opacity
- Matched nodes cluster toward center; others scatter randomly
- Score computed via bag-of-words overlap between query text and node text

### Graph Insights Sidebar

Shown when graph view mode is "Global Cluster":

- Background: `var(--bg-sidebar)`
- Width: `280px` in a `1fr 280px` grid layout
- Sections: stats cards, cross-document synthesis, top entity links, recent explorations
- Entity bars: colored progress bars (danger/default/ok variants)
- Action buttons: "Generate Global Report", "Export Graph (JSONL)"

### Node Info Cards

Side-by-side info cards in the Selected Context panel:

- Layout: flex row with `1fr 1fr` sizing
- Each card: `var(--bg-code)` background, `var(--border-card)` border
- Label: uppercase kicker text (`NODE ID`, `TOKENS`)
- Value: monospace text, truncated with ellipsis

### Timing Badge

Pill-shaped latency badge on trace step cards:

- Background: `rgba(44, 141, 255, 0.12)`
- Border: `1px solid rgba(44, 141, 255, 0.25)`
- Border radius: `999px`
- Font: monospace, `10px`

### Run Summary

Displayed after a reasoning run completes:

- Top row: SUCCESS/FAILED badge + run ID
- Metrics row: 3 cards (Latency, Tokens, Cost) in a `1fr 1fr 1fr` grid
- Metric cards: `var(--bg-code)` background, `var(--border-card)` border

### Query Bar

Redesigned input area at bottom of TracePane:

- Input row: flex with paperclip attach button, textarea, and blue send button
- Send button: circular, `32px`, gradient blue background
- Action buttons row: "Explain Selection" + "Expand Context" as ghost buttons below

---

## 7. Layout & Breakpoints

### Desktop (default)

```
┌──────────────────────────────────────────────────┐
│  UTILITY BAR (logo, doc selector, upload, ⚙)     │
├────────┬───────────────────────┬─────────────────┤
│ TREE   │  TRACE / GRAPH        │  DOCUMENT       │
│ 280px  │  minmax(520px, 1fr)   │  430px          │
│        │                       │                 │
│        │                       │                 │
├────────┴───────────────────────┴─────────────────┤
│  STATUS BAR                                       │
└──────────────────────────────────────────────────┘
```

### Breakpoints

| Width | Changes |
|-------|---------|
| `<= 1420px` | Right pane: `430px → 380px`; reduce heading sizes |
| `<= 1220px` | Single-column stack; header controls stack vertically |

### Window

- Default size: `1460 x 920px`
- Minimum size: `1120 x 760px`

---

## 8. Iconography

### Tree Node Icons

| Type | Icon | Color |
|------|------|-------|
| Document | 📄 | `var(--accent-1)` |
| Section | 📂 | `var(--accent-0)` |
| Subsection | 📁 | `var(--accent-0)` |
| Table | 📊 | `var(--accent-1)` |
| Figure | 🖼 | `var(--accent-2)` |
| Equation | ∑ | `var(--accent-purple)` |
| Paragraph | ¶ | `var(--text-2)` |
| Claim | ★ | `var(--text-2)` |
| Caption | — | `var(--text-2)` |
| Reference | ↗ | `var(--text-2)` |

### Status Icons

| Context | Icon |
|---------|------|
| Success / Grounded | Green circle checkmark (✓) |
| Error / Failed | Red circle cross |
| Settings | ⚙ |
| Upload | Upload button text |
| Helpful | Thumbs up (👍) |
| Copy | Clipboard text |

---

## 9. Animation & Transitions

| Property | Duration | Easing | Usage |
|----------|----------|--------|-------|
| `filter, transform` | `120ms` | `ease` | Button hover lift |
| Timeline entry | (static) | — | No animation yet |

---

## 10. Accessibility

- Focus ring: `2px solid var(--accent-1)` with `2px` offset
- All interactive elements should be `:focus-visible` styled
- Keyboard navigation: Tab order follows visual layout
- Color contrast: Text tokens meet WCAG AA on dark backgrounds
- ARIA labels on icon-only buttons (settings, close, etc.)
