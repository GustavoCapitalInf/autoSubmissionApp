// CapDesk — Submission Desk frontend.
// State shape mirrors the design handoff's prototype; data comes from the
// conduit-server via Tauri commands, live updates via `server-event`.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();

const state = {
  deals: [],
  baseUrl: "",
  fileKey: "",              // desk API key, appended to direct file URLs
  selId: null,              // selected deal; null = review panel closed
  lastSelId: null,          // content source while the panel animates closed
  filter: "Awaiting",       // Awaiting | Approved | Rejected
  reviewer: "Santi",        // stand-in for auth, per the handoff
  decided: 0,               // decisions made this session
  autoSubmitted: 0,         // packets sent (server stat); approve increments
  docsOpen: true,
  viewing: null,            // { docId, name, meta, company, hasFile }
  rejecting: false,         // reject-reason capture visible
  search: "",
  navOpen: false,           // sidebar expanded (collapsed 64px is the default)
  navPage: "desk",          // active nav destination
  navHover: null,           // nav item under the cursor — drives the tooltip
};

// Direct-loaded resources (PDF fetch, downloads) can't carry headers, so the
// key rides as a query parameter.
const docFileUrl = (docId) =>
  `${state.baseUrl}/api/documents/${docId}/file?key=${encodeURIComponent(state.fileKey)}`;

/* ── Formatting helpers ────────────────────────────────────────────── */

const esc = (s) =>
  String(s ?? "").replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

const fmtBytes = (b) =>
  b >= 1024 * 1024
    ? (b / (1024 * 1024)).toFixed(1) + " MB"
    : Math.max(1, Math.round(b / 1024)) + " KB";

// "bank_statements" → "Bank statements"
const categoryLabel = (c) => {
  const s = String(c || "").replace(/[_-]+/g, " ").trim();
  return s ? s[0].toUpperCase() + s.slice(1) : "";
};

const isPdfDoc = (d) => !d.contentType || d.contentType === "application/pdf";

const docMeta = (d) => {
  const parts = [];
  if (d.category) parts.push(categoryLabel(d.category));
  if (d.pages > 0) parts.push(`${d.pages} page${d.pages === 1 ? "" : "s"}`);
  if (d.sizeBytes > 0) parts.push(fmtBytes(d.sizeBytes));
  if (!d.hasFile) parts.push("no file attached");
  return parts.join(" · ") || "—";
};

const clock12 = (date) => {
  let h = date.getHours();
  const ampm = h >= 12 ? "PM" : "AM";
  h = h % 12 || 12;
  return `${h}:${String(date.getMinutes()).padStart(2, "0")} ${ampm}`;
};

const fmtSubmitted = (iso) => {
  const d = new Date(iso);
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `Submitted ${mm}/${dd}/${d.getFullYear()} ${clock12(d)}`;
};

const ago = (iso) => {
  const d = new Date(iso);
  const now = new Date();
  const mins = Math.max(0, Math.round((now - d) / 60000));
  if (mins < 1) return "just now";
  if (mins < 60) return `${mins} min ago`;
  const startOfDay = (x) => new Date(x.getFullYear(), x.getMonth(), x.getDate());
  const dayDiff = Math.round((startOfDay(now) - startOfDay(d)) / 86400000);
  if (dayDiff === 0) return `Today, ${clock12(d)}`;
  if (dayDiff === 1) return `Yesterday, ${clock12(d)}`;
  return `${String(d.getMonth() + 1).padStart(2, "0")}/${String(d.getDate()).padStart(2, "0")}, ${clock12(d)}`;
};

