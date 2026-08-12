# CapDesk — Theming (light + dark) and the sidebar theme toggle

An addendum to `README.md`. The dashboard now ships **two themes driven by one token set**.
Every color in the UI resolves through a CSS custom property; nothing hardcodes a hex.
Switching themes flips one attribute — `data-theme` on the document root — and the whole app
recolors instantly, with no re-render and no component knowing which theme is active.

Implement it exactly this way. Do **not** fork a second stylesheet or a second component tree.

---

## 1. How the theme switch works

```html
<html data-theme="light">   <!-- or "dark" -->
```

- **Light is the default** — the base `:root` block holds the light values, so a document with
  no `data-theme` attribute renders correctly.
- `:root[data-theme="dark"]` overrides the same names with dark values.
- Set the attribute on the **document root element**, not on a wrapper div — the PDF overlay,
  scrim, and scrollbars all need it.
- **Persist the choice** under the key `capdesk-theme` (values `"light"` / `"dark"`). Read it on
  mount, apply it before first paint, and write on every toggle. In Tauri, prefer the app's own
  settings store over `localStorage` so the choice survives a WebView data reset — and consider
  reading the OS appearance on first run (`window.matchMedia('(prefers-color-scheme: dark)')`
  or Tauri's theme API) as the initial default when nothing is stored.
- **Avoid the flash of light theme:** apply the stored attribute in an inline script in `<head>`,
  before the app bundle mounts.

---

## 2. Token set

48 tokens. Left column is the name; the two value columns are the exact values to ship.

### Surfaces
| Token | Light | Dark |
|---|---|---|
| `--canvas` | `#EDF3EE` | `#0E1411` |
| `--surface` | `#FFFFFF` | `#161D19` |
| `--sunken` | `#FAFCFA` | `#1A221D` |
| `--row` | `#F7FAF7` | `#1A221D` |
| `--viewer` | `#F6F9F6` | `#121815` |
| `--chip` | `#F2F6F2` | `#212A24` |
| `--track` | `#EEF3EE` | `#121915` |
| `--btn` | `#EDF1EE` | `#232C27` |
| `--btn-hover` | `#E3E9E4` | `#2B352E` |
| `--emphasis` | `#E6ECE7` | `#2B352E` |
| `--green-card` | `#EBF6EE` | `#16261D` |
| `--green-tint` | `#E4F2E8` | `#173021` |

### Ink
| Token | Light | Dark |
|---|---|---|
| `--ink` | `#10231A` | `#E7EEE9` |
| `--ink-82` | `rgba(16,35,26,.82)` | `rgba(223,235,227,.82)` |
| `--ink-72` | `rgba(16,35,26,.72)` | `rgba(223,235,227,.72)` |
| `--ink-65` | `rgba(16,35,26,.65)` | `rgba(223,235,227,.65)` |
| `--ink-55` | `rgba(16,35,26,.55)` | `rgba(223,235,227,.55)` |
| `--ink-52` | `rgba(16,35,26,.52)` | `rgba(223,235,227,.52)` |
| `--ink-50` | `rgba(16,35,26,.50)` | `rgba(223,235,227,.50)` |
| `--ink-48` | `rgba(16,35,26,.48)` | `rgba(223,235,227,.48)` |
| `--ink-45` | `rgba(16,35,26,.45)` | `rgba(223,235,227,.45)` |
| `--ink-42` | `rgba(16,35,26,.42)` | `rgba(223,235,227,.42)` |
| `--ink-40` | `rgba(16,35,26,.40)` | `rgba(223,235,227,.40)` |
| `--ink-38` | `rgba(16,35,26,.38)` | `rgba(223,235,227,.38)` |
| `--ink-35` | `rgba(16,35,26,.35)` | `rgba(223,235,227,.35)` |
| `--ink-muted` | `#42604E` | `#B7C8BD` |
| `--ink-muted-2` | `#5B6B60` | `#93A599` |
| `--note-ink` | `#25543A` | `#A9DCBB` |

The `--ink-NN` scale is one ink color at N% opacity. If your styling layer supports color-mix or
an alpha helper, collapse it to a single ink token plus opacities — the numbers are what matter.

### Green (the only hue)
| Token | Light | Dark | Use |
|---|---|---|---|
| `--green` | `#2E9E58` | `#2FA45D` | Approve button, selected card border, approved badge |
| `--green-hover` | `#278A4C` | `#37B96C` | Approve hover |
| `--green-text` | `#1B8F4F` | `#4FCB85` | Green figures, active nav label, status pill text |
| `--green-link` | `#1FA75C` | `#4FCB85` | `a`, "View" links |
| `--green-link-h` | `#17864A` | `#7BE0AB` | Link hover |
| `--row-hover` | `rgba(52,199,89,.10)` | `rgba(79,203,133,.12)` | Document row hover fill |
| `--row-hover-bd` | `rgba(52,199,89,.30)` | `rgba(79,203,133,.34)` | Document row hover border |

Dark keeps the green **desaturated and mid-weight** — it is a UI accent, not a neon. White text
still passes on `--green` in both themes.

### Lines, shadows, misc
| Token | Light | Dark | Use |
|---|---|---|---|
| `--line-07` | `rgba(16,42,26,.07)` | `rgba(255,255,255,.07)` | Standard hairline |
| `--line-08` | `rgba(16,42,26,.08)` | `rgba(255,255,255,.08)` | Field grid, doc rows |
| `--line-09` | `rgba(16,42,26,.09)` | `rgba(255,255,255,.09)` | Deal card border |
| `--line-16` | `rgba(16,42,26,.16)` | `rgba(255,255,255,.16)` | Scrollbar thumb |
| `--line-04` | `rgba(16,42,26,.04)` | `rgba(0,0,0,.34)` | Card shadow |
| `--line-14` | `rgba(16,42,26,.14)` | `rgba(0,0,0,.45)` | Active segment shadow |
| `--line-40` | `rgba(16,42,26,.40)` | `rgba(0,0,0,.70)` | Panel shadow |
| `--line-45` | `rgba(16,42,26,.45)` | `rgba(0,0,0,.75)` | PDF dialog shadow |
| `--line-6` | `rgba(16,42,26,.6)` | `rgba(0,0,0,.75)` | Tooltip shadow |
| `--scrim` | `rgba(20,40,28,.28)` | `rgba(0,0,0,.58)` | PDF overlay scrim |
| `--tooltip-bg` | `#10231A` | `#E7EEE9` | Collapsed-nav tooltip fill |
| `--tooltip-ink` | `#FFFFFF` | `#0E1411` | Collapsed-nav tooltip text |
| `--pdf-filter` | `none` | `invert(1) brightness(1.15)` | See §4 |

**Borders and shadows swap semantics between themes.** In light, both are dark ink at low alpha.
In dark, *borders* become white at low alpha (a lit edge) while *shadows* become black at much
higher alpha. Do not simply lighten the light values — use the table.

**Literal `#FFFFFF` still exists in two places** and must NOT be tokenized: the Approve button's
label and the approved `✓` badge glyph, both of which sit on `--green` in both themes.

---

## 3. Dark theme rules
1. **Same flatness.** No gradients, no glows, no elevation tricks. Dark surfaces step up in
   lightness (`--canvas` → `--surface` → `--chip`) rather than gaining shadows.
2. **Still two colors.** Green plus gray-green neutrals. No red, no amber — Reject, rejected
   badges, and NSF alerts remain neutral in dark exactly as in light.
3. **Watch/high risk tiles** use `--emphasis` with `--ink-muted` initials; `clean` uses
   `--green-tint` with `--green-text`. In dark these must not stay light — a near-white chip on a
   dark card is the single most visible failure mode; verify it.
4. **Layout, spacing, radii, type, and motion are identical between themes.** Dark changes color
   only.

---

## 4. The PDF icon in dark
`assets/pdf-icon.png` is dark line art and disappears on dark surfaces. The prototype inverts it
with a filter: `filter: var(--pdf-filter)` on both `<img>` instances (the 44 × 44 document-row
icon and the 32 × 32 viewer-header icon), where the token is `none` in light and
`invert(1) brightness(1.15)` in dark.

**A shipped app should prefer a real light-ink asset** — export a second PNG (or an SVG using
`currentColor`) and swap the source by theme. The filter is a correct-looking stopgap, not a
long-term answer. `assets/logo.png` reads fine on both themes and needs no treatment.

---

## 5. The sidebar theme toggle
The toggle is **a nav row, not a floating icon** — it must match the other sidebar items exactly.
It sits at the bottom of the nav area, pushed down with `margin-top: auto`, separated from the
reviewer footer by `padding-bottom: 10px` (the footer keeps its own `border-top`).

**Wrapper:** `margin-top: auto; padding-bottom: 10px; display: flex;`
(collapsed adds `justify-content: center`).

**Button:**
```
position: relative; display: flex; align-items: center; gap: 11px;
height: 40px; border-radius: 13px; cursor: pointer;
transition: background .16s ease, color .16s ease;
expanded:  flex: 1; padding: 0 11px;
collapsed: width: 40px; justify-content: center;
idle:      background: transparent; color: var(--ink-52);
hover:     background: var(--chip);  color: var(--ink);
```
These are the same metrics as a nav destination row — 40px tall, 13px radius, 11px gap, a 22px
icon box, and a 13.5px / 500 / `-.008em` label. The only difference is that it has no active
state: it is an action, never a selected destination.

**Icon** — 18px stroked, `stroke-width: 1.9`, round caps/joins, inside a 22 × 22 box:
- Light theme active → **moon** (the action is "go dark").
- Dark theme active → **sun** (circle `r=4` plus eight rays).
Lucide equivalents: `moon` and `sun`.

**Label** — renders only when the sidebar is expanded, and reads the *destination* theme:
**"Dark mode"** while light is active, **"Light mode"** while dark is active. The collapsed
tooltip uses the same string and the same tooltip styling as the nav items
(`left: calc(100% + 12px)`, dark-on-light or light-on-dark per `--tooltip-bg`).

**Behavior** — click flips the theme, writes the preference, and updates `data-theme`. No
transition on the color change: an instant swap reads as deliberate; a crossfade of forty
properties reads as a bug. The sidebar's own width transition is unaffected.

---

## 6. Acceptance checks
- Toggle in both collapsed and expanded sidebar; label and icon swap correctly.
- Reload — the theme persists, with no flash of the wrong theme.
- In dark: no near-white chips (check the watch/high initials tiles), the PDF icon is legible in
  both the document rows and the viewer header, and the collapsed-nav tooltip is light-on-dark.
- In both themes: Approve is white-on-green; Reject and rejected states are neutral; no red or
  amber appears anywhere.
- Grep the built CSS for hex literals outside the token block — the only legal survivors are the
  two `#FFFFFF` text values noted in §2.
