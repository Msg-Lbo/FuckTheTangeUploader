<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check as checkForUpdate } from "@tauri-apps/plugin-updater";
import { NButton, NForm, NFormItem, NInput, NInputNumber, NProgress, NSelect, NTabPane, NTabs, useMessage } from "naive-ui";
import { saveConfig } from "../api";
import { applyFirmware, findFirmware, firmwareOptions, store } from "../store";

const emit = defineEmits<{ (e: "saved"): void }>();
const message = useMessage();
const appVersion = ref(""); // 当前应用版本
const availableVersion = ref(""); // 可更新版本
const updateMessage = ref(""); // 更新提示
const updateError = ref(""); // 更新错误
const checkingUpdate = ref(false); // 检查更新状态
const installingUpdate = ref(false); // 安装更新状态
const updateProgress = ref(0); // 更新下载进度
let pendingUpdate: Awaited<ReturnType<typeof checkForUpdate>> = null; // 待安装更新
const updateTimeout = 120000; // 更新请求超时时间

const updateButtonLabel = computed(() => {
  if (installingUpdate.value) return updateProgress.value > 0 ? `正在下载 ${updateProgress.value}%` : "正在准备更新";
  if (checkingUpdate.value) return "正在检查";
  if (availableVersion.value) return `更新到 v${availableVersion.value}`;
  return "检查更新";
});

/** 表单模型：从 store 复制一份编辑，保存时写回 */
const form = reactive({
  sftp: {
    host: store.config?.sftp.host ?? "www.genatatech.com",
    port: store.config?.sftp.port ?? 22,
    username: store.config?.sftp.username ?? "",
    password: store.config?.sftp.password ?? "",
    remoteDir: store.config?.sftp.remoteDir ?? "",
  },
  genata: {
    username: store.config?.genata.username ?? "",
    password: store.config?.genata.password ?? "",
    baseUrl: store.config?.genata.baseUrl ?? "https://www.genatatech.com",
    productId: store.config?.genata.productId ?? "188",
  },
  tange: {
    appId: store.config?.tange.appId ?? "",
    accessKey: store.config?.tange.accessKey ?? "",
    accessSecret: store.config?.tange.accessSecret ?? "",
    firmwareId: store.config?.tange.firmwareId ?? "GENATA_JZ-T32LQ_IPC_Z300S-4G",
  },
});

/** 配置异步加载后同步到表单 */
watch(
  () => store.config,
  (cfg) => {
    if (!cfg) return;
    Object.assign(form.sftp, cfg.sftp);
    Object.assign(form.genata, cfg.genata);
    Object.assign(form.tange, cfg.tange);
  },
  { immediate: true },
);

/**
 * 读取当前应用版本
 */
async function loadAppVersion() {
  try {
    appVersion.value = await getVersion();
  } catch (error) {
    updateError.value = `无法读取应用版本：${String(error)}`;
  }
}

/**
 * 检查 GitHub Release 更新
 */
async function handleCheckUpdate() {
  if (checkingUpdate.value || installingUpdate.value) return;
  checkingUpdate.value = true;
  updateMessage.value = "";
  updateError.value = "";

  try {
    pendingUpdate = await checkForUpdate({ timeout: updateTimeout });
    if (!pendingUpdate) {
      availableVersion.value = "";
      updateMessage.value = "当前已是最新版本";
      return;
    }
    availableVersion.value = pendingUpdate.version;
    updateMessage.value = `发现新版本 v${pendingUpdate.version}`;
  } catch (error) {
    updateError.value = `检查更新失败：${String(error)}`;
  } finally {
    checkingUpdate.value = false;
  }
}

/**
 * 下载并安装已发现的签名更新
 */
async function handleInstallUpdate() {
  if (!pendingUpdate || installingUpdate.value) return;
  installingUpdate.value = true;
  updateProgress.value = 0;
  updateMessage.value = `正在下载 v${pendingUpdate.version}`;
  updateError.value = "";
  let downloaded = 0; // 已下载字节数
  let contentLength = 0; // 更新包总字节数

  try {
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === "Started") {
        contentLength = event.data.contentLength ?? 0;
      } else if (event.event === "Progress") {
        downloaded += event.data.chunkLength;
        if (contentLength > 0) updateProgress.value = Math.min(99, Math.round(downloaded / contentLength * 100));
      } else if (event.event === "Finished") {
        updateProgress.value = 100;
      }
    }, { timeout: updateTimeout });
    updateMessage.value = "更新已安装，正在重启";
    await relaunch();
  } catch (error) {
    updateError.value = `安装更新失败：${String(error)}`;
    installingUpdate.value = false;
  }
}

/**
 * 根据更新状态执行检查或安装
 */
async function handleUpdateAction() {
  if (pendingUpdate) {
    await handleInstallUpdate();
    return;
  }
  await handleCheckUpdate();
}