const MONTHS = ["Sep", "Oct", "Nov", "Dec", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"];

/* ── Derived state ─────────────────────────────────────────────────── */

const statusOf = (d) => d.status || "awaiting";
const wantedStatus = { Awaiting: "awaiting", Approved: "approved", Rejected: "rejected" };

const counts = () => ({
  Awaiting: state.deals.filter((d) => statusOf(d) === "awaiting").length,
  Approved: state.deals.filter((d) => statusOf(d) === "approved").length,
  Rejected: state.deals.filter((d) => statusOf(d) === "rejected").length,
});

const matchesSearch = (d, q) => {
  if (!q) return true;
  const hay = [d.company, d.puller, ...(d.lenders || [])].join(" ").toLowerCase();
  return hay.includes(q.toLowerCase());
};

const visible = () =>
  state.deals.filter(
    (d) => statusOf(d) === wantedStatus[state.filter] && matchesSearch(d, state.search)
  );

const selected = () => state.deals.find((d) => d.id === state.selId) || null;

/// Deal whose content the panel shows — the live selection, or the last
/// selection while the panel animates closed (so it doesn't flash empty).
const panelDeal = () =>
  state.deals.find((d) => d.id === (state.selId ?? state.lastSelId)) || null;

/* ── Rendering ─────────────────────────────────────────────────────── */

function renderAll() {
  renderSidebar();
  renderHeader();
  renderStats();
  renderTabs();
  renderList();
  renderPanel();
  renderOverlay();
}

/* Theme — data-theme on the document root drives every color through the
   token set; swapping it recolors instantly with no re-render. */

const MOON_ICON =
  `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M20.5 14.2A8.6 8.6 0 0 1 9.8 3.5a8.7 8.7 0 1 0 10.7 10.7z"></path></svg>`;
const SUN_ICON =
  `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"></circle><path d="M12 2.8v2.1M12 19.1v2.1M2.8 12h2.1M19.1 12h2.1M5.5 5.5l1.5 1.5M17 17l1.5 1.5M18.5 5.5 17 7M7 17l-1.5 1.5"></path></svg>`;

const currentTheme = () =>
  document.documentElement.dataset.theme === "dark" ? "dark" : "light";

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
  renderSidebar();
}

function renderSidebar() {
  const sidebar = document.getElementById("sidebar");
  sidebar.classList.toggle("open", state.navOpen);
  sidebar.querySelectorAll("[data-page]").forEach((item) => {
    const page = item.dataset.page;
    item.classList.toggle("active", state.navPage === page);
    // Tooltip shows on hover only while collapsed, driven by navHover state.
    item.querySelector(".nav-tip")
      .classList.toggle("show", !state.navOpen && state.navHover === page);
  });
  // The theme row reads the *destination* theme: moon/"Dark mode" while
  // light is active, sun/"Light mode" while dark is.
  const dark = currentTheme() === "dark";
  const themeLabel = dark ? "Light mode" : "Dark mode";
  document.getElementById("theme-icon").innerHTML = dark ? SUN_ICON : MOON_ICON;
  document.getElementById("theme-label").textContent = themeLabel;
  document.getElementById("theme-tip").textContent = themeLabel;
  document.getElementById("side-avatar").textContent =
    state.reviewer.slice(0, 2).toUpperCase();
  document.getElementById("side-foot-name").textContent = state.reviewer;
}

function renderHeader() {
  document.getElementById("page-title").textContent =
    state.filter === "Approved" ? "Submitted deals"
    : state.filter === "Rejected" ? "Rejected submissions"
    : "Awaiting decision";
}

function renderStats() {
  document.getElementById("stat-queue").textContent = counts().Awaiting;
  document.getElementById("stat-submitted").textContent = state.autoSubmitted;
}

function renderTabs() {
  const c = counts();
  document.getElementById("tabs").innerHTML = ["Awaiting", "Approved", "Rejected"]
    .map(
      (name) =>
        `<div class="tab${state.filter === name ? " active" : ""}" data-action="tab" data-tab="${name}">${name}  ${c[name]}</div>`
    )
    .join("");
}

