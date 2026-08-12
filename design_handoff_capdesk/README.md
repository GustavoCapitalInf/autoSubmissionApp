# Handoff: CapDesk — Submission Desk (light green iOS-style UI)

## Overview
A light, flat, Apple-iOS-flavored desktop UI for **CapDesk**, an auto-submission app used by
a funding brokerage. Two reviewers — **Santi** and **Careem** — work a queue of newly
submitted merchant funding deals and either **approve** (the packet auto-submits to the
matched lenders) or **reject** each one.

The screen answers three questions:
1. What came in and hasn't been decided yet?
2. Everything about the selected deal, including the PDFs in its submission payload.
3. One decisive action: Approve, or Reject.

## Target: Rust + Tauri desktop app
- **Backend (Rust):** deal list, deal detail, document fetch, seasonality image, and the
  approve/reject mutations belong in `#[tauri::command]` handlers; the frontend calls them
  through `invoke`. Run the lender auto-submission job on the Rust side (async, off the UI
  thread) and report back with a Tauri event (`emit`/`listen`) so the dashboard updates the
  "Auto-submitted" counter and the decision stamp without polling.
- **PDFs:** don't stream bytes through `invoke`. Serve the file to the WebView via a custom
  protocol or the `asset:` scope and render with PDF.js inside the overlay, or hand off to the
  OS viewer with `tauri-plugin-opener`.
- **Window:** `"decorations": false` plus a custom drag region suits this design. Set
  `"minWidth": 1280, "minHeight": 860` in `tauri.conf.json` — the layout is designed at
  1440 × 980 and breaks below 1280 wide.
- **No `backdrop-filter` dependency.** This design is flat by request: solid fills, hairline
  borders, one soft shadow. The only blur in the file is the PDF overlay scrim, and a solid
  `--scrim` fill is an acceptable substitute on WebKitGTK.
- **Theme:** apply the stored `data-theme` in an inline `<head>` script before the bundle mounts
  so there is no flash of the wrong theme; consider seeding it from the OS appearance on first
  run. Persist through the app's settings store rather than `localStorage`.
- Keep the state table below in the frontend; treat Rust as the source of truth and reconcile
  after each mutation.

## About the design files
`CapDesk Dashboard.dc.html` is a **working prototype** — open it in a browser to see and click
the intended behavior, and read it for exact values. It is **reference only**: it runs on a
proprietary streaming-template runtime (`support.js`, `image-slot.js`, `<sc-for>`, `<sc-if>`,
`{{ }}` holes). **Do not port that runtime and do not ship any file from this folder inside the
app** — except the two real assets in `assets/` (the logo and the PDF icon), which are yours to use.

Rebuild the UI idiomatically in the app's frontend framework. Read the HTML for pixel values,
colors, and copy; write normal components.

## Fidelity
**High fidelity.** Colors, type sizes, radii, spacing, and copy are final and should be matched
closely. Everything in "Design Tokens" is exact. Deal data is realistic placeholder content —
replace with real payload data.

> **Theming:** the app ships **light and dark themes plus a sidebar toggle**. Every color resolves
> through a CSS custom property; `data-theme="dark"` on the document root flips the whole UI.
> **`THEMING.md` in this folder is the authority** for the 48-token set, the dark values, the dark
> theme rules, the PDF-icon treatment, and the toggle's markup and behavior. Where the color
> tables below give a light hex, that hex is the light value of a token — read `THEMING.md` for
> its name and dark counterpart.

---

## Design principles (do not violate)
1. **Flat only.** No gradients, no glows, no glassmorphism, no colored shadows. Fills are solid;
   depth comes from one hairline border and at most one very soft neutral shadow.
2. **Two colors: green and neutral.** Green (`#2E9E58` / `#1B8F4F` / `#E4F2E8`) means active,
   approved, or matched. Everything else is a gray-green neutral. **There is no red, amber, or
   any third hue anywhere** — including Reject, rejected badges, NSF alerts, and the PDF icon.
   Earlier iterations had them; they were deliberately removed. Do not reintroduce them.
3. **iOS shapes and type.** Generous continuous radii (12–26px), SF system font stack, tight
   negative letter-spacing on headings, uppercase micro-labels only for section headers.
4. **Restrained motion.** Only two transitions exist: the sidebar width and the review panel
   slide. Nothing pulses, bounces, or animates on load.

