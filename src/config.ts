import { loadSettings, saveSettings, type Settings } from "./api";

let current: Settings = {
  port: 9000,
  db_path: "",
  gemini_api_key: "",
  mirror_enabled: true,
  version: "latest",
};

export function renderSettings(container: HTMLElement, onSaved?: () => void) {
  container.innerHTML = `
    <div class="cfg-row">
      <label class="cfg-item">
        <span>端口</span>
        <input id="cfg-port" type="number" min="1" max="65535" />
      </label>
      <label class="cfg-item cfg-db">
        <span>数据库</span>
        <input id="cfg-db" type="text" placeholder="~/openlearn-next/data.db" />
      </label>
    </div>
    <div class="cfg-row">
      <label class="checkbox-row">
        <input id="cfg-mirror" type="checkbox" checked />
        <span>中国镜像</span>
      </label>
      <button id="cfg-save" class="btn small">保存</button>
      <span id="cfg-msg" class="hint"></span>
    </div>
  `;

  const port = container.querySelector<HTMLInputElement>("#cfg-port")!;
  const db = container.querySelector<HTMLInputElement>("#cfg-db")!;
  const msg = container.querySelector<HTMLSpanElement>("#cfg-msg")!;
  port.value = String(current.port);
  db.value = current.db_path;

  container.querySelector<HTMLButtonElement>("#cfg-save")!.addEventListener(
    "click",
    async () => {
      const mirrorCb = container.querySelector<HTMLInputElement>("#cfg-mirror")!;
      const select = document.querySelector<HTMLSelectElement>("#cfg-version")!;
      const next: Settings = {
        port: parseInt(port.value, 10) || 9000,
        db_path: db.value.trim(),
        gemini_api_key: "",
        mirror_enabled: mirrorCb.checked,
        version: select.value,
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
