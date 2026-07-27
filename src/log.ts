import { getLogs } from "./api";

let timer: number | undefined;

function ansiToHtml(text: string): string {
  // Strip OSC 8 hyperlink sequences.
  text = text.replace(/\x1b\]8;[^\x07\x1b]*(\x07|\x1b\\)/g, "");

  const colorMap: Record<number, string> = {
    1: "font-weight:bold",
    2: "opacity:0.7",
    3: "font-style:italic",
    4: "text-decoration:underline",
    30: "color:#111827",
    31: "color:#ef4444",
    32: "color:#22c55e",
    33: "color:#f59e0b",
    34: "color:#3b82f6",
    35: "color:#a855f7",
    36: "color:#06b6d4",
    37: "color:#e6edf3",
    90: "color:#6b7280",
    91: "color:#fca5a5",
    92: "color:#86efac",
    93: "color:#fde68a",
    94: "color:#93c5fd",
    95: "color:#c4b5fd",
    96: "color:#67e8f9",
  };

  let html = text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");

  const re = /\x1b\[(\d+(?:;\d+)*)m/g;
  const parts: string[] = [];
  let last = 0;
  let openTags: string[] = [];
  let match: RegExpExecArray | null;

  while ((match = re.exec(html)) !== null) {
    if (match.index > last) {
      parts.push(html.slice(last, match.index));
    }

    const codes = match[1].split(";").map(Number);

    if (codes.includes(0)) {
      for (let i = openTags.length - 1; i >= 0; i--) {
        parts.push(openTags[i].replace("<span", "</span").replace(/>.*/, ">"));
      }
      openTags = [];
    } else {
      const styles = codes
        .filter((c) => colorMap[c])
        .map((c) => colorMap[c]);
      if (styles.length > 0) {
        const tag = `<span style="${styles.join(";")}">`;
        parts.push(tag);
        openTags.push(tag);
      }
    }

    last = match.index + match[0].length;
  }

  if (last < html.length) {
    parts.push(html.slice(last));
  }

  for (let i = openTags.length - 1; i >= 0; i--) {
    parts.push(openTags[i].replace("<span", "</span").replace(/>.*/, ">"));
  }

  return parts.join("");
}

export function initLogPanel(container: HTMLElement) {
  container.innerHTML = `
    <div class="log-actions">
      <button id="log-refresh" class="btn small">刷新</button>
      <button id="log-copy" class="btn small">复制</button>
      <button id="log-clear" class="btn small">清除</button>
    </div>
    <pre id="log-text" class="log"></pre>
  `;

  const text = container.querySelector<HTMLPreElement>("#log-text")!;
  const refreshBtn = container.querySelector<HTMLButtonElement>("#log-refresh")!;
  const copyBtn = container.querySelector<HTMLButtonElement>("#log-copy")!;
  const clearBtn = container.querySelector<HTMLButtonElement>("#log-clear")!;

  const refresh = async () => {
    const raw = await getLogs(300);
    text.innerHTML = ansiToHtml(raw);
  };

  refreshBtn.addEventListener("click", refresh);

  copyBtn.addEventListener("click", async () => {
    const raw = text.textContent ?? "";
    await navigator.clipboard.writeText(raw);
    copyBtn.textContent = "已复制";
    setTimeout(() => (copyBtn.textContent = "复制"), 1500);
  });

  clearBtn.addEventListener("click", () => {
    text.innerHTML = "";
  });

  refresh();
  timer = window.setInterval(refresh, 2000);
}

export function stopLogPolling() {
  if (timer) window.clearInterval(timer);
}