---

## Screen: Submission Desk (the only screen)
Design frame **1440 × 980**, background `#EDF3EE`. Root is `display: flex` — sidebar, then main.
`<main>` is `flex: 1`, `padding: 20px 24px 22px 18px`, column flex, `gap: 16px`.

### Component: Sidebar (collapsible)
A `flex: none` rail with `margin: 16px 0 16px 16px`, `padding: 14px 12px`, `border-radius: 24px`,
`background: #FFFFFF`, `border: 1px solid rgba(16,42,26,.07)`, and
`transition: width .3s cubic-bezier(.22,.9,.24,1)`.
**Collapsed width 64px (default) · expanded width 232px.**

Three stacked regions:

**Header** — `display: flex; align-items: center; gap: 8px; min-height: 48px;`
`padding: 8px 2px 16px`, `border-bottom: 1px solid rgba(16,42,26,.07)`. Collapsed it becomes
`flex-direction: column`.
- **Expanded only:** the logo image at **52 × 52** with `object-fit: contain` and
  `margin: -4px -3px` (negative margins crop the PNG's built-in padding so it reads larger
  without growing the row), then the wordmark **"CapDesk"** at 17px / 700 / `-.022em` / `line-height: 1`.
  Both sit in a 48px-tall flex row so they share one baseline.
- **Always:** the panel toggle — a **bare 42 × 42 icon button, no background box**,
  `color: rgba(16,35,26,.50)`, hover `#10231A`, holding a 23px stroked "sidebar" glyph
  (rounded rect + vertical divider line at x=9, `stroke-width: 1.7`). Expanded it is pushed
  right with `margin-left: auto; margin-right: -8px`; collapsed it gets `margin-top: 2px`.
  **Collapsed shows ONLY this toggle** — no logo, no wordmark.

**Nav list** — `display: flex; flex-direction: column; gap: 4px; padding-top: 14px`
(collapsed adds `align-items: center`). One destination currently ships: **Submission desk**
(active), a 22px-boxed 18px stroked house icon (`stroke-width: 1.9`). Row style:
```
position: relative; display: flex; align-items: center; gap: 11px;
height: 40px; border-radius: 13px; cursor: pointer;
transition: background .16s ease, color .16s ease;
expanded:  padding: 0 11px;
collapsed: width: 40px; justify-content: center;
active:    background: #E4F2E8; color: #1B8F4F;   (label weight 600)
hover:     background: #F2F6F2; color: #10231A;
idle:      background: transparent; color: rgba(16,35,26,.52);   (label weight 500)
```
Labels are 13.5px, `letter-spacing: -.008em`, `white-space: nowrap`, and render **only when
expanded**.

**Hover tooltip (collapsed only)** — hovering a nav icon shows a dark pill to its right:
```
position: absolute; left: calc(100% + 12px); top: 50%; transform: translateY(-50%);
z-index: 20; height: 30px; padding: 0 11px; border-radius: 9px;
background: #10231A; color: #FFFFFF; font-size: 12.5px; font-weight: 500;
white-space: nowrap; pointer-events: none;
box-shadow: 0 6px 18px -10px rgba(16,42,26,.6);
```
It shows the page name and never appears when the sidebar is expanded. Drive it from a
`navHover` state value, not CSS `:hover` on a sibling.

**Footer** — `padding-top: 14px`, `border-top: 1px solid rgba(16,42,26,.07)`,
`display: flex; align-items: center; gap: 10px` (collapsed: `justify-content: center`).
A 32px round avatar (`background: #E4F2E8`, `color: #1B8F4F`, 11px/700 initials) showing the
active reviewer, plus — expanded only — their name (12.5px/600) and "Reviewer"
(11px `rgba(16,35,26,.45)`).

**Theme toggle** — a nav-styled row directly above the footer, pushed down with `margin-top: auto`
and `padding-bottom: 10px`. Full spec (metrics, sun/moon icons, label, tooltip, persistence) in
`THEMING.md` §5.

### Component: Header
`display: flex; align-items: center; gap: 16px`.
- **H1** 27px / 700 / `-.026em` / `line-height: 1.1`. Text follows the active tab:
  `Awaiting decision` / `Submitted deals` / `Rejected submissions`.
  (There is no uppercase eyebrow above it — it was removed.)
