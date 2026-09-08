<script setup lang="ts">
import { h, ref, watch } from "vue";
import { NButton, NDataTable, NForm, NFormItem, NInput, NTag, useMessage } from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import StepShell from "./StepShell.vue";
import { genataInsertVersion, genataListFiles, genataListVersions } from "../api";
import { buildCdnUrl, currentGenataConfig, currentProductId, store } from "../store";
import type { VersionItem } from "../types";

const emit = defineEmits<{ (e: "next"): void; (e: "prev"): void }>();
const message = useMessage();

const versions = ref<VersionItem[]>([]);
const files = ref<string[]>([]);
const version = ref(bumpVersion(store.latestVersion)); // 自动建议新版本号
const filename = ref(store.remoteFilename); // 自动带出文件名
const description = ref("");
const loading = ref(false);
const cdnUrl = ref("");
const selectedVersion = ref(""); // 选中的版本号

/** 版本列表表格列定义 */
const columns: DataTableColumns<VersionItem> = [
  { title: "id", key: "id", width: 64 },
  { title: "版本号", key: "version", width: 110 },
  { title: "文件名", key: "filename", ellipsis: { tooltip: true } },
  { title: "MD5", key: "md5", ellipsis: { tooltip: true } },
  {
    title: "操作",
    key: "action",
    width: 90,
    render: (row) =>
      h(
        NButton,
        {
          size: "small",
          type: selectedVersion.value === row.version ? "success" : "default",
          onClick: () => selectVersion(row),
        },
        { default: () => (selectedVersion.value === row.version ? "已选" : "选择") },
      ),
  },
];

/** 版本号递增：V1.0.0.4 -> V1.0.0.5 */
function bumpVersion(v: string): string {
  const m = v.match(/^(.*?)(\d+)$/);
  if (!m) return v;
  return `${m[1]}${parseInt(m[2], 10) + 1}`;
}

/** 从固件文件名解析版本号，如 ...-V1.0.0.5.BIN */
function parseVersionFromFilename(name: string): string {
  const m = name.match(/-V(\d+(?:\.\d+)*)\.BIN$/i);
  return m ? `V${m[1]}` : "";
}

/** 选择已有版本，直接进入 Tange 登记 */
function selectVersion(row: VersionItem) {
  selectedVersion.value = row.version;
  store.newVersion = row.version; // 版本号（带 V）
  store.remoteFilename = row.filename; // 文件名
  store.md5 = row.md5 ?? ""; // MD5
  store.cdnUrl = buildCdnUrl(row.filename); // CDN 地址
  store.description = row.description ?? "";
  filename.value = row.filename;
  cdnUrl.value = store.cdnUrl;
  message.success(`已选择版本：${row.version}`);
  emit("next");
}

/** 查询版本列表，自动记录最新 id，并带出上一步上传的文件名/版本 */
async function queryVersions(silent = false) {
  if (!store.config) return message.error("请先配置账号信息");
  loading.value = true;
  try {
    const list = await genataListVersions(currentGenataConfig());
    versions.value = list;
    if (list.length > 0) {
      const latest = list.reduce((current, item) => item.id > current.id ? item : current); // id 最大的最新版本
      store.latestPId = latest.id;
      store.latestVersion = latest.version;
    }
    filename.value = store.remoteFilename || filename.value;
    const fromFile = parseVersionFromFilename(filename.value);
    version.value = fromFile || (store.latestVersion ? bumpVersion(store.latestVersion) : version.value);
    if (!silent) message.success(`查询到 ${list.length} 个版本`);
  } catch (e) {
    message.error(`查询失败：${e}`);
  } finally {
    loading.value = false;
  }
}

/** 查询服务器已存在的文件列表 */
async function queryFiles() {
  if (!store.config) return message.error("请先配置账号信息");
  try {
    files.value = await genataListFiles(currentGenataConfig());
    message.success(`查询到 ${files.value.length} 个文件`);
  } catch (e) {
    message.error(`查询失败：${e}`);
  }
}

