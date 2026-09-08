import { reactive } from "vue";
import type { AppConfig, FirmwareProfile, SftpConfig } from "./types";

/** 可选固件批次 */
export const FIRMWARE_PROFILES: FirmwareProfile[] = [
  {
    firmwareId: "GENATA_JZ-T32LQ_IPC_Z300S-4G",
    label: "GENATA_JZ-T32LQ_IPC_Z300S-4G",
    serverDir: "GT-Z300S-4G",
    productId: "188",
  },
  {
    firmwareId: "GENATA_JZ-T33VL_IPC_ZH300S-V4",
    label: "GENATA_JZ-T33VL_IPC_ZH300S-V4",
    serverDir: "GT-ZH300S-V4",
    productId: "189",
  },
];

/** 固件下拉选项 */
export const firmwareOptions = FIRMWARE_PROFILES.map((item) => ({
  label: item.label,
  value: item.firmwareId,
}));

/**
 * 全局共享状态：本地配置 + 三步流程的产物
 * 各步骤组件直接读写，避免 props/emit 层层传递
 */
export const store = reactive({
  config: null as AppConfig | null,
  current: 1, // 当前步骤（1-based）
  firmwareId: FIRMWARE_PROFILES[0].firmwareId, // 当前固件 ID
  serverDir: FIRMWARE_PROFILES[0].serverDir, // CDN/SFTP 目录
  /** 步骤1 产物 */
  localPath: "",
  remoteFilename: "",
  md5: "",
  /** 步骤2 产物 */
  latestPId: 0,
  latestVersion: "",
  newVersion: "",
  cdnUrl: "",
  description: "",
});

/**
 * 按固件 ID 查找批次
 * @param {String} firmwareId - 固件 ID
 */
export function findFirmware(firmwareId: string): FirmwareProfile | undefined {
  return FIRMWARE_PROFILES.find((item) => item.firmwareId === firmwareId);
}

/**
 * 拼接 SFTP 远程目录：账号配置目录 + 当前固件服务器目录
 * @param {String} base - 配置里的远程目录
 * @param {String} dir - 固件服务器目录
 */
export function joinRemote(base: string, dir: string): string {
  const prefix = base.replace(/[/\\]+$/, "");
  const name = dir.replace(/^[/\\]+|[/\\]+$/g, "");
  if (!prefix) return name;
  if (!name) return prefix.replace(/\\/g, "/");
  const normalized = prefix.replace(/\\/g, "/");
  if (normalized === name || normalized.endsWith(`/${name}`)) return normalized;
  return `${normalized}/${name}`;
}

/**
 * 拼 CDN 下载地址
 * @param {String} filename - 固件文件名
 */
export function buildCdnUrl(filename: string): string {
  return `http://fw.genatatech.com/static/firmware/${store.serverDir}/${filename}`;
}

/**
 * 当前上传/列目录用的 SFTP 配置（带上固件服务器目录）
 */
export function currentSftpConfig(): SftpConfig {
  const sftp = store.config!.sftp;
  return {
    ...sftp,
    remoteDir: joinRemote(sftp.remoteDir, store.serverDir),
  };
}

/**
 * 当前 Genata 配置（productId 用所选固件批次，不沿用配置里的旧值）
 */
export function currentGenataConfig() {
  const genata = store.config!.genata;
  const profile = findFirmware(store.firmwareId);
  return {
    ...genata,
    productId: profile!.productId,
  };
}

/**
 * 第一步选中固件后，后续步骤全部按这套 ID/目录/产品走
 * @param {String} firmwareId - 固件 ID
 */
export function applyFirmware(firmwareId: string) {
  const profile = findFirmware(firmwareId);
  if (!profile) return;
  store.firmwareId = profile.firmwareId;
  store.serverDir = profile.serverDir;
  if (!store.config) return;
  store.config.tange.firmwareId = profile.firmwareId;
  store.config.genata.productId = profile.productId;
}
