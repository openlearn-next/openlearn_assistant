import "./styles.css";
import {
  detectNode,
  getStatus,
  provisionNode,
  cleanData,
  listVersions,
  startService,
  stopService,
  type NodeInfo,
  type Status,
} from "./api";
import { initSettings, renderSettings } from "./config";
import { initLogPanel } from "./log";

const app = document.querySelector<HTMLDivElement>("#app")!;

app.innerHTML = `
  <header class="topbar">
    <h1>OpenLearn-Next 助手</h1>
    <span id="st-run-dot" class="dot"></span>
  </header>

  <section class="card actions">
    <button id="btn-node" class="btn">安装 Node 22</button>
    <select id="cfg-version" class="ver-select"></select>
    <button id="cfg-more-versions" class="btn small">更多</button>
    <button id="btn-start" class="btn primary">启动</button>
    <button id="btn-stop" class="btn danger">停止</button>
    <button id="btn-clean" class="btn">清除数据</button>
    <a id="btn-open" class="btn ghost" target="_blank" rel="noreferrer">打开</a>
  </section>

  <section class="card status-bar">
    <span><span class="k">Node</span> <span id="st-node" class="v">检测中…</span></span>
    <span class="sep">·</span>
    <span><span class="k">远端</span> <span id="st-pkg" class="v">检测中…</span></span>
    <span class="sep">·</span>
    <span id="st-url-wrap"><a id="st-url" target="_blank" rel="noreferrer">—</a></span>
  </section>

  <section class="card" id="settings"></section>

  <section class="card log-panel" id="log-card">
    <div class="log-toggle" id="log-toggle">
      <span class="log-arrow">▶</span> 日志
    </div>
    <div id="log-body" class="log-collapsed"></div>
  </section>

  <div id="toast" class="toast hidden"></div>
`;

const el = {
  node: document.querySelector<HTMLSpanElement>("#st-node")!,
  pkg: document.querySelector<HTMLSpanElement>("#st-pkg")!,
  url: document.querySelector<HTMLAnchorElement>("#st-url")!,
  urlWrap: document.querySelector<HTMLSpanElement>("#st-url-wrap")!,
  runDot: document.querySelector<HTMLSpanElement>("#st-run-dot")!,
  btnNode: document.querySelector<HTMLButtonElement>("#btn-node")!,
  btnStart: document.querySelector<HTMLButtonElement>("#btn-start")!,
  btnStop: document.querySelector<HTMLButtonElement>("#btn-stop")!,
  btnClean: document.querySelector<HTMLButtonElement>("#btn-clean")!,
  btnOpen: document.querySelector<HTMLAnchorElement>("#btn-open")!,
  toast: document.querySelector<HTMLDivElement>("#toast")!,
};

let busy = false;

function toast(msg: string, kind: "ok" | "err" = "ok") {
  el.toast.textContent = msg;
  el.toast.className = `toast ${kind}`;
  setTimeout(() => (el.toast.className = "toast hidden"), 3500);
}

async function run(label: string, fn: () => Promise<void>) {
  if (busy) return;
  busy = true;
  setButtonsDisabled(true);
  try {
    await fn();
    toast(`${label} 完成`);
  } catch (e) {
    toast(`${label} 失败：${(e as Error).message ?? e}`, "err");
  } finally {
    busy = false;
    setButtonsDisabled(false);
    await refresh();
  }
}

function setButtonsDisabled(d: boolean) {
  [el.btnNode, el.btnStart, el.btnStop, el.btnClean].forEach(
    (b) => (b.disabled = d)
  );
}

