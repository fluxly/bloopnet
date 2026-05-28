// bundler target auto-initializes; no explicit init() call needed.
import {
  encodeText,
  decodeBloopText,
  inspectText,
} from "bloop-wasm";

// ── Types matching WasmInspectionReport from bloop-wasm ───────────────────

interface ValidationIssue {
  kind: string;
  position: number | null;
  message: string;
}

interface InspectionReport {
  text: string;
  symbols: number;
  bits: number;
  payloadBytes: number;
  valid: boolean;
  issues: ValidationIssue[];
  sizeClass: string;
  loraPolite: boolean;
}

// ── DOM refs ──────────────────────────────────────────────────────────────

const inputEl = document.getElementById("input") as HTMLTextAreaElement;
const outputEl = document.getElementById("output") as HTMLDivElement;
const charCountEl = document.getElementById("char-count") as HTMLSpanElement;
const sizeBadgeEl = document.getElementById("size-badge") as HTMLSpanElement;

// ── Helpers ───────────────────────────────────────────────────────────────

function toHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function esc(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function row(label: string, value: string, cls = ""): string {
  return `<span class="report-label">${label}</span><span class="report-value ${cls}">${value}</span>`;
}

// ── Render ────────────────────────────────────────────────────────────────

function render(text: string): void {
  if (!text) {
    outputEl.innerHTML = "";
    charCountEl.textContent = "0";
    sizeBadgeEl.textContent = "";
    sizeBadgeEl.className = "size-badge";
    return;
  }

  // inspectText never throws — it reports issues for invalid input.
  const report = inspectText(text) as unknown as InspectionReport;

  // Update char counter and size badge.
  charCountEl.textContent = String(report.symbols);
  sizeBadgeEl.textContent = report.sizeClass;
  const badgeCls = report.loraPolite
    ? "polite"
    : report.sizeClass === "too long"
    ? "over"
    : "warn";
  sizeBadgeEl.className = `size-badge ${badgeCls}`;

  // Get encoded bytes if the input is valid.
  let hexStr = "";
  let roundTrip = "";
  if (report.valid) {
    const enc = encodeText(text);
    hexStr = toHex(enc.bytes);
    roundTrip = decodeBloopText(enc.bytes, enc.symbolCount);
  }

  const validClass = report.valid ? "ok" : "no";
  const validText = report.valid ? "yes" : "no";
  const loraText = report.loraPolite ? "yes" : "no";
  const loraCls = report.loraPolite ? "ok" : "warn";

  // Build report rows.
  const rows = [
    row("symbols", String(report.symbols)),
    row("bits", String(report.bits)),
    row("payload bytes", String(report.payloadBytes)),
    row("size class", report.sizeClass),
    row("lora polite", loraText, loraCls),
    row("valid", validText, validClass),
  ].join("\n");

  // Hex row (only shown for valid input).
  const hexRow = hexStr
    ? `<div class="hex-row">
        <span class="hex-label">hex</span>
        <span class="hex-value">${esc(hexStr)}</span>
        <button class="copy-btn" data-hex="${esc(hexStr)}">copy</button>
       </div>`
    : "";

  // Decoded section (proves round-trip).
  const decodedSection =
    roundTrip !== ""
      ? `<div class="decoded-section">
           <div class="decoded-label">decoded ›</div>
           <div class="decoded-text">${esc(roundTrip)}</div>
         </div>`
      : "";

  // Issues section.
  const issueItems = report.issues
    .map(
      (i) =>
        `<div class="issue"><span class="issue-marker">!</span><span>${esc(i.message)}</span></div>`
    )
    .join("\n");
  const issuesSection = issueItems
    ? `<div class="issues-section">${issueItems}</div>`
    : "";

  outputEl.innerHTML = `
    <div class="report">${rows}</div>
    ${hexRow}
    ${decodedSection}
    ${issuesSection}
  `;
}

// ── Copy button ───────────────────────────────────────────────────────────

outputEl.addEventListener("click", (e) => {
  const btn = (e.target as Element).closest(".copy-btn") as HTMLButtonElement | null;
  if (!btn) return;
  const hex = btn.dataset.hex ?? "";
  navigator.clipboard.writeText(hex).then(() => {
    btn.textContent = "copied";
    btn.classList.add("copied");
    setTimeout(() => {
      btn.textContent = "copy";
      btn.classList.remove("copied");
    }, 1500);
  });
});

// ── Init ──────────────────────────────────────────────────────────────────

// bundler target initializes synchronously on import; wire up events directly.
inputEl.addEventListener("input", () => render(inputEl.value));
inputEl.focus();
