# Handoff: Submission Desk — Deal Approval Dashboard

## Overview
A dark, "liquid glass" desktop dashboard for an auto-submission app used by a
brokerage back office. Two reviewers — **Sanit** and **Careem** — work a queue of
newly submitted merchant funding deals and decide, one at a time, whether each
deal is **approved** (packet auto-submits to the matched lenders) or **rejected**.

The screen answers three questions at a glance:
1. What came in and hasn't been decided yet?
2. Everything about the selected deal, including the PDFs in its payload.
3. One decisive action: Approve & auto-submit, or Reject.

## About the Design Files
The files in this bundle are **design references authored in HTML** — a working
prototype that shows the intended look, layout, and behavior. They are **not
production code to copy**. `Conduit Dashboard.dc.html` uses a proprietary
streaming-template runtime (`support.js`, `<sc-for>`, `<sc-if>`, `{{ }}` holes);
do not port that runtime.

The task is to **recreate this design in the target app: a Rust + Tauri desktop
application.** Read the HTML for exact pixel values, colors, and copy; write idiomatic
code for the codebase's frontend framework and styling approach.

### Target: Rust + Tauri
- **Backend (Rust):** deal list, deal detail, document fetch, seasonality image, and the
  approve/reject mutations belong in `#[tauri::command]` handlers; the frontend calls them
  through `invoke`. Run the lender auto-submission job on the Rust side (async, off the UI
  thread) and report back with a Tauri event (`emit`/`listen`) so the dashboard updates the
  "Auto-submitted" counter and the decision stamp without polling.
- **PDFs:** don't stream bytes through `invoke`. Serve the file to the WebView via a custom
  protocol or the `asset:` scope and render with PDF.js inside the overlay (closest match to
  the design), or hand off to the OS viewer with `tauri-plugin-opener`.
- **Window:** `"decorations": false` plus a custom drag region gives the frameless look this
  design implies. Optionally `"transparent": true` + `windowEffects` (`acrylic`/`mica` on
  Windows, `hudWindow`/`underWindowBackground` on macOS) so the glass picks up the real
  desktop behind it; the CSS recipe below is sufficient on its own.
- **`backdrop-filter` caveat — test this first.** WebKit (macOS) and WebView2 (Windows) both
  support it; **WebKitGTK on Linux does not reliably.** Ship a fallback that swaps the blur for
  a solid `rgba(18,20,26,.92)` fill under `@supports not (backdrop-filter: blur(1px))`. The
  entire look rests on this one property.
- **Set `"minWidth": 1280, "minHeight": 860`** in `tauri.conf.json`; the layout is designed at
  1440 × 980 and breaks below 1280 wide.
- Keep the state table below in the frontend; treat Rust as the source of truth and reconcile
  after each mutation.

Open `Conduit Dashboard.dc.html` in a browser to see and click the prototype.

## Fidelity
**High fidelity.** Colors, type sizes, radii, spacing, and copy are final and
should be matched closely. Everything in the "Design Tokens" section below is
exact. Data is realistic placeholder content — replace with real payload data.

---

## Screens / Views

### 1. Submission Desk (the only screen)
**Purpose:** review the incoming deal queue and decide each deal.

**Frame:** fixed design width **1440 × 980 px**, background `#06070a`.
In the real app this becomes the app window; the layout should stretch
horizontally (the deal list column is fluid) with a **minimum width of 1280 px**.
The right review panel stays a fixed **470 px**.

**Root layout:** CSS grid, `grid-template-columns: 80px 1fr`, full height.

**Ambient background (behind everything, `pointer-events: none`):**
- One blurred teal orb: 760 × 760, `left: 420px; bottom: -460px`, `border-radius: 50%`,
  `background: radial-gradient(circle at 50% 50%, rgba(50,215,180,.18), rgba(50,215,180,0) 68%)`,
  `filter: blur(26px)`, animated by the `floaty` keyframes (27s, ease-in-out, infinite).