- **Search field**, right-aligned: 280 × 38, `border-radius: 19px`, `background: #FFFFFF`,
  `border: 1px solid rgba(16,42,26,.07)`, `box-shadow: 0 1px 2px rgba(16,42,26,.04)`, holding a
  `⌕` glyph (`rgba(16,35,26,.35)`) and the placeholder **"Search deals"** (13px
  `rgba(16,35,26,.38)`). Not wired up in the prototype — implement as a filter across
  company / puller / lender. (No ⌘K hint — removed.)

### Component: Stat cards
`display: grid; grid-template-columns: repeat(4,1fr); gap: 12px` — **two cards occupy the first
two cells**, each a horizontal row:
```
display: flex; align-items: center; gap: 13px; padding: 15px 17px;
border-radius: 20px; background: #FFFFFF; border: 1px solid rgba(16,42,26,.07);
```
- **Icon tile** 38 × 38, `border-radius: 12px`, holding a 19px stroked icon:
  queue card → neutral (`color: #42604E`, `background: #EEF3EF`, three-line "list" glyph);
  auto-submitted card → green (`color: #1B8F4F`, `background: #E4F2E8`, checkmark glyph).
- **Value row:** number 26px / 700 / `-.03em` / `line-height: 1` (green `#1B8F4F` on the
  auto-submitted card, default ink on the queue card), then the unit "deals" 12px
  `rgba(16,35,26,.45)`.
- **Label** below, `margin-top: 4px`, 12px / 500 / `rgba(16,35,26,.50)`, sentence case:
  "In queue" and "Auto-submitted". Not uppercase.

### Component: Deal queue
The queue and the review panel sit in a `display: flex; gap: 16px; flex: 1; min-height: 0` row.
The queue is `flex: 1; min-width: 0` — **not a fixed grid column**, so it expands when the
panel is closed.

Panel shell: `border-radius: 26px`, `background: #FFFFFF`,
`border: 1px solid rgba(16,42,26,.07)`,
`box-shadow: 0 1px 2px rgba(16,42,26,.04), 0 24px 50px -34px rgba(16,42,26,.40)`,
`overflow: hidden`, column flex.

**Tab bar** — `padding: 14px 16px 12px`, `border-bottom: 1px solid rgba(16,42,26,.07)`.
iOS segmented control: track `padding: 3px; gap: 2px; border-radius: 14px; background: #EEF3EE`.
Three tabs — **Awaiting N · Approved N · Rejected N** (count follows the label after two
spaces), each `height: 29px`, `padding: 0 14px`, `border-radius: 11px`, 12.5px,
`white-space: nowrap`, `transition: background .18s ease`.
- Active: 600, `color: #10231A`, `background: #FFFFFF`, `box-shadow: 0 1px 3px rgba(16,42,26,.14)`.
- Inactive: 500, `color: rgba(16,35,26,.50)`, no background.

**Scroll area** — `flex: 1; overflow-y: auto; padding: 10px 12px 14px`, column flex, `gap: 9px`.

**Deal card** — `padding: 13px 14px`, `border-radius: 20px`, `cursor: pointer`,
`transition: background .18s ease, border-color .18s ease, box-shadow .18s ease`.
- Idle: `background: #FFFFFF; border: 1px solid rgba(16,42,26,.09)`.
- Selected: `background: #EBF6EE; border: 1px solid #2E9E58;` (a solid green hairline — no glow).

Row 1 (`display: flex; gap: 12px; align-items: flex-start`):
- **Initials tile** 38 × 38, `border-radius: 13px`, 12.5px / 700. Tinted by the deal's internal
  risk band: `clean` → `color: #1B8F4F`, `background: #E4F2E8`; `watch` and `high` →
  `color: #42604E`, `background: #EAF0EB`. **No text risk badges anywhere** — the tint is the
  only signal, and watch/high intentionally share one neutral tint.
- **Company** 15px / 600 / `-.014em`, truncates.
- **Meta line** 11.5px `rgba(16,35,26,.48)`, dot-separated (`·` at `opacity: .4`, `gap: 7px`):
  industry · state · "{tib} in business" · "Position {n}".
- **Right rail:** the submission time only, 11px `rgba(16,35,26,.40)`. **No dollar amount** —
  requested amounts aren't known at submission time. Do not add one.

