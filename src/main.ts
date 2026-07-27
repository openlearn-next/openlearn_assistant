import "./styles.css";
import {
  detectNode,
  getStatus,
  provisionNode,
  installPkg,
  uninstallPkg,
  upgradePkg,
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
    <span class="sub">安装 · 卸载 · 升级 · 启动 · 停止</span>
  </header>

  <section class="card" id="status-card">
    <div class="row"><span class="k">Node.js</span><span id="st-node" class="v">检测中…</span></div>
    <div class="row"><span class="k">openlearn-next</span><span id="st-pkg" class="v">检测中…</span></div>
    <div class="row"><span class="k">运行状态</span><span id="st-run" class="v">检测中…</span></div>
    <div class="row"><span class="k">访问地址</span><span id="st-url" class="v">—</span></div>
  </section>

  <section class="card actions">
    <button id="btn-node" class="btn">安装 Node 22</button>
    <button id="btn-install" class="btn primary">安装</button>
    <button id="btn-uninstall" class="btn">卸载</button>
    <button id="btn-upgrade" class="btn">升级</button>
    <button id="btn-start" class="btn primary">启动</button>
    <button id="btn-stop" class="btn danger">停止</button>
    <a id="btn-open" class="btn ghost" target="_blank" rel="noreferrer">打开</a>
  </section>

  <section class="card" id="settings"></section>

  <section class="card">
    <div class="log-head"><h2>日志</h2><button id="log-refresh" class="btn small">刷新</button></div>
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
  btnInstall: document.querySelector<HTMLButtonElement>("#btn-install")!,
  btnUninstall: document.querySelector<HTMLButtonElement>("#btn-uninstall")!,
  btnUpgrade: document.querySelector<HTMLButtonElement>("#btn-upgrade")!,
  btnStart: document.querySelector<HTMLButtonElement>("#btn-start")!,
  btnStop: document.querySelector<HTMLButtonElement>("#btn-stop")!,
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
  [
    el.btnNode,
    el.btnInstall,
    el.btnUninstall,
    el.btnUpgrade,
    el.btnStart,
    el.btnStop,
  ].forEach((b) => (b.disabled = d));
}

function updateUI(node: NodeInfo, st: Status) {
  const installed = !!st.version;

  el.node.textContent = node.installed
    ? `${node.version}${node.meets_requirement ? " ✓" : " ✗ (<22)"}`
    : "未安装";
  el.node.className = `v ${node.meets_requirement ? "ok" : "bad"}`;

  el.pkg.textContent = installed ? `已安装 ${st.version}` : "未安装";
  el.pkg.className = `v ${installed ? "ok" : ""}`;

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

  // Button visibility / enabled state
  el.btnNode.style.display = node.meets_requirement ? "none" : "";
  el.btnInstall.disabled = !node.meets_requirement || installed;
  el.btnUninstall.disabled = !installed;
  el.btnUpgrade.disabled = !installed;
  el.btnStart.disabled = !installed || st.running;
  el.btnStop.disabled = !st.running;
}

let nodeCache: NodeInfo | null = null;
let statusCache: Status | null = null;

async function refresh() {
  try {
    const [node, st] = await Promise.all([detectNode(), getStatus()]);
    nodeCache = node;
    statusCache = st;
    updateUI(node, st);
  } catch (e) {
    toast(`状态刷新失败：${(e as Error).message ?? e}`, "err");
  }
}

el.btnNode.addEventListener("click", () =>
  run("安装 Node 22", provisionNode)
);
el.btnInstall.addEventListener("click", () =>
  run("安装", installPkg)
);
el.btnUninstall.addEventListener("click", async () => {
  const keep = confirm("是否保留用户数据（数据库与上传文件）？\n确定 = 保留，取消 = 彻底删除");
  await run("卸载", () => uninstallPkg(keep));
});
el.btnUpgrade.addEventListener("click", () => run("升级", upgradePkg));
el.btnStart.addEventListener("click", () => run("启动", startService));
el.btnStop.addEventListener("click", () => run("停止", stopService));

async function main() {
  await initSettings();
  renderSettings(document.querySelector<HTMLElement>("#settings")!, refresh);
  initLogPanel(
    document.querySelector<HTMLElement>("#log-body")!,
    document.querySelector<HTMLElement>("#log-refresh")!
  );
  await refresh();
  // light polling so running/stopped reflects without manual refresh
  window.setInterval(() => {
    if (!busy) refresh();
  }, 5000);
}

main();