function dealCard(d) {
  const sel = d.id === state.selId;
  const nsfEmphasis = d.nsf >= 5;
  const decided = statusOf(d) !== "awaiting";
  const decidedLabel = decided
    ? `${statusOf(d) === "approved" ? "Approved" : "Rejected"} by ${esc(d.decidedBy || "")}`
    : "";
  const decidedCluster = decided
    ? `<div class="decided-cluster">
         <span class="decided-label">${decidedLabel}</span>
         <div class="decided-badge ${statusOf(d)}">${statusOf(d) === "approved" ? "✓" : "✕"}</div>
       </div>`
    : "";
  return `
  <div class="deal-card${sel ? " selected" : ""}" data-action="select-deal" data-id="${d.id}">
    <div class="deal-top">
      <div class="avatar risk-${esc(d.risk)}">${esc(d.initials)}</div>
      <div class="deal-main">
        <div class="deal-company">${esc(d.company)}</div>
        <div class="deal-meta">
          <span>${esc(d.industry ?? "—")}</span><span class="dot">·</span>
          <span>${esc(d.state ?? "—")}</span><span class="dot">·</span>
          <span>${esc(d.tib ?? "—")} in business</span><span class="dot">·</span>
          <span>Position ${esc(d.position)}</span>
        </div>
      </div>
      <div class="deal-right">
        <div class="deal-ago">${ago(d.submittedAt)}</div>
      </div>
    </div>
    <div class="deal-chips">
      <div class="chip"><span class="chip-key">FICO</span> ${esc(d.fico ?? "—")}</div>
      <div class="chip"><span class="chip-key">ADB</span> ${esc(d.adb ?? "—")}</div>
      <div class="chip${nsfEmphasis ? " emph" : ""}"><span class="chip-key">NSF</span> ${esc(d.nsf)}</div>
      <div class="chip">${(d.lenders || []).length} lenders matched</div>
      ${decidedCluster}
    </div>
  </div>`;
}

function renderList() {
  const list = visible();
  const el = document.getElementById("deal-list");
  if (list.length === 0) {
    const searching = state.search.trim().length > 0;
    const copy = searching
      ? { title: "No matches", body: `No ${state.filter.toLowerCase()} deals match “${esc(state.search.trim())}”.` }
      : state.filter === "Awaiting"
        ? { title: "Queue cleared", body: "Every submission has a decision. New deals land here automatically." }
        : state.filter === "Approved"
          ? { title: "Nothing submitted yet", body: "Approved deals move here once the packet goes out to lenders." }
          : { title: "Nothing rejected yet", body: "Rejected deals are kept here with the reviewer who declined them." };
    el.innerHTML = `
      <div class="empty">
        <div class="empty-title">${copy.title}</div>
        <div class="empty-body">${copy.body}</div>
      </div>`;
    return;
  }
  el.innerHTML = list.map(dealCard).join("");
}

function fieldCell(k, v, cls = "") {
  return `
  <div class="field">
    <span class="field-k">${esc(k)}</span>
    <span class="field-v ${cls}">${v}</span>
  </div>`;
}

function dealFields(sel) {
  const dim = (v) => (v == null || v === "N/A" || v === "None" ? "dim" : "");
  const val = (v) => esc(v ?? "N/A");
  // Field order matches the source submission sheet (reading left→right).
  // Credit score and NSFs stay plain ink regardless of value — no thresholds.
  return [
    fieldCell("Company", val(sel.company)),
    fieldCell("Lead source", val(sel.leadSource), dim(sel.leadSource)),
    fieldCell(
      "Email",
      sel.email ? `<a href="mailto:${esc(sel.email)}">${esc(sel.email)}</a>` : "N/A",
      sel.email ? "" : "dim"
    ),
    fieldCell("Credit score", val(sel.fico)),
    fieldCell("Phone", val(sel.phone), dim(sel.phone)),
    fieldCell("Industry", val(sel.industry), dim(sel.industry)),
    fieldCell("Puller", val(sel.puller), dim(sel.puller)),
    fieldCell("Revenue", val(sel.revenue), dim(sel.revenue)),
    fieldCell("Re-puller", val(sel.rePuller), dim(sel.rePuller)),
    fieldCell("Avg daily balance", val(sel.adb), dim(sel.adb)),
    fieldCell("State", val(sel.state), dim(sel.state)),
    fieldCell("Deposits", val(sel.deposits), dim(sel.deposits)),
    fieldCell("Time in business", val(sel.tib), dim(sel.tib)),
    fieldCell("NSFs", val(sel.nsf)),
  ].join("");
}

const fmtMoney = (v) =>
  typeof v === "number" ? "$" + Math.round(v).toLocaleString("en-US") : esc(v);

