import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  GenataConfig,
  SftpConfig,
  SftpFileEntry,
  TangeConfig,
  TangePublishParams,
  TangeVersionListResponse,
  VersionItem,
} from "./types";

/** 加载本地配置 */
export function loadConfig() {
  return invoke<AppConfig>("load_config");
}

/** 保存本地配置 */
export function saveConfig(config: AppConfig) {
  return invoke<void>("save_config", { config });
}

/** SFTP 上传并返回 MD5 */
export function uploadSftp(localPath: string, remoteFilename: string, sftpCfg: SftpConfig) {
  return invoke<string>("upload_sftp", { localPath, remoteFilename, sftpCfg });
}

/** 列出 SFTP 远程目录文件 */
export function sftpListFiles(sftpCfg: SftpConfig, remoteDir: string) {
  return invoke<SftpFileEntry[]>("sftp_list_files", { sftpCfg, remoteDir });
}

/** GenataTech 登录 */
export function genataLogin(genataCfg: GenataConfig) {
  return invoke<string>("genata_login", { genataCfg });
}

/** GenataTech 查询已存在固件文件列表 */
export function genataListFiles(genataCfg: GenataConfig) {
  return invoke<string[]>("genata_list_files", { genataCfg });
}

/** GenataTech 查询版本列表 */
export function genataListVersions(genataCfg: GenataConfig) {
  return invoke<VersionItem[]>("genata_list_versions", { genataCfg });
}

/** GenataTech 登记新版本 */
export function genataInsertVersion(
  genataCfg: GenataConfig,
  version: string,
  filename: string,
  pId: number,
  description: string,
) {
  return invoke<number>("genata_insert_version", { genataCfg, version, filename, pId, description });
}

/** Tange 查询固件版本列表 */
export function tangeListVersions(tangeCfg: TangeConfig) {
  return invoke<TangeVersionListResponse>("tange_list_versions", { tangeCfg });
}

/** Tange 推送升级规则 */
export function tangeAddRule(
  tangeCfg: TangeConfig,
  versionNumber: string,
  reason: string,
  deviceIds: string[],
) {
  return invoke<Record<string, unknown>>("tange_add_rule", { tangeCfg, versionNumber, reason, deviceIds });
}

/** Tange 取消设备升级规则 */
export function tangeRemoveRule(tangeCfg: TangeConfig, reason: string, deviceIds: string[]) {
  return invoke<Record<string, unknown>>("tange_remove_rule", { tangeCfg, reason, deviceIds });
}

/** Tange 删除固件版本 */
export function tangeDeleteVersion(tangeCfg: TangeConfig, versionNumber: string) {
  return invoke<Record<string, unknown>>("tange_delete_version", { tangeCfg, versionNumber });
}

/** Tange 登记固件版本（需 HMAC 签名，暂未实现） */
export function tangePublishVersion(tangeCfg: TangeConfig, params: TangePublishParams) {
  return invoke<Record<string, unknown>>("tange_publish_version", { tangeCfg, ...params });
}
