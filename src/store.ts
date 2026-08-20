import { reactive } from "vue";
import type { AppConfig } from "./types";

/**
 * 全局共享状态：本地配置 + 三步流程的产物
 * 各步骤组件直接读写，避免 props/emit 层层传递
 */
export const store = reactive({
  config: null as AppConfig | null,
  current: 1, // 当前步骤（1-based）
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