- A vignette on top: `radial-gradient(130% 95% at 50% -5%, rgba(0,0,0,0) 28%, rgba(3,4,6,.80) 100%)`.
- `floaty` keyframes: `0% translate3d(0,0,0) scale(1)` → `50% translate3d(0,-26px,0) scale(1.05)` → `100%` back to start.

---

### Component: Left nav rail
- Container: `width: 64px`, `margin: 16px 0 16px 16px`, `border-radius: 24px`,
  column flex, `align-items: center`, `gap: 22px`, `padding: 22px 0 20px`.
- **Glass recipe (see Design Tokens → Glass):** background
  `linear-gradient(160deg, rgba(255,255,255,.085), rgba(255,255,255,.025))`,
  `backdrop-filter: blur(34px) saturate(180%)`, `border: 1px solid rgba(255,255,255,.10)`,
  `box-shadow: inset 0 1px 0 rgba(255,255,255,.16), 0 24px 60px -24px rgba(0,0,0,.9)`.
- **Logo tile:** 34 × 34, `border-radius: 12px`, `linear-gradient(150deg,#a9c0ff,#6d5cff)`,
  glyph "C", `font-weight: 700`, `font-size: 15px`, `color: #0a0b12`,
  `box-shadow: 0 6px 18px -6px rgba(110,120,255,.85), inset 0 1px 0 rgba(255,255,255,.55)`.
- **Home item (active):** 38 × 38, `border-radius: 13px`,
  `background: linear-gradient(160deg, rgba(255,255,255,.92), rgba(255,255,255,.66))`,
  icon color `#0a0b12`, 17 × 17 stroked house icon (`stroke-width: 2`, round caps/joins).
  Inactive nav items (none present today) use `color: rgba(238,240,244,.42)` with
  `border: 1px solid rgba(255,255,255,.06)` and no fill.
- Additional nav destinations were intentionally removed for v1 — add them in this
  same style when the app has more sections.

### Component: Page header
Row, `align-items: center`, `gap: 16px`, inside `<main>` (`padding: 20px 24px 22px 18px`, column flex, `gap: 16px`).
- **Eyebrow:** "Submission desk" — 10.5px, `font-weight: 600`, `letter-spacing: .16em`,
  `text-transform: uppercase`, `color: rgba(238,240,244,.40)`.
- **Title (H1):** 26px, `font-weight: 600`, `letter-spacing: -.022em`, `line-height: 1.1`.
  Text is **derived from the active tab**: `Awaiting decision` / `Submitted deals` / `Rejected submissions`.
- **Search field (right):** 280 × 38, `border-radius: 19px`, `background: rgba(255,255,255,.055)`,
  `backdrop-filter: blur(24px) saturate(160%)`, `border: 1px solid rgba(255,255,255,.09)`,
  `box-shadow: inset 0 1px 0 rgba(255,255,255,.12)`. Contains a `⌕` glyph at `opacity: .4`,
  placeholder "Search deals, pullers, lenders" (13px, `rgba(238,240,244,.40)`), and a
  `⌘K` key hint (10px monospace, `rgba(238,240,244,.32)`, `border: 1px solid rgba(255,255,255,.12)`,
  `border-radius: 5px`, `padding: 2px 5px`). Search is **not wired up** in the prototype —
  implement as a filter across company / puller / lender.

### Component: Stat cards
Grid, `grid-template-columns: repeat(4, 1fr)`, `gap: 12px` — currently **two cards occupy the first two cells**.
Each card: `padding: 14px 16px`, `border-radius: 20px`, glass recipe with
`backdrop-filter: blur(30px) saturate(170%)` and `box-shadow: inset 0 1px 0 rgba(255,255,255,.15), 0 20px 44px -28px rgba(0,0,0,.9)`.
- Label: 10.5px, uppercase, `letter-spacing: .13em`, `font-weight: 600`, `rgba(238,240,244,.42)`.
- Value: 30px, `font-weight: 600`, `letter-spacing: -.03em`; unit text 11.5px `rgba(238,240,244,.45)`, baseline-aligned, `gap: 8px`.
- **In queue** → count of deals with status `awaiting`, unit "deals".
- **Auto-submitted** → running count of packets sent, unit "packets sent".

