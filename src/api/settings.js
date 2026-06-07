import { invoke } from "@tauri-apps/api/core";

/** 阅读器与全局设置 */

export async function getReaderSettings() {
  const json = await invoke("get_reader_settings");
  return typeof json === "string" ? JSON.parse(json || "{}") : json;
}

export async function saveReaderSettings(settings) {
  return invoke("save_reader_settings", {
    settingsJson: JSON.stringify(settings),
  });
}

/** 本地书库路径信息（运行时按当前用户/系统生成，非写死） */
export async function getLibraryPath() {
  return invoke("get_library_path");
}

/** 设置自定义书库文件夹，需重启应用 */
export async function setLibraryPath(path) {
  return invoke("set_library_path", { path });
}

/** 恢复默认书库位置，需重启应用 */
export async function resetLibraryPath() {
  return invoke("reset_library_path");
}