Row 2 (`margin-top: 11px`, `display: flex; gap: 8px; align-items: center`) — chips are
`height: 26px`, `padding: 0 11px`, `border-radius: 13px`, 11.5px,
`background: #F2F6F2`, `color: rgba(16,35,26,.72)`, with the leading label word dimmed to
`rgba(16,35,26,.42)`: `FICO {n}`, `ADB {value}`, `NSF {n}`, `{n} lenders matched`.
- `NSF` when `nsf >= 5` stays neutral but emphasizes: `color: #10231A; font-weight: 600;`
  `background: #E6ECE7`. **Not red.**
- **Far right (decided deals only):** label 11px `rgba(16,35,26,.42)` reading
  **"Approved by Santi"** / **"Rejected by Careem"** etc., then a 24px round badge —
  approved `✓` white on `#2E9E58`; rejected `✕` `#5B6B60` on `#E6ECE7`.
  Awaiting deals render nothing here.

**Empty state** — `padding: 56px 20px`, centered; title 15px / 600 `rgba(16,35,26,.72)`,
body 12.5px `rgba(16,35,26,.45)`:
- Awaiting → "Queue cleared" / "Every submission has a decision. New deals land here automatically."
- Approved → "Nothing submitted yet" / "Approved deals move here once the packet goes out to lenders."
- Rejected → "Nothing rejected yet" / "Rejected deals are kept here with the reviewer who declined them."

### Component: Review panel (470px — closed by default, opens on click)
**Closed by default.** With nothing selected the queue fills the full width. Clicking a deal card
opens the panel; clicking another card swaps the content in place; the **✕ in the panel header**
closes it; approving or rejecting also closes it. **Clicking a card never closes the panel** —
card clicks are non-toggling (see State Management for why).

**Open/close animation** — the panel lives in a wrapper that animates:
```
flex: none; min-height: 0; overflow: hidden;
transition: width .42s cubic-bezier(.22,.9,.24,1),
            opacity .34s ease,
            transform .42s cubic-bezier(.22,.9,.24,1);
open:   width: 470px; opacity: 1; transform: translateX(0);
closed: width: 0;     opacity: 0; transform: translateX(30px); pointer-events: none;
```
The `<aside>` inside keeps a fixed `width: 470px; height: 100%` so content doesn't reflow
mid-animation. Keep the **last-selected** deal mounted while it collapses so the close doesn't
flash empty. Honor `prefers-reduced-motion` by dropping the transition.

Panel shell matches the queue: `border-radius: 26px`, `#FFFFFF`,
`border: 1px solid rgba(16,42,26,.07)`,
`box-shadow: 0 1px 2px rgba(16,42,26,.04), 0 24px 50px -34px rgba(16,42,26,.40)`.
Three regions: fixed header, scrolling body, fixed action footer.

**Header** — `padding: 18px 20px 14px`, `border-bottom: 1px solid rgba(16,42,26,.07)`.
- **Status pill** (not plain text): `display: inline-flex; height: 24px; padding: 0 11px;`
  `border-radius: 12px; font-size: 10px; letter-spacing: .12em; text-transform: uppercase;`
  `font-weight: 700; white-space: nowrap`.
  Awaiting → "NEW DEAL SUBMITTED", `color: #1B8F4F`, `background: #E4F2E8`.
  Approved → "SUBMITTED TO LENDERS", same green.
  Rejected → "REJECTED", `color: #5B6B60`, `background: #EDF1EE`.
- Right-aligned "Submitted MM/DD/YYYY hh:mm AM", 11px `rgba(16,35,26,.40)`.
- **Close ✕** — 26 × 26, `border-radius: 13px`, 12px glyph, `color: rgba(16,35,26,.50)`,
  `background: #F1F5F1`; hover `background: #E6ECE6; color: #10231A`.
- **Company H2** 23px / 700 / `-.026em` / `line-height: 1.15`, `margin-top: 7px`.
- **Chips row** (`margin-top: 9px`, `gap: 8px`), both `height: 28px`, `padding: 0 12px`,
  `border-radius: 14px`, 12px:
  "{n} lenders matched" — `background: #E4F2E8; color: #1B8F4F; font-weight: 600`;
  "Position {n}" — `background: #F2F6F2; color: rgba(16,35,26,.65)`, built as two spans with
  `gap: 5px` (never rely on source whitespace between the word and the number).

