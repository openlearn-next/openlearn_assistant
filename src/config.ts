import { loadSettings, saveSettings, type Settings } from "./api";

let current: Settings = { port: 9000, db_path: "", gemini_api_key: "" };

export function renderSettings(container: HTMLElement) {
  container.innerHTML = `
    <h2>设置</h2>
    <label class="field">
      <span>端口</span>
      <input id="cfg-port" type="number" min="1" max="65535" />
    </label>
    <label class="field">
      <span>数据库路径</span>
      <input id="cfg-db" type="text" placeholder="~/openlearn-next/data.db" />
    </label>
    <label class="field">
      <span>GEMINI_API_KEY（可选）</span>
      <input id="cfg-key" type="password" placeholder="留空使用后台配置" />
    </label>
    <button id="cfg-save" class="btn">保存设置</button>
    <span id="cfg-msg" class="hint"></span>
  `;

  const port = container.querySelector<HTMLInputElement>("#cfg-port")!;
  const db = container.querySelector<HTMLInputElement>("#cfg-db")!;
  const key = container.querySelector<HTMLInputElement>("#cfg-key")!;
  const msg = container.querySelector<HTMLSpanElement>("#cfg-msg")!;

  port.value = String(current.port);
  db.value = current.db_path;
  key.value = current.gemini_api_key;

  container.querySelector<HTMLButtonElement>("#cfg-save")!.addEventListener(
    "click",
    async () => {
      const next: Settings = {
        port: parseInt(port.value, 10) || 9000,
        db_path: db.value.trim(),
        gemini_api_key: key.value,
      };
      await saveSettings(next);
      current = next;
      msg.textContent = "已保存";
      setTimeout(() => (msg.textContent = ""), 2000);
    }
  );
}

export async function initSettings() {
  current = await loadSettings();
  return current;
}

export function getSettings(): Settings {
  return current;
}
