import { loadSettings, saveSettings, type Settings } from "./api";

let current: Settings = { port: 9000, db_path: "", gemini_api_key: "" };

export function renderSettings(container: HTMLElement, onSaved?: () => void) {
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
    <button id="cfg-save" class="btn">保存设置</button>
    <span id="cfg-msg" class="hint"></span>
  `;

  const port = container.querySelector<HTMLInputElement>("#cfg-port")!;
  const db = container.querySelector<HTMLInputElement>("#cfg-db")!;
  const msg = container.querySelector<HTMLSpanElement>("#cfg-msg")!;

  port.value = String(current.port);
  db.value = current.db_path;

  container.querySelector<HTMLButtonElement>("#cfg-save")!.addEventListener(
    "click",
    async () => {
      const next: Settings = {
        port: parseInt(port.value, 10) || 9000,
        db_path: db.value.trim(),
      };
      await saveSettings(next);
      current = next;
      msg.textContent = "已保存";
      onSaved?.();
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