### Component: Deal queue (left column)
Container: glass panel, `border-radius: 26px`, `backdrop-filter: blur(34px) saturate(175%)`,
`box-shadow: inset 0 1px 0 rgba(255,255,255,.15), 0 34px 70px -34px rgba(0,0,0,.95)`,
column flex, `overflow: hidden`. Outer grid: `grid-template-columns: minmax(0,1fr) 470px; gap: 16px`.

**Tab bar** — `padding: 14px 16px 12px`, `border-bottom: 1px solid rgba(255,255,255,.07)`.
Segmented control: wrapper `padding: 3px`, `gap: 2px`, `border-radius: 14px`,
`background: rgba(255,255,255,.05)`, `border: 1px solid rgba(255,255,255,.07)`.
Three tabs, each `height: 28px`, `padding: 0 13px`, `border-radius: 11px`, 12.5px, `white-space: nowrap`:
- **Awaiting N** · **Approved N** · **Rejected N** (the count follows the label with two spaces).
- Active: `font-weight: 600`, `color: #0a0b12`,
  `background: linear-gradient(160deg, rgba(255,255,255,.95), rgba(255,255,255,.72))`,
  `box-shadow: 0 6px 16px -8px rgba(255,255,255,.5)`.
- Inactive: `font-weight: 500`, `color: rgba(238,240,244,.52)`, no background.

**Scroll area:** `flex: 1`, `overflow-y: auto`, `padding: 10px 12px 14px`, column flex, `gap: 9px`.

**Deal card** — `padding: 13px 14px`, `border-radius: 20px`, `cursor: pointer`,
`transition: transform .18s ease, background .18s ease`.
- Default: `background: rgba(255,255,255,.045)`, `border: 1px solid rgba(255,255,255,.07)`.
- Selected: `background: linear-gradient(160deg, rgba(157,180,255,.20), rgba(255,255,255,.05))`,
  `border: 1px solid rgba(157,180,255,.34)`,
  `box-shadow: inset 0 1px 0 rgba(255,255,255,.22), 0 16px 36px -22px rgba(90,120,255,.9)`.

Card row 1 (`display: flex; gap: 12px; align-items: flex-start`):
- **Initials tile** 38 × 38, `border-radius: 13px`, 12.5px `font-weight: 700`. Its color is
  tinted by the deal's internal risk band (see State Management → risk):
  clean `#8ff0d4` on `rgba(78,224,181,.16)` / border `rgba(78,224,181,.30)`;
  watch `#ffd79a` on `rgba(255,199,102,.15)` / border `rgba(255,199,102,.28)`;
  high `#ffb3bd` on `rgba(255,107,129,.16)` / border `rgba(255,107,129,.30)`.
  **Note:** the text badges ("Clean file / Watch / High risk") were deliberately removed —
  the tint is the only remaining risk signal. Do not reintroduce the badges.
- **Company name** 15px `font-weight: 600`, `letter-spacing: -.012em`, truncates with ellipsis.
- **Meta line** 11.5px `rgba(238,240,244,.45)`, dot-separated (`·` at `opacity: .35`, `gap: 7px`):
  industry · state · "{time in business} in business" · "Position {n}".
- **Right rail:** requested amount 16px `font-weight: 600` `letter-spacing: -.02em`;
  below it, relative time 11px `rgba(238,240,244,.40)`.

Card row 2 (`margin-top: 11px`, `display: flex; gap: 8px; align-items: center`) — chips are
`height: 26px`, `padding: 0 10px`, `border-radius: 13px`, 11.5px:
- `FICO {n}`, `ADB {value}`, `{n} lenders matched` — neutral chips:
  `background: rgba(255,255,255,.055)`, `border: 1px solid rgba(255,255,255,.07)`,
  `color: rgba(238,240,244,.72)`; the leading label word is dimmed to `rgba(238,240,244,.40)`.
- `NSF {n}` — same neutral chip, **but** when `nsf >= 5` it turns red:
  `color: #ffb3bd`, `background: rgba(255,107,129,.14)`, `border: 1px solid rgba(255,107,129,.26)`.
