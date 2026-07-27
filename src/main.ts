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
    <span class="sub">Node 配置 · 启动 · 停止</span>
  </header>

  <section class="card" id="status-card">
    <div class="row"><span class="k">Node.js</span><span id="st-node" class="v">检测中…</span></div>
    <div class="row"><span class="k">远端版本</span><span id="st-pkg" class="v">检测中…</span></div>
    <div class="row"><span class="k">运行状态</span><span id="st-run" class="v">检测中…</span></div>
    <div class="row"><span class="k">访问地址</span><span id="st-url" class="v">—</span></div>
  </section>

  <section class="card actions">
    <button id="btn-node" class="btn">安装 Node 22</button>
    <button id="btn-start" class="btn primary">启动</button>
    <button id="btn-stop" class="btn danger">停止</button>
    <button id="btn-clean" class="btn">清除数据</button>
    <a id="btn-open" class="btn ghost" target="_blank" rel="noreferrer">打开</a>
  </section>

  <section class="card" id="settings"></section>

  <section class="card" id="runtime-card">
    <h2>运行时</h2>
    <label class="field checkbox-row">
      <input id="cfg-mirror" type="checkbox" checked />
      <span>使用中国镜像 (registry.npmmirror.com)</span>
    </label>
    <div class="field">
      <label for="cfg-version">版本</label>
      <select id="cfg-version"></select>
      <button id="cfg-more-versions" class="btn small" style="margin-top:4px">更多…</button>
    </div>
    <button id="cfg-runtime-save" class="btn" style="margin-top:8px">保存运行时设置</button>
    <span id="cfg-runtime-msg" class="hint"></span>
  </section>

  <section class="card">
    <div id="log-body"></div>
  </section>

  <div id="toast" class="toast hidden"></div>
`;

const el = {
  node: document.querySelector<HTMLSpanElement>("#st-node")!,
  pkg: document.querySelector<HTMLSpanElement>("#st-pkg")!,
  run: document.querySelector<HTMLSpanElement>("#st-run")!,
  url: document.querySelector<HTMLSpanElement>("#st-url")!,
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
    ? `${node.version}${node.meets_requirement ? " ✓" : " ✗ (<22)"}`
    : "未安装";
  el.node.className = `v ${node.meets_requirement ? "ok" : "bad"}`;

  el.pkg.textContent = st.version ?? "获取失败";
  el.pkg.className = "v";

  el.run.textContent = st.running
    ? `运行中 (PID ${st.pid ?? "?"})`
    : "已停止";
  el.run.className = `v ${st.running ? "ok" : ""}`;

  const url = `http://localhost:${st.port}/`;
  el.url.innerHTML = st.running
    ? `<a href="${url}" target="_blank" rel="noreferrer">${url}</a>`
    : "—";

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

// --- Runtime settings: mirror toggle + version selector ---

let runtimeOffset = 0;
const PAGE_SIZE = 10;

async function appendVersions() {
  const select = document.querySelector<HTMLSelectElement>("#cfg-version")!;
  const moreBtn = document.querySelector<HTMLButtonElement>("#cfg-more-versions")!;
  const currentVal = select.value;

  try {
    const vers = await listVersions(runtimeOffset, PAGE_SIZE);
    if (runtimeOffset === 0) {
      // First page: include "latest" option
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

    if (vers.length < PAGE_SIZE) {
      moreBtn.style.display = "none";
    }

    // Restore selection
    select.value = currentVal;
  } catch (e) {
    moreBtn.textContent = "加载失败";
    setTimeout(() => (moreBtn.textContent = "更多…"), 2000);
  }
}

async function initRuntimeSettings() {
  const settings = await import("./api").then((m) => m.loadSettings());

  const mirrorCb = document.querySelector<HTMLInputElement>("#cfg-mirror")!;
  mirrorCb.checked = settings.mirror_enabled;

  const moreBtn = document.querySelector<HTMLButtonElement>("#cfg-more-versions")!;
  moreBtn.addEventListener("click", appendVersions);

  await appendVersions();

  const select = document.querySelector<HTMLSelectElement>("#cfg-version")!;
  select.value = settings.version;

  // Save button
  const saveBtn = document.querySelector<HTMLButtonElement>("#cfg-runtime-save")!;
  const msg = document.querySelector<HTMLSpanElement>("#cfg-runtime-msg")!;
  saveBtn.addEventListener("click", async () => {
    const next = await import("./api").then((m) => m.loadSettings());
    next.mirror_enabled = mirrorCb.checked;
    next.version = select.value;
    await import("./api").then((m) => m.saveSettings(next));
    msg.textContent = "已保存";
    setTimeout(() => (msg.textContent = ""), 2000);
  });
}

async function main() {
  await initSettings();
  renderSettings(document.querySelector<HTMLElement>("#settings")!, refresh);
  await initRuntimeSettings();
  initLogPanel(document.querySelector<HTMLElement>("#log-body")!);
  await refresh();
  window.setInterval(() => {
    if (!busy) refresh();
  }, 5000);
}

main();
