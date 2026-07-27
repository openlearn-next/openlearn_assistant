import { invoke } from "@tauri-apps/api/core";

export interface NodeInfo {
  installed: boolean;
  version: string;
  path: string;
  meets_requirement: boolean;
}

export interface Status {
  running: boolean;
  pid: number | null;
  port: number;
  version: string | null;
  node_ok: boolean;
}

export interface Settings {
  port: number;
  db_path: string;
  gemini_api_key: string;
}

export const detectNode = () => invoke<NodeInfo>("detect_node");
export const provisionNode = () => invoke<void>("provision_node");
export const installPkg = () => invoke<void>("install_pkg");
export const uninstallPkg = (keepData: boolean) =>
  invoke<void>("uninstall_pkg", { keep_data: keepData });
export const upgradePkg = () => invoke<void>("upgrade_pkg");
export const startService = () => invoke<void>("start_service");
export const stopService = () => invoke<void>("stop_service");
export const getStatus = () => invoke<Status>("status");
export const getLogs = (tail: number) => invoke<string>("get_logs", { tail });
export const loadSettings = () => invoke<Settings>("load_settings");
export const saveSettings = (s: Settings) =>
  invoke<void>("save_settings", { settings: s });