- **Far right (decided deals only):** label 11px `rgba(238,240,244,.38)` reading
  **"Approved by Sanit"** / **"Approved by Careem"** / **"Rejected by …"**, followed by a
  24 px round badge: `✓` on `linear-gradient(150deg,#8ff0d4,#22b894)` (`color: #04120e`) for
  approved, `✕` on `linear-gradient(150deg,#ffb3bd,#ff6b81)` (`color: #2a0810`) for rejected.
  **Awaiting deals show nothing here** (no "claimed by" state in v1).

**Empty state** (when the active tab has no deals): `padding: 56px 20px`, centered,
title 15px `font-weight: 600` `rgba(238,240,244,.72)`, body 12.5px `rgba(238,240,244,.42)`:
- Awaiting → "Queue cleared" / "Every submission has a decision. New deals land here automatically."
- Approved → "Nothing submitted yet" / "Approved deals move here once the packet goes out to lenders."
- Rejected → "Nothing rejected yet" / "Rejected deals are kept here with the reviewer who declined them."

### Component: Review panel (right column, 470 px)
Glass panel, `border-radius: 26px`, slightly brighter than the queue:
`linear-gradient(165deg, rgba(255,255,255,.10), rgba(255,255,255,.028))`,
`backdrop-filter: blur(38px) saturate(180%)`, `border: 1px solid rgba(255,255,255,.12)`,
`box-shadow: inset 0 1px 0 rgba(255,255,255,.18), 0 34px 70px -34px rgba(0,0,0,.95)`.
Three stacked regions: fixed header, scrolling body, fixed action footer.

**Header** — `padding: 18px 20px 14px`, `border-bottom: 1px solid rgba(255,255,255,.07)`.
- **Eyebrow**, 10px, `font-weight: 700`, `letter-spacing: .16em`, uppercase; text and color
  derive from the selected deal's status:
  awaiting → "NEW DEAL SUBMITTED" `#9db4ff`; approved → "SUBMITTED TO LENDERS" `#8ff0d4`;
  rejected → "REJECTED" `#ffb3bd`.
- Right of it: "Submitted MM/DD/YYYY hh:mm AM" 11px `rgba(238,240,244,.38)`.
- **Company (H2)** 23px `font-weight: 600` `letter-spacing: -.024em` `line-height: 1.15`, `margin-top: 7px`.
- **Chips row** (`margin-top: 9px`, `gap: 8px`), each `height: 28px`, `padding: 0 11px`, `border-radius: 14px`, 12px:
  - "{amount} requested" — `background: linear-gradient(150deg, rgba(157,180,255,.24), rgba(109,92,255,.14))`,
    `border: 1px solid rgba(157,180,255,.28)`, `font-weight: 600`.
  - "Position {n}" — `background: rgba(255,255,255,.06)`, `border: 1px solid rgba(255,255,255,.08)`,
    `color: rgba(238,240,244,.70)`.

**Body** — `flex: 1`, `overflow-y: auto`, `padding: 16px 20px 18px`, column flex, `gap: 18px`.
Every section starts with a label: 10.5px, uppercase, `letter-spacing: .14em`,
`font-weight: 600`, `color: rgba(238,240,244,.40)`, `margin-bottom: 9–10px`.

1. **DEAL INFORMATION** — two-column grid of hairline-separated cells:
   `display: grid; grid-template-columns: 1fr 1fr; gap: 1px;`
   `background: rgba(255,255,255,.07)` (that 1px gap *is* the divider),
   `border: 1px solid rgba(255,255,255,.07)`, `border-radius: 16px`, `overflow: hidden`.
   Each cell: `background: rgba(12,14,19,.42)`, `padding: 9px 12px`, column flex, `gap: 2px`.
   Key 10.5px uppercase `letter-spacing: .05em` `font-weight: 600` `rgba(238,240,244,.38)`;
   value 13px `rgba(238,240,244,.86)`, truncating.
   **Field order (reading left→right, matching the source submission sheet):**
   Company · Lead source · Email · Credit score · Phone · Industry · Puller · Revenue ·
   Re-puller · Avg daily balance · State · Deposits · Time in business · NSFs.
   **Value color exceptions:** email renders as a link `#9db4ff`; `N/A` and `None` dim to
   `rgba(238,240,244,.35)`; credit score below 620 and NSFs ≥ 5 turn `#ffb3bd`.

