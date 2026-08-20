<script setup lang="ts">
import { computed, h, ref, watch } from "vue";
import {
  NButton,
  NCollapse,
  NCollapseItem,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NRadio,
  NRadioGroup,
  NSpace,
  NTag,
  useMessage,
} from "naive-ui";
import type { DataTableColumns } from "naive-ui";
import StepShell from "./StepShell.vue";
import { tangeDeleteVersion, tangeListVersions, tangePublishVersion, tangeRemoveRule } from "../api";
import { store } from "../store";
import type { TangeVersionItem } from "../types";

const emit = defineEmits<{ (e: "next"): void; (e: "prev"): void }>();
const message = useMessage();

const content = ref(""); // 版本说明
const importance = ref(2); // 重要性
const versionType = ref("testing"); // 版本类型
const versions = ref<TangeVersionItem[]>([]);
const loading = ref(false);

const removeReason = ref(""); // 取消理由
const removeDeviceIds = ref(""); // 设备 ID，逗号/换行分隔
const deleteTarget = ref<TangeVersionItem | null>(null); // 待删除的版本
const showDeleteConfirm = ref(false); // 删除确认弹窗显隐

/** 不带 V 前缀的版本号（Tange 用 1.0.0.5） */
const versionNumber = computed(() => store.newVersion.replace(/^V/i, ""));

/** Tange 已有版本表格列定义 */
const columns: DataTableColumns<TangeVersionItem> = [
  { title: "版本号", key: "version_number", width: 110 },
  { title: "类型", key: "version_type", width: 88 },
  { title: "MD5", key: "md5", ellipsis: { tooltip: true } },
  { title: "说明", key: "content", ellipsis: { tooltip: true } },
  { title: "重要性", key: "importance", width: 70 },
  {
    title: "操作",
    key: "action",
    width: 148,
    render: (row) =>
      h(NSpace, { size: "small" }, {
        default: () => [
          h(
            NButton,
            { size: "small", type: "primary", onClick: () => pushVersion(row) },
            { default: () => "推送" },
          ),
          h(
            NButton,
            { size: "small", type: "error", ghost: true, onClick: () => removeVersion(row) },
            { default: () => "删除" },
          ),
        ],
      }),
  },
];

/** 登记固件版本到 Tange（demo 环境，无需 HMAC 签名） */
async function doPublish() {
  if (!store.config) return message.error("请先配置账号信息");
  if (!store.cdnUrl) return message.error("请先在第二步完成登记");
  try {
    await tangePublishVersion(store.config.tange, {
      firmwareId: store.config.tange.firmwareId,
      versionNumber: versionNumber.value,
      versionType: versionType.value,
      downloadLink: store.cdnUrl,
      md5: store.md5,
      content: content.value,
      importance: importance.value,
    });
    message.success("登记成功");
    emit("next");
  } catch (e) {
    message.error(`${e}`);
  }
}

/** 查询 Tange 已有版本列表 */
async function doQuery(silent = false) {
  if (!store.config) return message.error("请先配置账号信息");
  loading.value = true;
  try {
    const res = await tangeListVersions(store.config.tange);
    versions.value = res.data.list ?? [];
    if (!silent) message.success(`查询到 ${versions.value.length} 个版本`);
  } catch (e) {
    message.error(`查询失败：${e}`);
  } finally {
    loading.value = false;
  }
}

/** 推送选中版本，携带版本号进入第四步 */
function pushVersion(row: TangeVersionItem) {
  store.newVersion = row.version_number;
  message.success(`准备推送版本 ${row.version_number}`);
  emit("next");
}

/** 弹出删除确认 */
function removeVersion(row: TangeVersionItem) {
  deleteTarget.value = row;
  showDeleteConfirm.value = true;
}

/** 确认删除版本 */
async function confirmDelete() {
  if (!deleteTarget.value || !store.config) return;
  try {
    await tangeDeleteVersion(store.config.tange, deleteTarget.value.version_number);
    message.success("删除成功");
    await doQuery(true); // 删除后刷新列表
  } catch (e) {
    message.error(`删除失败：${e}`);
  }
}