**Body** — `flex: 1; overflow-y: auto; padding: 16px 20px 18px`, column flex, `gap: 18px`.
Each section opens with a micro-label: 10.5px, uppercase, `letter-spacing: .12em`,
`font-weight: 600`, `color: rgba(16,35,26,.42)`.

1. **DEAL INFORMATION** — two-column hairline grid:
   `display: grid; grid-template-columns: 1fr 1fr; gap: 1px;`
   `background: rgba(16,42,26,.08)` (that 1px gap *is* the divider),
   `border: 1px solid rgba(16,42,26,.08)`, `border-radius: 16px`, `overflow: hidden`.
   Cells: `background: #FAFCFA`, `padding: 9px 12px`, column flex, `gap: 2px`.
   Key 10.5px uppercase `.05em` 600 `rgba(16,35,26,.40)`; value 13px `rgba(16,35,26,.82)`, truncating.
   **Field order (left→right, matching the source submission sheet):** Company · Lead source ·
   Email · Credit score · Phone · Industry · Puller · Revenue · Re-puller · Avg daily balance ·
   State · Deposits · Time in business · NSFs.
   **Value color exceptions:** email is `#1B8F4F`; `N/A` and `None` dim to `rgba(16,35,26,.35)`.
   Credit score and NSFs are **plain ink regardless of value** — no red thresholds.

2. **DOCUMENTS SUBMITTED** — collapsible, default open. Header row is clickable: the label, a
   count pill (`height: 19px; min-width: 19px; padding: 0 6px; border-radius: 9px;`
   `font-size: 10.5px; font-weight: 700; color: #1B8F4F; background: #E4F2E8`), and a caret
   `▾`/`▸` (11px `rgba(16,35,26,.45)`).
   Rows: column flex `gap: 6px`; each row
   `display: flex; align-items: center; gap: 11px; padding: 9px 11px; border-radius: 14px;`
   `background: #F7FAF7; border: 1px solid rgba(16,42,26,.08); cursor: pointer;`
   `transition: background .16s ease, border-color .16s ease`.
   Hover: `background: rgba(52,199,89,.10); border-color: rgba(52,199,89,.30)`.
   Contents: the **PDF icon PNG** (`assets/pdf-icon.png`) at **44 × 44** with
   `object-fit: contain` and `margin: -5px -6px -5px -4px` to crop its padding; the file name
   (12.5px / 500, truncating) with its meta line beneath ("4 pages · 2.1 MB", 11px
   `rgba(16,35,26,.42)`); and a right-aligned **"View"** in 11.5px / 500 `#1FA75C`.
   This list is data-driven from the submission payload.

3. **SEASONALITY · MONTHLY DEPOSITS** — label with right-aligned "trailing 12"
   (11px `rgba(16,35,26,.40)`). Body is a **150px tall frame**: `border-radius: 16px`,
   `background: #FAFCFA`, `border: 1px solid rgba(16,42,26,.08)`, `overflow: hidden`.
   In production this renders the **seasonality PNG that arrives with the payload**, contained
   (letterboxed). The prototype uses a drag-and-drop placeholder — replace with a plain image.

4. **SUGGESTED LENDERS · AUTO-MATCHED** — wrapping flex, `gap: 6px`. **Every** matched lender
   is a green chip: `height: 27px; padding: 0 12px; border-radius: 13px; font-size: 12px;`
   `color: #1B8F4F; background: #E4F2E8; font-weight: 600`. (An earlier version highlighted only
   the first three — that was wrong; all of them are matches.)

5. **NOTES** — two stacked blocks, `gap: 9px`, 12.5px `line-height: 1.5`:
   - General note: `padding: 11px 13px`, `border-radius: 14px`, `background: #FAFCFA`,
     `border: 1px solid rgba(16,42,26,.08)`, `color: rgba(16,35,26,.65)`.
   - **Santi's note** — green, not amber: `background: #E4F2E8`, `color: #25543A`, no border,
     with the bold prefix "Santi's note · ". (Spelling is **Santi**, not "Sanit".)