2. **DOCUMENTS IN PAYLOAD** — collapsible (default open). Header row is clickable:
   label, then a count pill (`height: 19px`, `min-width: 19px`, `padding: 0 6px`,
   `border-radius: 7px`, 10.5px `font-weight: 700`, `color: #c8d4ff`,
   `background: rgba(157,180,255,.18)`, `border: 1px solid rgba(157,180,255,.26)`),
   then right-aligned "{n} pages" (11px `rgba(238,240,244,.38)`) and a caret `▾`/`▸`.
   Rows: column flex `gap: 6px`; each row `display: flex; gap: 11px; align-items: center;`
   `padding: 9px 11px`, `border-radius: 13px`, `background: rgba(255,255,255,.045)`,
   `border: 1px solid rgba(255,255,255,.07)`, `cursor: pointer`.
   Hover: `background: rgba(157,180,255,.14)`, `border-color: rgba(157,180,255,.28)`.
   Row contents: a 30 × 34 "PDF" tile (`border-radius: 6px`, 8.5px `font-weight: 700`,
   `color: #ffb3bd`, `background: linear-gradient(160deg, rgba(255,107,129,.20), rgba(255,107,129,.07))`,
   `border: 1px solid rgba(255,107,129,.26)`), the file name (12.5px `font-weight: 500`, truncating),
   the meta line under it ("4 pages · 2.1 MB", 11px `rgba(238,240,244,.40)`), and a
   right-aligned "View" affordance (11.5px `rgba(238,240,244,.38)`).
   Documents come from the submission payload — this list must be data-driven.

3. **SEASONALITY · MONTHLY DEPOSITS** — label with right-aligned "trailing 12"
   (11px `rgba(238,240,244,.38)`). Body is a **150 px tall image frame**:
   `border-radius: 16px`, `background: rgba(12,14,19,.42)`, `border: 1px solid rgba(255,255,255,.07)`,
   `overflow: hidden`. In production this renders the **seasonality breakdown PNG that arrives
   with the payload**, contained (letterboxed) inside the frame. In the prototype it is a
   drag-and-drop image placeholder (`<image-slot>`) — replace with a plain `<img>` / native image view.

4. **NOTES** — two stacked blocks, `gap: 9px`, 12.5px `line-height: 1.5`:
   - General note: `padding: 11px 13px`, `border-radius: 14px`, `background: rgba(12,14,19,.42)`,
     `border: 1px solid rgba(255,255,255,.07)`, `color: rgba(238,240,244,.62)`.
   - **Sanit's note** (amber, mirrors the "Santi Notes" block on the source sheet):
     `background: linear-gradient(150deg, rgba(255,209,120,.14), rgba(255,209,120,.05))`,
     `border: 1px solid rgba(255,209,120,.22)`, `color: rgba(255,226,171,.86)`, with the
     bold prefix "Sanit's note · ".

**Footer (action bar)** — `padding: 14px 18px 16px`,
`border-top: 1px solid rgba(255,255,255,.08)`,
`background: linear-gradient(0deg, rgba(6,7,10,.55), rgba(6,7,10,0))`.
- Row above the buttons (`margin-bottom: 11px`, `gap: 8px`): "Deciding as" (11.5px
  `rgba(238,240,244,.42)`), then a **Sanit / Careem** segmented control — wrapper
  `padding: 3px`, `border-radius: 13px`, `background: rgba(255,255,255,.05)`,
  `border: 1px solid rgba(255,255,255,.08)`; each option `height: 26px`, `padding: 0 12px`,
  `border-radius: 10px`, 12px `font-weight: 600`; active = `color: #0a0b12` on
  `linear-gradient(160deg, rgba(255,255,255,.95), rgba(255,255,255,.74))`, inactive =
  `rgba(238,240,244,.50)`. Right-aligned session counter: "No decisions yet" /
  "{n} decided this session" (11.5px `rgba(238,240,244,.36)`).