function updateUI(node: NodeInfo, st: Status) {
  el.node.textContent = node.installed
    ? `${node.version}${node.meets_requirement ? "" : " ✗"}`
    : "未安装";
  el.node.className = `v ${node.meets_requirement ? "ok" : "bad"}`;

  el.pkg.textContent = st.version ?? "—";
  el.pkg.className = "v";

  el.runDot.className = st.running ? "dot running" : "dot";

  const url = `http://localhost:${st.port}/`;
  if (st.running) {
    el.url.href = url;
    el.url.textContent = url;
    el.urlWrap.style.display = "";
  } else {
    el.url.textContent = "—";
    el.url.removeAttribute("href");
  }

  el.btnOpen.href = url;
  el.btnOpen.style.display = st.running ? "" : "none";

  el.btnNode.style.display = node.meets_requirement ? "none" : "";
  el.btnStart.disabled = st.running;
  el.btnStop.disabled = !st.running;
}

async function refresh() {
  try {
    const [node, st] = await Promise.all([detectNode(), getStatus()]);
    updateUI(node, st);
  } catch (e) {
    toast(`状态刷新失败：${(e as Error).message ?? e}`, "err");
  }
}

el.btnNode.addEventListener("click", () =>
  run("安装 Node 22", provisionNode)
);
el.btnStart.addEventListener("click", () => run("启动", startService));
el.btnStop.addEventListener("click", () => run("停止", stopService));
el.btnClean.addEventListener("click", async () => {
  if (!confirm("将删除所有用户数据（数据库、上传文件、日志），确定？")) return;
  await run("清除数据", cleanData);
});

// --- Log toggle ---
let logExpanded = false;
document.querySelector<HTMLDivElement>("#log-toggle")!.addEventListener("click", () => {
  logExpanded = !logExpanded;
  const body = document.querySelector<HTMLDivElement>("#log-body")!;
  const arrow = document.querySelector<HTMLSpanElement>(".log-arrow")!;
  if (logExpanded) {
    body.className = "";
    arrow.textContent = "▼";
  } else {
    body.className = "log-collapsed";
    arrow.textContent = "▶";
  }
});

// --- Version selector (in actions row) ---
let runtimeOffset = 0;
const PAGE_SIZE = 10;

export async function appendVersions() {
  const select = document.querySelector<HTMLSelectElement>("#cfg-version");
  const moreBtn = document.querySelector<HTMLButtonElement>("#cfg-more-versions");
  if (!select || !moreBtn) return;
  const currentVal = select.value;

  try {
    const vers = await listVersions(runtimeOffset, PAGE_SIZE);
    if (runtimeOffset === 0) {
      const opt = document.createElement("option");
      opt.value = "latest";
      opt.textContent = "latest";
      select.appendChild(opt);
    }
    for (const v of vers) {
      const opt = document.createElement("option");
      opt.value = v;
      opt.textContent = v;
      select.appendChild(opt);
    }
    runtimeOffset += vers.length;
    if (vers.length < PAGE_SIZE) moreBtn.style.display = "none";
    select.value = currentVal;
  } catch {
    moreBtn.textContent = "加载失败";
    setTimeout(() => (moreBtn.textContent = "更多"), 2000);
  }
}

async function initRuntimeSettings() {
  const settings = await import("./api").then((m) => m.loadSettings());
  const moreBtn = document.querySelector<HTMLButtonElement>("#cfg-more-versions")!;
  moreBtn.addEventListener("click", appendVersions);
  await appendVersions();

  const select = document.querySelector<HTMLSelectElement>("#cfg-version")!;
  select.value = settings.version;

  // Version change triggers save to settings
  select.addEventListener("change", async () => {
    const s = await import("./api").then((m) => m.loadSettings());
    s.version = select.value;
    await import("./api").then((m) => m.saveSettings(s));
  });
}

async function main() {
  await initSettings();
  renderSettings(document.querySelector<HTMLElement>("#settings")!, refresh);
  setTimeout(() => initRuntimeSettings(), 0);
  initLogPanel(document.querySelector<HTMLElement>("#log-body")!);
  await refresh();
  window.setInterval(() => {
    if (!busy) refresh();
  }, 5000);
}

main();