**Footer (action bar)** — `padding: 14px 18px 16px`,
`border-top: 1px solid rgba(16,42,26,.07)`, `background: #FAFCFA`.
- Row above the buttons (`margin-bottom: 11px`, `gap: 8px`): "Deciding as" (11.5px
  `rgba(16,35,26,.45)`), then a **Santi / Careem** segmented control — track `padding: 3px;`
  `border-radius: 13px; background: #EEF3EE`; options `height: 26px; padding: 0 13px;`
  `border-radius: 10px; font-size: 12px; font-weight: 600`; active `color: #10231A;`
  `background: #FFFFFF; box-shadow: 0 1px 3px rgba(16,42,26,.14)`; inactive
  `color: rgba(16,35,26,.48)`. Right-aligned session counter: "No decisions yet" /
  "{n} decided this session" (11.5px `rgba(16,35,26,.38)`).
- **Awaiting deals** show two buttons, `display: flex; gap: 9px`, both `height: 46px`,
  `border-radius: 16px`, 14px / 600:
  - **Reject** — `flex: 1`, `color: #42604E`, `background: #EDF1EE`, hover `#E3E9E4`.
    Neutral, **not red**.
  - **Approve** — `flex: 2.1`, `color: #FFFFFF`, `background: #2E9E58`, hover `#278A4C`,
    `letter-spacing: -.012em`. The label is exactly **"Approve"** — no lender count, no
    second line.
- **Decided deals** replace both buttons with one full-width banner, `height: 46px`,
  `border-radius: 16px`, 13.5px / 600, reading e.g. "Submitted by Careem · 9:31 AM · 6 lenders"
  or "Rejected by Santi · 5:02 PM · NSF volume".
  Approved: `color: #1B8F4F; background: #E4F2E8`. Rejected: `color: #5B6B60; background: #EDF1EE`.

### Component: PDF viewer overlay
Opens when a document row is clicked. Scrim: `position: absolute; inset: 0; z-index: 40;`
`background: rgba(20,40,28,.28)`, `backdrop-filter: blur(18px) saturate(140%)`, centered.
Dialog **840 × 840**, `border-radius: 26px`, `background: #FFFFFF`,
`border: 1px solid rgba(16,42,26,.08)`, `box-shadow: 0 40px 90px -40px rgba(16,42,26,.45)`.
- Header (`padding: 15px 18px`, `border-bottom: 1px solid rgba(16,42,26,.08)`): the PDF icon PNG
  at 32 × 32, the file name (14.5px / 600) with meta "{pages · size} · {company}" (11.5px
  `rgba(16,35,26,.45)`), then a right-aligned **Download** pill (`height: 32px; padding: 0 14px;`
  `border-radius: 16px; background: #F1F5F1; border: 1px solid rgba(16,42,26,.08);` 12.5px / 500)
  and a 32 × 32 round **✕** in the same style.
- Body: `padding: 18px`, `background: #F6F9F6`, containing a `border-radius: 14px` page frame
  (`background: #FFFFFF`, `border: 1px solid rgba(16,42,26,.08)`, `min-height: 640px`).
  Render the real PDF here (PDF.js in the WebView).
- Close on ✕, on scrim click, and on `Escape` (prototype implements ✕ only).

---

## Interactions & Behavior
- **Select a deal** — opens the review panel (animation above) and gives the card the green
  selected treatment. Another card swaps content in place. **Card clicks never close the panel.**
- **Close the panel** — the ✕ in the panel header (add `Escape` too).
- **Switch tabs** — Awaiting / Approved / Rejected filters the list by status; the H1 follows.
- **Toggle the sidebar** — the header toggle switches 64px ↔ 232px; labels, logo, wordmark and
  the footer name fade in with the expansion. Collapsed shows tooltips on icon hover.
- **Toggle the theme** — the sidebar's bottom nav row flips light ↔ dark instantly and persists
  the choice (`THEMING.md`).
- **Toggle documents** — clicking the "Documents Submitted" header collapses/expands the list.
- **Open a document** — opens the PDF overlay for that file.
- **Switch reviewer** — the Santi / Careem control sets who the decision is attributed to. This
  is a *stand-in for auth*: in production derive the reviewer from the signed-in user and drop
  the toggle (or make it read-only).
- **Approve** — sets status `approved`, stamps "Submitted by {reviewer} · {time} · {n} lenders",
  increments the auto-submitted counter and the session decision count, **closes the panel**,
  and returns to the Awaiting tab. Real behavior: fire the auto-submission job to the matched
  lenders.