- **Awaiting deals** show two buttons, `display: flex; gap: 9px`, both `height: 46px`,
  `border-radius: 16px`, 14px `font-weight: 600` `letter-spacing: -.01em`:
  - **Reject** — `flex: 1`, `color: #ffb3bd`,
    `background: linear-gradient(160deg, rgba(255,90,115,.20), rgba(255,90,115,.07))`,
    `border: 1px solid rgba(255,120,140,.28)`, `box-shadow: inset 0 1px 0 rgba(255,255,255,.14)`.
    Hover: `background: linear-gradient(160deg, rgba(255,90,115,.32), rgba(255,90,115,.12))`, `color: #ffd2d8`.
  - **Approve & auto-submit to {n} lenders** — `flex: 2.1`, `color: #04140f`,
    `background: linear-gradient(160deg,#8ff0d4,#28c9a3)`, `border: 1px solid rgba(255,255,255,.32)`,
    `box-shadow: 0 14px 34px -16px rgba(40,201,163,.85), inset 0 1px 0 rgba(255,255,255,.6)`.
    Hover: `background: linear-gradient(160deg,#a6f6e0,#33dcb3)`.
- **Decided deals** replace both buttons with a single full-width status banner,
  `height: 46px`, `border-radius: 16px`, 13.5px `font-weight: 600`, reading e.g.
  "Submitted by Careem · 9:31 AM · 6 lenders" or "Rejected by Sanit · 5:02 PM · NSF volume".
  Approved: `color: #8ff0d4`, `background: linear-gradient(160deg,rgba(78,224,181,.18),rgba(78,224,181,.06))`,
  `border: 1px solid rgba(78,224,181,.30)`. Rejected: `color: #ffb3bd`,
  `background: linear-gradient(160deg,rgba(255,107,129,.18),rgba(255,107,129,.06))`,
  `border: 1px solid rgba(255,107,129,.30)`.

### Component: PDF viewer overlay
Opens when a document row is clicked. Full-frame scrim: `position: absolute; inset: 0; z-index: 40`,
`background: rgba(4,5,8,.62)`, `backdrop-filter: blur(14px) saturate(140%)`, centered content.
Dialog: **840 × 840**, `border-radius: 26px`, glass at
`linear-gradient(165deg, rgba(255,255,255,.11), rgba(255,255,255,.03))`,
`backdrop-filter: blur(40px) saturate(180%)`, `border: 1px solid rgba(255,255,255,.14)`,
`box-shadow: inset 0 1px 0 rgba(255,255,255,.20), 0 50px 100px -40px rgba(0,0,0,1)`.
- Header (`padding: 15px 18px`, `border-bottom: 1px solid rgba(255,255,255,.08)`): the 28 × 32
  PDF tile, file name (14.5px `font-weight: 600`), meta line "{pages · size} · {company}"
  (11.5px `rgba(238,240,244,.42)`), then right-aligned **Download** pill (`height: 32px`,
  `padding: 0 13px`, `border-radius: 16px`, `background: rgba(255,255,255,.06)`,
  `border: 1px solid rgba(255,255,255,.09)`) and a 32 × 32 round **✕** close button in the same style.
- Body: `padding: 18px`, `background: rgba(6,7,10,.45)`, containing a `border-radius: 12px`
  page frame with `border: 1px solid rgba(255,255,255,.08)` and `min-height: 640px`.
  Render the actual PDF here — PDF.js in the Tauri WebView, served over a custom protocol or
  the `asset:` scope; the prototype shows an image placeholder.
- Close on ✕, on scrim click, and on `Escape` (prototype implements ✕ only).

---

## Interactions & Behavior
- **Select a deal** — clicking a queue card sets it as the selected deal; the review panel
  swaps content and the card gets the selected treatment (blue-tinted fill, blue border, glow).
- **Switch tabs** — Awaiting / Approved / Rejected filters the list by status; the page H1
  changes with it. Selection persists; the review panel keeps showing the last selected deal.