/// The submission sheet's month-by-month seasonality calc, rendered as a
/// table. Columns with no data in any row are dropped; Revenue is dropped
/// when it just mirrors the deposits column.
function seasonCalcTable(rows) {
  const cell = (v) => esc(v);
  const revenueMirrorsDeposits = rows.every(
    (r) => r?.totalMonthlyRevenue == null || r.totalMonthlyRevenue === r.monthlyDeposits
  );
  const cols = [
    { key: "month", label: "Month", fmt: (v, r) => esc(r?.year ? `${v} ${r.year}` : v) },
    { key: "monthlyDeposits", label: "Deposits", fmt: fmtMoney },
    { key: "totalMonthlyRevenue", label: "Revenue", fmt: fmtMoney, skip: revenueMirrorsDeposits },
    { key: "depositsPerMonth", label: "# Deposits", fmt: cell },
    { key: "averageDailyBalance", label: "ADB", fmt: fmtMoney },
    { key: "ledger", label: "Ledger", fmt: fmtMoney },
    { key: "nsf", label: "NSF", fmt: (v) => `<span class="${Number(v) >= 5 ? "emph" : ""}">${esc(v)}</span>` },
  ].filter((c) => !c.skip && rows.some((r) => r && r[c.key] != null && r[c.key] !== ""));
  const tr = (r) =>
    `<tr>${cols
      .map((c) => {
        const v = r?.[c.key];
        const empty = v == null || v === "";
        return `<td>${empty ? '<span class="dim">—</span>' : c.fmt(v, r)}</td>`;
      })
      .join("")}</tr>`;
  return `
  <div class="season-table-wrap">
    <table class="season-table">
      <thead><tr>${cols.map((c) => `<th>${c.label}</th>`).join("")}</tr></thead>
      <tbody>${rows.map(tr).join("")}</tbody>
    </table>
  </div>`;
}

function seasonalitySection(sel) {
  const calcRows = Array.isArray(sel.seasonCalc)
    ? sel.seasonCalc.filter((r) => r && typeof r === "object")
    : [];
  if (calcRows.length) {
    return `
  <div>
    <div class="section-head">
      <div class="section-label">Seasonality · monthly deposits</div>
      <span class="section-head-note">${calcRows.length} month${calcRows.length === 1 ? "" : "s"}</span>
    </div>
    <div class="season-frame calc">${seasonCalcTable(calcRows)}</div>
  </div>`;
  }
  let body;
  if (Array.isArray(sel.season) && sel.season.length) {
    const max = Math.max(...sel.season);
    body = `<div class="season-bars">${sel.season
      .map((v, i) => {
        const last = i === sel.season.length - 1;
        const label = `${MONTHS[i % 12]} · ${v}`;
        return `<div class="season-bar${last ? " last" : ""}" style="height:${Math.round((v / max) * 100)}%" title="${esc(label)}"></div>`;
      })
      .join("")}</div>`;
  } else {
    body = `<div class="season-placeholder">Seasonality breakdown arrives with the payload</div>`;
  }
  return `
  <div>
    <div class="section-head">
      <div class="section-label">Seasonality · monthly deposits</div>
      <span class="section-head-note">trailing 12</span>
    </div>
    <div class="season-frame">${body}</div>
  </div>`;
}

function docsSection(sel) {
  const docs = sel.docs || [];
  const rows = state.docsOpen
    ? `<div class="doc-rows">${docs
        .map(
          (doc) => `
      <div class="doc-row" data-action="open-doc" data-doc-id="${doc.id}">
        <img class="doc-icon" src="assets/pdf-icon.png" alt="PDF">
        <div class="doc-main">
          <div class="doc-name">${esc(doc.name)}</div>
          <div class="doc-meta">${docMeta(doc)}</div>
        </div>
        <span class="doc-view">View</span>
      </div>`
        )
        .join("")}</div>`
    : "";
  return `
  <div>
    <div class="section-head docs-head" data-action="toggle-docs">
      <div class="section-label">Documents Submitted</div>
      <div class="doc-count">${docs.length}</div>
      <span class="docs-caret">${state.docsOpen ? "▾" : "▸"}</span>
    </div>
    ${rows}
  </div>`;
}