/** 解析取消推送的设备 ID 列表 */
function parseRemoveDeviceIds(): string[] {
  return removeDeviceIds.value
    .split(/[,\n，]/)
    .map((s) => s.trim())
    .filter(Boolean);
}

/** 取消设备升级规则 */
async function doRemove() {
  if (!store.config) return message.error("请先配置账号信息");
  const ids = parseRemoveDeviceIds();
  if (ids.length === 0) return message.error("请填写设备 ID");
  if (!removeReason.value) return message.error("请填写取消理由");
  try {
    await tangeRemoveRule(store.config.tange, removeReason.value, ids);
    message.success("取消推送成功");
  } catch (e) {
    message.error(`取消推送失败：${e}`);
  }
}

/** 进入第三步时自动拉列表，并带出上一步描述 */
watch(
  () => store.current,
  (val) => {
    if (val !== 3) return;
    if (!content.value && store.description) content.value = store.description;
    doQuery(true);
  },
);
</script>

<template>
  <StepShell title="上传 Tange" desc="登记成功后自动进入推送；也可从已有版本直接推送">
    <template #extra>
      <n-tag :bordered="false" type="info">{{ versionNumber || "未生成版本号" }}</n-tag>
    </template>

    <n-form label-placement="left" label-width="80" :show-feedback="false" class="pub-form">
      <n-form-item label="版本类型">
        <n-radio-group v-model:value="versionType">
          <n-radio value="testing">testing</n-radio>
          <n-radio value="release">release</n-radio>
        </n-radio-group>
      </n-form-item>
      <n-form-item label="重要性">
        <n-input-number v-model:value="importance" :min="1" :max="3" style="width: 120px" />
      </n-form-item>
      <n-form-item label="版本说明">
        <n-input
          v-model:value="content"
          type="textarea"
          placeholder="版本说明"
          :autosize="{ minRows: 3, maxRows: 6 }"
        />
      </n-form-item>
    </n-form>

    <n-data-table
      v-if="versions.length"
      :columns="columns"
      :data="versions"
      :bordered="true"
      :striped="true"
      size="small"
      :max-height="220"
    />

    <n-collapse>
      <n-collapse-item title="取消设备升级推送" name="remove">
        <n-form label-placement="left" label-width="80" :show-feedback="false" class="form-stack">
          <n-form-item label="固件 ID">
            <n-tag type="info" :bordered="false">{{ store.config?.tange.firmwareId }}</n-tag>
          </n-form-item>
          <n-form-item label="取消理由">
            <n-input v-model:value="removeReason" placeholder="如：测试取消" />
          </n-form-item>
          <n-form-item label="设备 ID">
            <n-input
              v-model:value="removeDeviceIds"
              type="textarea"
              placeholder="多个设备 ID 用逗号或换行分隔"
              :rows="2"
            />
          </n-form-item>
          <n-form-item>
            <n-button type="warning" @click="doRemove">确认取消</n-button>
          </n-form-item>
        </n-form>
      </n-collapse-item>
    </n-collapse>

    <n-modal
      v-model:show="showDeleteConfirm"
      preset="dialog"
      title="确认删除"
      positive-text="确认删除"
      negative-text="取消"
      @positive-click="confirmDelete"
    >
      确认删除版本 {{ deleteTarget?.version_number }}？
    </n-modal>

    <template #footer>
      <n-button type="primary" @click="doPublish">登记版本</n-button>
      <span class="spacer" />
      <n-button @click="emit('prev')">上一步</n-button>
      <n-button type="primary" ghost @click="emit('next')">下一步</n-button>
    </template>
  </StepShell>
</template>

<style scoped lang="scss">
.pub-form {
  display: grid;
  grid-template-columns: 1fr 180px;
  gap: 14px 20px;

  :deep(.n-form-item) {
    margin-bottom: 0;
  }

  :deep(.n-form-item:last-child) {
    grid-column: 1 / -1;
  }
}

.spacer {
  flex: 1;
}
</style>