- **Toggle documents** — clicking the "Documents in payload" header collapses/expands the list.
- **Open a document** — opens the PDF overlay for that file.
- **Switch reviewer** — the Sanit / Careem control sets who the decision is attributed to.
  This is a *stand-in for auth*: in production, derive the reviewer from the signed-in user
  and drop the toggle (or keep it read-only).
- **Approve** — sets the deal's status to `approved`, stamps
  "Submitted by {reviewer} · {time} · {n} lenders", increments the auto-submitted counter and the
  session decision count, returns to the Awaiting tab, and auto-selects the next awaiting deal.
  Real behavior: fire the auto-submission job to the matched lenders.
- **Reject** — same flow with status `rejected` and a "Rejected by {reviewer} · {time} · {reason}"
  stamp; does not increment auto-submitted. Real behavior: prompt for / capture a rejection reason —
  the prototype fakes it.
- **Transitions** — deal cards use `transition: transform .18s ease, background .18s ease`.
  The background orb animates on a 27s loop. No other motion. Keep it restrained; honor
  `prefers-reduced-motion` by pausing the orb.
- **Not yet designed** (call these out before building): search, bulk approve, keyboard
  shortcuts (J/K to move through the queue, A/R to decide are the obvious next step),
  toasts/undo after a decision, real-time arrival of new deals, and error/loading states.

## State Management
Prototype state (mirror this shape, whatever the framework):

| State | Type | Notes |
|---|---|---|
| `deals` | array | The full deal set; each item carries `status` (`awaiting` \| `approved` \| `rejected`) and, once decided, a `decision` stamp string and `decidedBy`. |
| `selId` | id \| null | Currently selected deal. |
| `filter` | `Awaiting` \| `Approved` \| `Rejected` | Active tab; drives the list and the H1. |
| `reviewer` | `Sanit` \| `Careem` | Attribution for the next decision — replace with auth. |
| `decided` | int | Decisions made this session (footer counter). |
| `autoSubmitted` | int | Packets sent (stat card); increments on approve only. |
| `docsOpen` | bool | Documents section expanded. |
| `viewing` | `{name, meta, company}` \| null | Open PDF, or null for closed. |

**Deal record fields** (from the source submission sheet): `company, email, phone, puller,
rePuller, state, tib (time in business), position, leadSource, fico (credit score), industry,
revenue, adb (average daily balance), deposits, nsf, lenders[] (suggested/matched),
season[] (12 monthly deposit values or the payload PNG), note, sanitNote, submittedAt, request (amount),
status, decision, decidedBy`.

**Derived `risk` band** — a display-only classification driving the initials-tile tint. In the
prototype it is authored per deal; a reasonable rule to implement: `high` if NSFs ≥ 5 or FICO < 600
or position ≥ 3; `watch` if NSFs 1–4 or FICO < 660; otherwise `clean`. **Confirm the real rule with
the business before shipping.**

- **Data fetching** — the desk needs: a paginated deal list by status, a single deal with its
payload documents, a document/PDF fetch (signed URL or local path), the seasonality image, and
approve/reject mutations — each a `#[tauri::command]`. Approve should be optimistic in the UI
but reconciled against the submission job's result, which arrives as a Tauri event.

## Design Tokens

**Colors**
| Token | Value | Use |
|---|---|---|
| `bg` | `#06070a` | App background |
| `text` | `#eef0f4` | Primary text |
| `text-70` | `rgba(238,240,244,.72)` | Chip / secondary text |
| `text-45` | `rgba(238,240,244,.45)` | Meta text |
| `text-40` | `rgba(238,240,244,.40)` | Section labels, dim keys |
| `text-35` | `rgba(238,240,244,.35)` | Empty values (N/A, None) |
| `accent` | `#9db4ff` | Links, "new deal" eyebrow |
| `accent-deep` | `#6d5cff` | Logo / selected gradients |
| `accent-soft` | `#c8d4ff` | Lender chips (top matches), count pill |
| `success` | `#8ff0d4` → `#28c9a3` | Approve button, approved states |
| `success-ink` | `#04140f` | Text on the approve button |
| `danger` | `#ffb3bd` / `#ff6b81` | Reject, NSF alarm, PDF tile |
| `warn` | `#ffd79a` / `rgba(255,209,120,…)` | Watch tint, Sanit's note |
| `teal-orb` | `rgba(50,215,180,.18)` | Ambient background glow |