function lendersSection(sel) {
  const lenders = sel.lenders || [];
  // Every matched lender renders as a green chip — all of them are matches.
  return `
  <div>
    <div class="section-label" style="margin-bottom:10px;">Suggested lenders · auto-matched</div>
    <div class="lender-chips">
      ${lenders.map((name) => `<div class="lender-chip">${esc(name)}</div>`).join("")}
    </div>
  </div>`;
}

function footerActions(sel) {
  if (!sel) return "";
  if (statusOf(sel) !== "awaiting") {
    return `<div class="actions">
      <div class="decision-banner ${statusOf(sel)}">${esc(sel.decision || "")}</div>
    </div>`;
  }
  if (state.rejecting) {
    return `<div class="reject-capture">
      <input id="reject-reason" class="reject-input" type="text"
             placeholder="Reason — e.g. NSF volume" autocomplete="off" spellcheck="false">
      <button class="btn btn-reject btn-confirm-reject" data-action="confirm-reject">Confirm reject</button>
      <button class="btn btn-cancel" data-action="cancel-reject">Cancel</button>
    </div>`;
  }
  return `<div class="actions">
    <button class="btn btn-reject" data-action="reject">Reject</button>
    <button class="btn btn-approve" data-action="approve">Approve</button>
  </div>`;
}

function renderPanel() {
  const wrap = document.getElementById("panel-wrap");
  wrap.classList.toggle("open", state.selId != null);
  const sel = panelDeal();
  const panel = document.getElementById("panel");

  const eyebrow = !sel
    ? { cls: "awaiting", text: "No deal selected" }
    : statusOf(sel) === "approved"
      ? { cls: "approved", text: "Submitted to lenders" }
      : statusOf(sel) === "rejected"
        ? { cls: "rejected", text: "Rejected" }
        : { cls: "awaiting", text: "New deal submitted" };

  const header = `
  <div class="panel-header">
    <div class="panel-eyebrow-row">
      <span class="panel-eyebrow ${eyebrow.cls}">${eyebrow.text}</span>
      <span class="panel-submitted">${sel ? fmtSubmitted(sel.submittedAt) : ""}</span>
      <button class="panel-close" data-action="close-panel" title="Close">✕</button>
    </div>
    <h2>${esc(sel ? sel.company : "Nothing to review")}</h2>
    <div class="panel-chips">
      <div class="panel-chip matched">${sel ? (sel.lenders || []).length : 0} lenders matched</div>
      <div class="panel-chip position"><span>Position</span><span>${sel ? esc(sel.position) : "—"}</span></div>
    </div>
  </div>`;

  const body = sel
    ? `
  <div class="panel-body">
    <div>
      <div class="section-label" style="margin-bottom:9px;">Deal information</div>
      <div class="fields">${dealFields(sel)}</div>
    </div>
    ${docsSection(sel)}
    ${seasonalitySection(sel)}
    ${lendersSection(sel)}
    <div class="notes">
      <div class="section-label">Notes</div>
      <div class="note">${esc(sel.note || "—")}</div>
      <div class="note santi"><span class="prefix">Santi's note · </span>${esc(sel.santiNote || "Nothing pending.")}</div>
    </div>
  </div>`
    : `
  <div class="panel-body">
    <div class="notes">
      <div class="section-label">Notes</div>
      <div class="note">The desk is empty.</div>
      <div class="note santi"><span class="prefix">Santi's note · </span>Nothing pending.</div>
    </div>
  </div>`;

  const sessionLabel = state.decided
    ? `${state.decided} decided this session`
    : "No decisions yet";

  const footer = `
  <div class="panel-footer">
    <div class="deciding-row">
      <span class="deciding-label">Deciding as</span>
      <div class="reviewer-seg">
        ${["Santi", "Careem"]
          .map(
            (name) =>
              `<div class="reviewer-opt${state.reviewer === name ? " active" : ""}" data-action="reviewer" data-name="${name}">${name}</div>`
          )
          .join("")}
      </div>
      <span class="session-counter">${sessionLabel}</span>
    </div>
    ${footerActions(sel)}
  </div>`;

  panel.innerHTML = header + body + footer;

  if (state.rejecting) {
    const input = document.getElementById("reject-reason");
    if (input) {
      input.focus();
      input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") confirmReject();
        if (e.key === "Escape") {
          e.stopPropagation();
          state.rejecting = false;
          renderPanel();
        }
      });
    }
  }
}