- **Reject** — same flow with status `rejected` and a "Rejected by {reviewer} · {time} · {reason}"
  stamp; does not increment auto-submitted. Real behavior: capture a rejection reason — the
  prototype fakes it.
- **Not yet designed** — ask before inventing: search behavior, rejection-reason capture,
  keyboard shortcuts (J/K through the queue, A/R to decide are the obvious next step),
  toasts/undo after a decision, real-time arrival of new deals, loading and error states, and
  the destinations beyond "Submission desk" in the sidebar.

## State Management

| State | Type | Notes |
|---|---|---|
| `deals` | array | Full deal set; each item carries `status` (`awaiting` \| `approved` \| `rejected`) and, once decided, `decision` (stamp string) and `decidedBy`. |
| `selId` | id \| null | Selected deal; **`null` means the review panel is closed.** |
| `lastSelId` | id | Last opened deal — keeps panel content mounted during the close animation. |
| `filter` | `Awaiting` \| `Approved` \| `Rejected` | Active tab; drives the list and the H1. |
| `reviewer` | `Santi` \| `Careem` | Attribution for the next decision — replace with auth. |
| `decided` | int | Decisions this session (footer counter). |
| `autoSubmitted` | int | Packets sent (stat card); increments on approve only. |
| `docsOpen` | bool | Documents section expanded. |
| `viewing` | `{name, meta, company}` \| null | Open PDF, or null for closed. |
| `navOpen` | bool | Sidebar expanded. |
| `navPage` | string | Active nav destination. |
| `navHover` | string \| null | Nav item under the cursor — drives the collapsed tooltip. |
| `theme` | `light` \| `dark` | Active theme; mirrored onto `data-theme` and persisted as `capdesk-theme`. See `THEMING.md`. |

**Card click must not toggle.** A toggling handler (`selId === d.id ? null : d.id`) broke in the
prototype: the click handler fired twice, the second dispatch read the re-rendered closure where
the card was already active, and the panel opened and closed within one frame — making the
entire review flow unreachable. Always `{ selId: d.id, lastSelId: d.id }` on card click and
close only from the ✕. If your framework double-invokes handlers (StrictMode or similar), verify:
first click opens; clicking another card swaps; clicking the same card leaves it open.

**Deal record fields** (from the source submission sheet): `company, initials, email, phone,
puller, rePuller, state, tib, position, leadSource, fico, industry, revenue, adb, deposits, nsf,
lenders[], note, sanitNote, submittedAt, ago, risk, status, decision, decidedBy`.

**Derived `risk` band** — display-only, drives the initials-tile tint. Authored per deal in the
prototype; a reasonable rule: `high` if NSFs ≥ 5 or FICO < 600 or position ≥ 3; `watch` if
NSFs 1–4 or FICO < 660; else `clean`. **Confirm the real rule with the business.** Note `watch`
and `high` render identically today.

**Data fetching** — the desk needs: a paginated deal list by status, a single deal with its
payload documents, a document/PDF fetch, the seasonality image, and approve/reject mutations —
each a `#[tauri::command]`. Approve should be optimistic in the UI but reconciled against the
submission job's result, which arrives as a Tauri event.

## Design Tokens

**Colors**
| Token | Value | Use |
|---|---|---|
| `canvas` | `#EDF3EE` | App background |
| `surface` | `#FFFFFF` | Cards, panels, sidebar, active segment |
| `surface-sunken` | `#FAFCFA` | Field cells, note blocks, footer bar |
| `surface-alt` | `#F7FAF7` | Document rows |
| `fill-neutral` | `#F2F6F2` | Chips, small buttons |
| `fill-track` | `#EEF3EE` | Segmented-control tracks |
| `fill-emphasis` | `#E6ECE7` | Emphasized neutral chip (NSF ≥ 5), rejected badge |
| `fill-button` | `#EDF1EE` (hover `#E3E9E4`) | Reject button, rejected banner |
| `border` | `rgba(16,42,26,.07)` | Standard hairline |
| `border-strong` | `rgba(16,42,26,.08–.09)` | Field grid, deal cards |
| `ink` | `#10231A` | Primary text |
| `ink-82` | `rgba(16,35,26,.82)` | Field values |
| `ink-72` | `rgba(16,35,26,.72)` | Chip text |
| `ink-50` | `rgba(16,35,26,.48–.52)` | Meta, inactive nav |
| `ink-42` | `rgba(16,35,26,.40–.45)` | Micro-labels, timestamps |
| `ink-35` | `rgba(16,35,26,.35)` | Empty values (N/A, None) |
| `ink-muted` | `#42604E` / `#5B6B60` | Neutral icon + rejected text |
| `green` | `#2E9E58` (hover `#278A4C`) | Approve, selected border, approved badge |
| `green-text` | `#1B8F4F` | Green text, figures, active nav label |
| `green-link` | `#1FA75C` | "View" links, `a` |
| `green-tint` | `#E4F2E8` | Green chip / pill / tile fills |
| `green-tint-2` | `#EBF6EE` | Selected deal card fill |
| `tooltip` | `#10231A` on white text | Collapsed nav tooltip |

