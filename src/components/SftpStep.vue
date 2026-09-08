<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { NAlert, NButton, NDataTable, NForm, NFormItem, NInput, NSelect, useMessage } from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import StepShell from "./StepShell.vue";
import { sftpListFiles, uploadSftp } from "../api";
import { applyFirmware, currentSftpConfig, firmwareOptions, store } from "../store";
import type { SftpFileEntry } from "../types";

const emit = defineEmits<{ (e: "next"): void }>();
const message = useMessage();
const uploading = ref(false);
const remoteFilename = ref("");
const files = ref<SftpFileEntry[]>([]);

/** 文件列表列定义 */
const filesColumns: DataTableColumns<SftpFileEntry> = [
  { title: "文件名", key: "name", ellipsis: { tooltip: true } },
  { title: "大小", key: "size", width: 100, render: (row) => formatSize(row.size) },
  { title: "类型", key: "isDir", width: 80, render: (row) => (row.isDir ? "目录" : "文件") },
];

/** 从路径提取文件名 */
function baseName(p: string): string {
  return p.split(/[\\/]/).pop() ?? "";
}

const localName = computed(() => (store.localPath ? baseName(store.localPath) : ""));

/** 格式化文件大小 */
function formatSize(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / 1024 / 1024).toFixed(1)} MB`;
}

/** 选择本地 Bin 文件 */
async function pickFile() {
  const path = await open({
    multiple: false,
    filters: [{ name: "固件文件", extensions: ["bin", "BIN"] }],
  });
  if (typeof path === "string") {
    store.localPath = path;
    remoteFilename.value = baseName(path);
  }
}

/** 列出服务器文件 */
async function listRemoteFiles() {
  if (!store.config) return message.error("请先配置账号信息");
  if (!store.firmwareId) return message.error("请先选择固件 ID");
  try {
    const sftpCfg = currentSftpConfig();
    files.value = await sftpListFiles(sftpCfg, sftpCfg.remoteDir);
    message.success(`查到 ${files.value.length} 个条目`);
  } catch (e) {
    message.error(`读取失败：${e}`);
  }
}

/** 上传并计算 MD5 */
async function doUpload() {
  if (!store.config) return message.error("请先配置账号信息");
  if (!store.firmwareId) return message.error("请先选择固件 ID");
  if (!store.localPath) return message.error("请先选择本地文件");
  if (!remoteFilename.value) return message.error("请填写远程文件名");

  uploading.value = true;
  try {
    const md5 = await uploadSftp(store.localPath, remoteFilename.value, currentSftpConfig());
    store.remoteFilename = remoteFilename.value;
    store.md5 = md5;
    message.success("上传成功，MD5 已计算");
    emit("next");
  } catch (e) {
    message.error(`上传失败：${e}`);
  } finally {
    uploading.value = false;
  }
}

/**
 * 第一步选定固件批次，后续 CDN/Tange 都按这套走
 * @param {String} firmwareId - 固件 ID
 */
function onFirmwareChange(firmwareId: string) {
  applyFirmware(firmwareId);
}
</script>

<template>
  <StepShell title="SFTP 上传" desc="选择本地 .BIN 固件并上传，成功后自动进入 CDN 登记">
    <button type="button" class="file-drop" @click="pickFile">
      <span class="file-drop__icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="28" height="28" fill="none">
          <path
            d="M12 16V7m0 0l-3.5 3.5M12 7l3.5 3.5M6 18h12"
            stroke="currentColor"
            stroke-width="1.7"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </span>
      <span class="file-drop__title">{{ localName || "点击选择本地固件" }}</span>
      <span class="file-drop__hint">{{ store.localPath || "仅支持 .BIN 文件" }}</span>
    </button>

    <n-form label-placement="left" label-width="92" :show-feedback="false" class="form-stack">
      <n-form-item label="固件 ID">
        <n-select
          :value="store.firmwareId"
          :options="firmwareOptions"
          placeholder="选择要推送的固件批次"
          @update:value="onFirmwareChange"
        />
      </n-form-item>
      <n-form-item label="服务器目录">
        <n-input :value="store.serverDir" disabled />
      </n-form-item>
      <n-form-item label="远程文件名">
        <n-input v-model:value="remoteFilename" placeholder="上传到服务器的文件名" />
      </n-form-item>
    </n-form>

    <n-alert v-if="store.md5" title="MD5 已计算" type="success" :bordered="false">
      <code class="mono">{{ store.md5 }}</code>
    </n-alert>

    <div v-if="files.length">
      <h3 class="section__title">服务器文件</h3>
      <n-data-table
        :columns="filesColumns"
        :data="files"
        :bordered="true"
        :striped="true"
        size="small"
        :max-height="220"
      />
    </div>

    <template #footer>
      <n-button @click="listRemoteFiles">查看服务器文件</n-button>
      <n-button type="primary" :loading="uploading" @click="doUpload">上传并计算 MD5</n-button>
      <span class="spacer" />
      <n-button type="primary" ghost @click="emit('next')">下一步</n-button>
    </template>
  </StepShell>
</template>

<style scoped lang="scss">
.file-drop {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  min-height: 120px;
  padding: 20px;
  border: 1.5px dashed #94a3b8;
  border-radius: 6px;
  background: #f8fafc;
  cursor: pointer;
  color: inherit;
  font-family: inherit;
  appearance: none;
  transition: border-color 0.15s ease, background 0.15s ease;

  &:hover {
    border-color: #0e7490;
    background: #ecfeff;
  }

  &__icon {
    color: #0e7490;
  }

  &__title {
    font-size: 14px;
    font-weight: 600;
    color: #0f172a;
  }

  &__hint {
    font-size: 12px;
    color: #64748b;
    word-break: break-all;
    text-align: center;
    max-width: 100%;
  }
}

.spacer {
  flex: 1;
}
</style>