/* ── PDF viewer overlay ────────────────────────────────────────────── */

function renderOverlay() {
  const root = document.getElementById("overlay-root");
  const v = state.viewing;
  if (!v) {
    root.innerHTML = "";
    return;
  }
  root.innerHTML = `
  <div class="overlay" data-action="close-viewer">
    <div class="pdf-dialog" data-stop-close>
      <div class="pdf-dialog-header">
        <img class="pdf-dialog-icon" src="assets/pdf-icon.png" alt="PDF">
        <div class="pdf-dialog-title">
          <div class="pdf-dialog-name">${esc(v.name)}</div>
          <div class="pdf-dialog-meta">${esc(v.meta)} · ${esc(v.company)}</div>
        </div>
        <div class="pdf-dialog-actions">
          <button class="pill-btn" data-action="download-doc">Download</button>
          <button class="pill-btn round" data-action="close-viewer">✕</button>
        </div>
      </div>
      <div class="pdf-dialog-body">
        <div class="pdf-page-frame" id="pdf-page-frame">
          <div class="pdf-loading">${v.hasFile ? "Loading document…" : "No file arrived with this document"}</div>
        </div>
      </div>
    </div>
  </div>`;
  if (!v.hasFile) return;
  const url = docFileUrl(v.docId);
  if (v.isPdf) loadPdf(url);
  else if (v.isImage) {
    document.getElementById("pdf-page-frame").innerHTML =
      `<img src="${url}" alt="${esc(v.name)}" style="max-width:100%;display:block;margin:0 auto;">`;
  } else loadTextPreview(url);
}

/// Non-PDF files (CSV bank exports, plain text) — fetched and shown as
/// preformatted text inside the same dialog.
async function loadTextPreview(url) {
  try {
    const resp = await fetch(url);
    if (!resp.ok) throw new Error(`service answered ${resp.status}`);
    const text = await resp.text();
    const current = document.getElementById("pdf-page-frame");
    if (!current) return; // overlay closed
    current.innerHTML = `<pre class="text-preview">${esc(text)}</pre>`;
  } catch (e) {
    console.error("text preview failed, falling back to <embed>:", e);
    const current = document.getElementById("pdf-page-frame");
    if (current) {
      current.innerHTML = `<embed src="${url}"
        style="width:100%;height:100%;min-height:640px;">`;
    }
  }
}

async function loadPdf(url) {
  const frame = document.getElementById("pdf-page-frame");
  try {
    const pdfjs = await import("./vendor/pdfjs/pdf.min.mjs");
    pdfjs.GlobalWorkerOptions.workerSrc = new URL(
      "./vendor/pdfjs/pdf.worker.min.mjs",
      import.meta.url
    ).toString();
    const pdf = await pdfjs.getDocument(url).promise;
    if (!document.getElementById("pdf-page-frame")) return; // overlay closed
    frame.innerHTML = "";
    const width = frame.clientWidth || 768;
    for (let i = 1; i <= pdf.numPages; i++) {
      const page = await pdf.getPage(i);
      const base = page.getViewport({ scale: 1 });
      const scale = (width / base.width) * (window.devicePixelRatio || 1);
      const viewport = page.getViewport({ scale });
      const canvas = document.createElement("canvas");
      canvas.width = viewport.width;
      canvas.height = viewport.height;
      const current = document.getElementById("pdf-page-frame");
      if (!current) return;
      current.appendChild(canvas);
      await page.render({ canvasContext: canvas.getContext("2d"), viewport }).promise;
    }
  } catch (e) {
    console.error("pdf.js failed, falling back to <embed>:", e);
    const current = document.getElementById("pdf-page-frame");
    if (current) {
      current.innerHTML = `<embed src="${url}" type="application/pdf"
        style="width:100%;height:100%;min-height:640px;">`;
    }
  }
}