**No red, no amber, no third hue.** Rejection, NSF alarms, and PDF icons are all neutral.

**Spacing scale (px):** 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 18, 20, 22, 24.
Panel gap 16; card gap 9; chip gap 8; section gap 18.

**Radii (px):** 9 (count pill, tooltip), 10–11 (segment options), 12–14 (icon tiles, chips,
note blocks, document rows), 16 (buttons, field grid), 19–20 (search, stat cards, deal cards),
24 (sidebar), 26 (main panels, PDF dialog), 50% (avatars, status badges).

**Typography** — system stack:
`-apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", "Helvetica Neue", Helvetica, sans-serif`,
`-webkit-font-smoothing: antialiased`.

| Role | Size / weight / tracking |
|---|---|
| H1 page title | 27 / 700 / -.026em |
| H2 company (panel) | 23 / 700 / -.026em |
| Wordmark | 17 / 700 / -.022em |
| Stat value | 26 / 700 / -.03em |
| Card company | 15 / 600 / -.014em |
| Buttons | 14 / 600 / -.01em |
| Nav label | 13.5 / 500–600 / -.008em |
| Field value, doc name | 12.5–13 / 400–500 |
| Chips, meta | 11–12 / 400–600 |
| Micro-label / status pill | 10–10.5 / 600–700 / .12em uppercase |

**Shadows** (neutral and soft — never colored):
- Cards / panels: `0 1px 2px rgba(16,42,26,.04)`
- Main panels add: `0 24px 50px -34px rgba(16,42,26,.40)`
- Active segment: `0 1px 3px rgba(16,42,26,.14)`
- Tooltip: `0 6px 18px -10px rgba(16,42,26,.6)`
- PDF dialog: `0 40px 90px -40px rgba(16,42,26,.45)`

**Scrollbars:** 8px, thumb `rgba(16,42,26,.16)`, `border-radius: 8px`, transparent track.

**Links:** `a { color: #1FA75C }`, `a:hover { color: #17864A }`.

## Assets (ship these — they are real)
- `assets/logo.png` — CapDesk mark, shown at 52 × 52 in the expanded sidebar with
  `margin: -4px -3px` to crop its internal padding.
- `assets/pdf-icon.png` — line-art PDF file icon, at 44 × 44 in document rows
  (`margin: -5px -6px -5px -4px`) and 32 × 32 in the viewer header.
- **Inline stroked SVG icons** (`stroke-width: 1.7–1.9`, round caps/joins) for: sidebar toggle
  (rounded rect + divider), nav home, stat "list", stat checkmark. Lucide equivalents:
  `panel-left`, `home`, `align-left`, `check`. Swap for the codebase's icon set.
- `⌕`, `✕`, `▾`/`▸`, `✓` are glyph placeholders — replace with real icons.
- No fonts to bundle — system stack only.

## Files
- `README.md` — this file: layout, components, interactions, state, tokens.
- `THEMING.md` — light/dark token set, dark-theme rules, and the sidebar theme toggle.
- `CapDesk Dashboard.dc.html` — the full design (markup + all styling + prototype logic). Deal
  data, the documents map, and the decision logic live in the `class Component` script block at
  the bottom; the token blocks are at the top of `<helmet>`.
- `assets/logo.png`, `assets/pdf-icon.png` — real assets, use them.
- `support.js`, `image-slot.js` — prototype runtime only. **Do not port; do not ship.**