/** 登记新版本（后端完成 CDN 上传） */
async function doInsert() {
  if (!store.config) return message.error("请先配置账号信息");
  if (!version.value) return message.error("请填写版本号");
  if (!filename.value) return message.error("请填写文件名");

  loading.value = true;
  try {
    const id = await genataInsertVersion(
      currentGenataConfig(),
      version.value,
      filename.value,
      store.latestPId,
      description.value,
    );
    cdnUrl.value = buildCdnUrl(filename.value);
    store.cdnUrl = cdnUrl.value;
    store.newVersion = version.value;
    store.remoteFilename = filename.value;
    store.description = description.value;
    message.success(`登记成功，新版本 id=${id}`);
    emit("next");
  } catch (e) {
    message.error(`登记失败：${e}`);
  } finally {
    loading.value = false;
  }
}

function rowClassName(row: VersionItem): string {
  return selectedVersion.value === row.version ? "row-selected" : "";
}

/** 进入第二步时自动查版本并带出上一步文件名 */
watch(
  () => store.current,
  (val) => {
    if (val === 2) queryVersions(true);
  },
);
</script>

<template>
  <StepShell title="GenataTech 登记" desc="进入后自动查询版本并带出文件名；登记成功后自动进入 Tange">
    <template #extra>
      <n-tag :bordered="false">product_id：{{ currentProductId() || "未选择" }}　p_id：{{ store.latestPId || "未查询" }}</n-tag>
    </template>

    <div class="toolbar">
      <n-button type="primary" :loading="loading" @click="queryVersions(false)">查询版本列表</n-button>
      <n-button @click="queryFiles">查询已存在文件</n-button>
    </div>

    <n-data-table
      v-if="versions.length"
      :columns="columns"
      :data="versions"
      :bordered="true"
      :striped="true"
      size="small"
      :max-height="200"
      :row-class-name="rowClassName"
    />

    <n-form
      v-if="files.length"
      label-placement="left"
      label-width="80"
      :show-feedback="false"
      class="file-block"
    >
      <n-form-item label="服务器文件">
        <div class="file-chips">
          <n-tag v-for="f in files" :key="f" size="small">{{ f }}</n-tag>
        </div>
      </n-form-item>
    </n-form>

    <n-form label-placement="left" label-width="80" :show-feedback="false" class="insert-form">
      <n-form-item label="版本号">
        <n-input v-model:value="version" placeholder="如 V1.0.0.5" />
      </n-form-item>
      <n-form-item label="文件名">
        <n-input v-model:value="filename" placeholder="如 GENATA_..._V1.0.0.5.BIN" />
      </n-form-item>
      <n-form-item label="描述">
        <n-input v-model:value="description" placeholder="版本描述" />
      </n-form-item>
    </n-form>

    <n-form
      v-if="cdnUrl"
      label-placement="left"
      label-width="80"
      :show-feedback="false"
      class="cdn-block"
    >
      <n-form-item label="CDN 地址">
        <code class="mono">{{ cdnUrl }}</code>
      </n-form-item>
    </n-form>

    <template #footer>
      <n-button type="primary" :loading="loading" @click="doInsert">登记版本</n-button>
      <span class="spacer" />
      <n-button @click="emit('prev')">上一步</n-button>
      <n-button type="primary" ghost @click="emit('next')">下一步</n-button>
    </template>
  </StepShell>
</template>

<style scoped lang="scss">
.toolbar {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.file-block {
  flex-shrink: 0;

  :deep(.n-form-item) {
    margin-bottom: 0;
    align-items: flex-start;
  }

  :deep(.n-form-item-label) {
    padding-top: 10px;
  }
}

.file-chips {
  display: flex;
  flex-wrap: wrap;
  align-content: flex-start;
  align-items: center;
  gap: 6px;
  width: 100%;
  min-height: 36px;
  max-height: 120px;
  overflow: auto;
  padding: 8px 10px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
}

.insert-form {
  display: grid;
  grid-template-columns: minmax(200px, 1fr) minmax(280px, 1.6fr);
  gap: 14px 20px;
  flex-shrink: 0;

  :deep(.n-form-item) {
    margin-bottom: 0;
  }

  :deep(.n-form-item:last-child) {
    grid-column: 1 / -1;
  }
}

.cdn-block {
  flex-shrink: 0;
  padding: 8px 10px;
  background: #ecfdf5;
  border-radius: 4px;

  :deep(.n-form-item) {
    margin-bottom: 0;
  }

  :deep(.n-form-item-label) {
    color: #047857;
  }

  code {
    color: #047857;
    word-break: break-all;
  }
}

.spacer {
  flex: 1;
}
</style>