/* ── Actions ───────────────────────────────────────────────────────── */

function upsertDeal(deal) {
  const i = state.deals.findIndex((d) => d.id === deal.id);
  // The auto-submitted stat counts approval transitions, so the SSE echo of
  // a decision this desk already applied doesn't double-count.
  const wasApproved = i >= 0 && statusOf(state.deals[i]) === "approved";
  if (statusOf(deal) === "approved" && !wasApproved) state.autoSubmitted += 1;
  if (i >= 0) state.deals[i] = deal;
  else state.deals.push(deal);
  state.deals.sort(
    (a, b) => new Date(b.submittedAt) - new Date(a.submittedAt) || b.id - a.id
  );
}

async function decide(kind, reason) {
  const sel = selected();
  if (!sel || statusOf(sel) !== "awaiting") return;
  try {
    const updated = await invoke(kind === "approve" ? "approve_deal" : "reject_deal", {
      id: sel.id,
      reviewer: state.reviewer,
      ...(kind === "reject" ? { reason: reason || null } : {}),
    });
    upsertDeal(updated);
    state.decided += 1;
    state.filter = "Awaiting";
    state.rejecting = false;
    // Deciding closes the panel; lastSelId keeps the decided deal's content
    // mounted so the collapse animation shows the decision banner, not a blank.
    state.selId = null;
    renderAll();
  } catch (e) {
    console.error("decision failed:", e);
    state.rejecting = false;
    renderAll();
  }
}

function confirmReject() {
  const reason = document.getElementById("reject-reason")?.value?.trim() || "";
  decide("reject", reason);
}

function openDoc(docId) {
  const sel = panelDeal();
  const doc = (sel?.docs || []).find((d) => d.id === Number(docId));
  if (!doc || !sel) return;
  state.viewing = {
    docId: doc.id,
    name: doc.name,
    meta: docMeta(doc),
    company: sel.company,
    hasFile: doc.hasFile,
    isPdf: isPdfDoc(doc),
    isImage: (doc.contentType || "").startsWith("image/"),
  };
  renderOverlay();
}

/* ── Event wiring ──────────────────────────────────────────────────── */

document.addEventListener("click", (e) => {
  const stop = e.target.closest("[data-stop-close]");
  const el = e.target.closest("[data-action]");
  if (!el) {
    return;
  }
  const action = el.dataset.action;
  switch (action) {
    case "tab":
      state.filter = el.dataset.tab;
      renderHeader();
      renderTabs();
      renderList();
      break;
    case "select-deal": {
      // Never a toggle: clicking a card always opens/swaps the panel. A
      // toggling handler broke the prototype (double-fired click closed the
      // panel within one frame). Close only via the panel's ✕.
      const id = Number(el.dataset.id);
      state.selId = id;
      state.lastSelId = id;
      state.rejecting = false;
      renderList();
      renderPanel();
      break;
    }
    case "close-panel":
      state.selId = null;
      state.rejecting = false;
      renderList();
      renderPanel();
      break;
    case "toggle-nav":
      state.navOpen = !state.navOpen;
      state.navHover = null;
      renderSidebar();
      break;
    case "toggle-theme": {
      // Instant swap, no crossfade; persist to the cache and the app's
      // settings store on every toggle.
      const next = currentTheme() === "dark" ? "light" : "dark";
      applyTheme(next);
      try { localStorage.setItem("capdesk-theme", next); } catch (e) {}
      invoke("set_theme", { theme: next }).catch((e) =>
        console.error("theme save failed:", e));
      break;
    }
    case "nav":
      state.navPage = el.dataset.page;
      renderSidebar();
      break;
    case "reviewer":
      state.reviewer = el.dataset.name;
      renderSidebar();
      renderPanel();
      break;
    case "toggle-docs":
      state.docsOpen = !state.docsOpen;
      renderPanel();
      break;
    case "open-doc":
      openDoc(el.dataset.docId);
      break;
    case "close-viewer":
      // Scrim click closes; clicks inside the dialog (except ✕) do not.
      if (el.classList.contains("overlay") && stop) break;
      state.viewing = null;
      renderOverlay();
      break;
    case "download-doc":
      if (state.viewing) {
        window.__TAURI__.opener.openUrl(docFileUrl(state.viewing.docId));
      }
      break;
    case "approve":
      decide("approve");
      break;
    case "reject":
      state.rejecting = true;
      renderPanel();
      break;
    case "confirm-reject":
      confirmReject();
      break;
    case "cancel-reject":
      state.rejecting = false;
      renderPanel();
      break;
    case "win-min":
      appWindow.minimize();
      break;
    case "win-max":
      appWindow.toggleMaximize();
      break;
    case "win-close":
      appWindow.close();
      break;
  }
});