**Glass surface recipe** (the whole look depends on this — apply consistently):
```
background: linear-gradient(165deg, rgba(255,255,255,.075–.11), rgba(255,255,255,.022–.03));
backdrop-filter: blur(30–40px) saturate(170–180%);
border: 1px solid rgba(255,255,255,.10–.14);
box-shadow: inset 0 1px 0 rgba(255,255,255,.15–.20), 0 34px 70px -34px rgba(0,0,0,.95);
```
Larger/nearer surfaces use the higher end of every range. The `inset 0 1px 0` highlight is the
single most important detail — it is what reads as a glass edge. On platforms without
`backdrop-filter` (WebKitGTK on Linux), substitute Tauri's native window effects or a solid
`rgba(18,20,26,.92)` fill rather than a transparent surface.

**Spacing scale (px):** 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26.
Panel gap 16; card gap 9; chip gap 8; section gap 18.

**Radii (px):** 5 (key hint), 6–7 (small tiles/pills), 10–11 (segmented options),
12–14 (icons, note blocks, chips), 16 (buttons, field grid), 19–20 (search, stat cards, deal cards),
24 (nav rail), 26 (main panels, PDF dialog), 50% (status badges).

**Typography** — system stack:
`-apple-system, BlinkMacSystemFont, "SF Pro Display", "Helvetica Neue", Helvetica, sans-serif`,
with `ui-monospace, Menlo, monospace` for the `⌘K` hint. `-webkit-font-smoothing: antialiased`.

| Role | Size / weight / tracking |
|---|---|
| H1 page title | 26 / 600 / -.022em |
| H2 company (panel) | 23 / 600 / -.024em |
| Stat value | 30 / 600 / -.03em |
| Card company | 15 / 600 / -.012em |
| Amount | 16 / 600 / -.02em |
| Action buttons | 14 / 600 / -.01em |
| Field value, doc name | 12.5–13 / 400–500 |
| Chips, meta | 11–11.5 / 400 |
| Section label / eyebrow | 10–10.5 / 600–700 / .13–.16em uppercase |

**Shadows**
- Panel: `0 34px 70px -34px rgba(0,0,0,.95)`
- Stat card: `0 20px 44px -28px rgba(0,0,0,.9)`
- Nav rail: `0 24px 60px -24px rgba(0,0,0,.9)`
- Selected deal card: `0 16px 36px -22px rgba(90,120,255,.9)`
- Approve button: `0 14px 34px -16px rgba(40,201,163,.85)`
- PDF dialog: `0 50px 100px -40px rgba(0,0,0,1)`
- Every glass surface additionally carries `inset 0 1px 0 rgba(255,255,255,.15–.20)`

## Assets
- **Home icon** — inline 24×24 stroked SVG path, `stroke-width: 2`, round caps/joins.
  Swap for the codebase's icon set (Lucide `home` is an exact match).
- **Glyphs used as icons** — `⌕` (search), `✕` (close), `▾`/`▸` (caret), `✓`/`✕` (decision badges).
  Replace all of these with real icons from the app's icon library.
- **Seasonality image** — supplied per deal by the submission payload (PNG). No asset shipped here.
- **No fonts to bundle** — system stack only.
- No logo asset exists; the "C" tile is a placeholder for the real product mark.

## Files
- `Conduit Dashboard.dc.html` — the full design (markup + all styling + prototype logic).
  The data model, seeded deals, documents list, and the decision logic live in the
  `class Component` script block at the bottom of the file.
- `support.js`, `image-slot.js` — prototype runtime only. **Do not port.** They exist so
  the HTML opens and runs in a browser. Nothing in this bundle should be shipped inside the
  Tauri app; it is reference material for rebuilding the UI.
