import { getLogs } from "./api";

let timer: number | undefined;

export function initLogPanel(container: HTMLElement, refreshBtn: HTMLElement) {
  container.innerHTML = `<pre id="log-text" class="log"></pre>`;
  const text = container.querySelector<HTMLPreElement>("#log-text")!;

  const refresh = async () => {
    text.textContent = await getLogs(300);
  };

  refreshBtn.addEventListener("click", refresh);
  refresh();

  timer = window.setInterval(refresh, 2000);
}

export function stopLogPolling() {
  if (timer) window.clearInterval(timer);
}