// Collapsed-sidebar tooltips ride on navHover state (mouseenter/leave don't
// bubble, so each nav item — and the theme row — gets its own listeners).
document.querySelectorAll("[data-page]").forEach((item) => {
  item.addEventListener("mouseenter", () => {
    state.navHover = item.dataset.page;
    renderSidebar();
  });
  item.addEventListener("mouseleave", () => {
    state.navHover = null;
    renderSidebar();
  });
});

document.getElementById("search").addEventListener("input", (e) => {
  state.search = e.target.value;
  renderList();
});

document.addEventListener("keydown", (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
    e.preventDefault();
    document.getElementById("search").focus();
    return;
  }
  if (e.key === "Escape") {
    if (state.viewing) {
      state.viewing = null;
      renderOverlay();
    } else if (state.rejecting) {
      state.rejecting = false;
      renderPanel();
    } else if (state.selId != null) {
      state.selId = null;
      renderList();
      renderPanel();
    }
  }
});

/* ── Live events from the service ──────────────────────────────────── */

listen("server-event", (event) => {
  const msg = event.payload;
  if (!msg || !msg.type) return;
  if (msg.type === "deal.created" && msg.deal) {
    upsertDeal(msg.deal);
    renderAll();
  } else if (msg.type === "deal.decided" && msg.deal) {
    upsertDeal(msg.deal);
    renderAll();
  }
  // submission.completed → the packet went out; the approval already moved
  // the counters. Hook toasts here later.
});

// Desktop-notification click: jump to the deal. The deal may have been
// decided (or the filter/search changed) since the toast fired, so land on
// whichever tab it currently lives in and clear the search.
listen("open-deal", (event) => {
  const id = Number(event.payload);
  const deal = state.deals.find((d) => d.id === id);
  if (!deal) return;
  state.filter =
    statusOf(deal) === "approved" ? "Approved"
    : statusOf(deal) === "rejected" ? "Rejected"
    : "Awaiting";
  state.search = "";
  document.getElementById("search").value = "";
  state.selId = id;
  state.lastSelId = id;
  state.rejecting = false;
  renderAll();
});

/* ── Boot ──────────────────────────────────────────────────────────── */

(async function boot() {
  renderSidebar();
  // The inline <head> script already applied the cached/OS theme; reconcile
  // with the Rust-side settings store, which survives a WebView data reset.
  try {
    const stored = await invoke("get_theme");
    if (stored === "light" || stored === "dark") {
      if (stored !== currentTheme()) applyTheme(stored);
      try { localStorage.setItem("capdesk-theme", stored); } catch (e) {}
    }
  } catch (e) {
    console.error("theme load failed:", e);
  }
  try {
    const snapshot = await invoke("desk_state");
    state.baseUrl = snapshot.baseUrl;
    state.fileKey = snapshot.fileKey || "";
    state.deals = snapshot.deals || [];
    state.autoSubmitted = snapshot.stats?.autoSubmitted ?? 0;
    state.deals.sort(
      (a, b) => new Date(b.submittedAt) - new Date(a.submittedAt) || b.id - a.id
    );
    // Panel starts closed; a deal opens it on click.
  } catch (e) {
    console.error(e);
    document.getElementById("deal-list").innerHTML = `
      <div class="empty">
        <div class="empty-title">Service unreachable</div>
        <div class="empty-body">${esc(String(e))}</div>
      </div>`;
    document.getElementById("panel").innerHTML = "";
    return;
  }
  renderAll();
  // Keep relative timestamps fresh.
  setInterval(renderList, 30000);
})();