/** 配置里改固件 ID 时同步产品 ID 和服务器目录 */
function onFirmwareChange(firmwareId: string) {
  applyFirmware(firmwareId);
  const profile = findFirmware(firmwareId);
  if (profile) form.genata.productId = profile.productId;
}

/** 保存配置 */
async function onSave() {
  const config = { sftp: form.sftp, genata: form.genata, tange: form.tange };
  try {
    await saveConfig(config);
    store.config = config;
    applyFirmware(form.tange.firmwareId);
    message.success("配置已保存");
    emit("saved");
  } catch (e) {
    message.error(`保存失败：${e}`);
  }
}

onMounted(() => void loadAppVersion());
</script>

<template>
  <div class="config">
    <n-tabs type="line" animated>
      <n-tab-pane name="sftp" tab="SFTP 上传">
        <n-form label-placement="left" label-width="88" :show-feedback="false" class="form-stack">
          <n-form-item label="服务器">
            <n-input v-model:value="form.sftp.host" />
          </n-form-item>
          <n-form-item label="端口">
            <n-input-number v-model:value="form.sftp.port" :min="1" :max="65535" class="full" />
          </n-form-item>
          <n-form-item label="用户名">
            <n-input v-model:value="form.sftp.username" />
          </n-form-item>
          <n-form-item label="密码">
            <n-input v-model:value="form.sftp.password" type="password" show-password-on="click" />
          </n-form-item>
          <n-form-item label="远程目录">
            <n-input v-model:value="form.sftp.remoteDir" placeholder="留空则上传到根目录" />
          </n-form-item>
        </n-form>
      </n-tab-pane>

      <n-tab-pane name="genata" tab="GenataTech">
        <n-form label-placement="left" label-width="88" :show-feedback="false" class="form-stack">
          <n-form-item label="登录账号">
            <n-input v-model:value="form.genata.username" />
          </n-form-item>
          <n-form-item label="登录密码">
            <n-input v-model:value="form.genata.password" type="password" show-password-on="click" />
          </n-form-item>
          <n-form-item label="接口地址">
            <n-input v-model:value="form.genata.baseUrl" />
          </n-form-item>
          <n-form-item label="产品 ID">
            <n-input v-model:value="form.genata.productId" />
          </n-form-item>
        </n-form>
      </n-tab-pane>

      <n-tab-pane name="tange" tab="Tange">
        <n-form label-placement="left" label-width="110" :show-feedback="false" class="form-stack">
          <n-form-item label="App ID">
            <n-input v-model:value="form.tange.appId" />
          </n-form-item>
          <n-form-item label="Access Key">
            <n-input v-model:value="form.tange.accessKey" />
          </n-form-item>
          <n-form-item label="Access Secret">
            <n-input v-model:value="form.tange.accessSecret" type="password" show-password-on="click" />
          </n-form-item>
          <n-form-item label="固件 ID">
            <n-select
              v-model:value="form.tange.firmwareId"
              :options="firmwareOptions"
              placeholder="选择固件批次"
              @update:value="onFirmwareChange"
            />
          </n-form-item>
        </n-form>
      </n-tab-pane>
    </n-tabs>

    <section class="update-panel">
      <div class="update-panel__row">
        <div class="update-panel__version">
          <strong>应用版本</strong>
          <span>v{{ appVersion || "--" }}</span>
        </div>
        <n-button
          secondary
          type="info"
          :loading="checkingUpdate"
          :disabled="checkingUpdate || installingUpdate"
          @click="handleUpdateAction"
        >
          {{ updateButtonLabel }}
        </n-button>
      </div>
      <n-progress
        v-if="installingUpdate"
        type="line"
        :percentage="updateProgress"
        :processing="updateProgress < 100"
        indicator-placement="inside"
      />
      <p v-if="updateError" class="update-panel__message is-error">{{ updateError }}</p>
      <p v-else-if="updateMessage" class="update-panel__message is-success">{{ updateMessage }}</p>
    </section>

    <n-button type="primary" block @click="onSave">保存配置</n-button>
  </div>
</template>

<style scoped lang="scss">
.config {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 100%;

  .full {
    width: 100%;
  }

  .update-panel {
    padding-top: 16px;
    border-top: 1px solid #e2e8f0;

    &__row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
    }

    &__version {
      display: flex;
      flex-direction: column;
      gap: 3px;

      strong {
        font-size: 13px;
        color: #334155;
      }

      span {
        font-size: 12px;
        color: #64748b;
        font-variant-numeric: tabular-nums;
      }
    }

    &__message {
      margin: 8px 0 0;
      font-size: 12px;

      &.is-success {
        color: #047857;
      }

      &.is-error {
        color: #dc2626;
      }
    }

    .n-progress {
      margin-top: 10px;
    }
  }
}
</style>
