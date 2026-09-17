<script setup lang="ts">
import { computed, ref } from "vue";
import { NButton, NCheckbox, NCheckboxGroup, NForm, NFormItem, NInput, NTag, useMessage } from "naive-ui";
import StepShell from "./StepShell.vue";
import { saveConfig, tangeAddRule } from "../api";
import { store } from "../store";
import type { PushRecord } from "../types";

const emit = defineEmits<{ (e: "prev"): void }>();
const message = useMessage();

const reason = ref(""); // 推送原因
const deviceIdsText = ref(""); // 设备 ID，逗号/换行分隔
const pushing = ref(false);

/** 不带 V 前缀的版本号 */
const versionNumber = computed(() => store.newVersion.replace(/^V/i, ""));

/** 当前固件的推送记录（最近一次在前） */
const firmwareHistory = computed(
  () => (store.config?.pushHistory ?? []).filter((item) => item.firmwareId === store.firmwareId),
);

/** 上次推送记录 */
const lastPush = computed(() => firmwareHistory.value[0] ?? null);

/** 历史设备去重（最近推送的排在前面） */
const historyDevices = computed(() => {
  const seen = new Set<string>(); // 已出现过的设备
  const list: string[] = [];
  for (const record of firmwareHistory.value) {
    for (const id of record.deviceIds) {
      if (seen.has(id)) continue;
      seen.add(id);
      list.push(id);
    }
  }
  return list;
});

/** 解析设备 ID 列表（逗号或换行分隔） */
function parseDeviceIds(): string[] {
  return deviceIdsText.value
    .split(/[,\n，]/)
    .map((s) => s.trim())
    .filter(Boolean);
}

/** 勾选设备与文本域双向同步 */
const selectedIds = computed({
  get: () => parseDeviceIds(),
  set: (ids: string[]) => {
    deviceIdsText.value = ids.join(", ");
  },
});

/** 可勾选设备：历史设备 + 手填设备 */
const deviceChoices = computed(() => Array.from(new Set([...historyDevices.value, ...selectedIds.value])));

const deviceCount = computed(() => selectedIds.value.length);

/** 载入上次推送的设备与原因 */
function useLastPush() {
  const last = lastPush.value;
  if (!last) return message.error("当前固件没有推送记录");
  deviceIdsText.value = last.deviceIds.join(", ");
  if (last.reason) reason.value = last.reason;
  message.success(`已载入上次推送：${last.versionNumber}（${last.deviceIds.length} 台）`);
}

/** 推送升级，成功后记录本次设备 */
async function doPush() {
  if (!store.config) return message.error("请先配置账号信息");
  const ids = selectedIds.value;
  if (ids.length === 0) return message.error("请勾选或填写设备 ID");
  if (!reason.value) return message.error("请填写推送原因");
  pushing.value = true;
  try {
    await tangeAddRule(store.config.tange, versionNumber.value, reason.value, ids);
    const record: PushRecord = {
      firmwareId: store.firmwareId,
      versionNumber: versionNumber.value,
      reason: reason.value,
      deviceIds: ids,
      time: new Date().toLocaleString("zh-CN"),
    };
    const others = (store.config.pushHistory ?? []).filter((item) => item.firmwareId !== store.firmwareId); // 其它固件的记录
    store.config.pushHistory = [record, ...firmwareHistory.value.slice(0, 9), ...others];
    await saveConfig(store.config);
    message.success("推送成功，已记录本次设备");
  } catch (e) {
    message.error(`推送失败：${e}`);
  } finally {
    pushing.value = false;
  }
}
</script>

<template>
  <StepShell title="推送升级" desc="勾选设备后手动确认下发，这一步不会自动推送">
    <template #extra>
      <n-tag :bordered="false" type="info">{{ versionNumber || "未生成版本号" }}</n-tag>
    </template>

    <div class="summary">
      <div class="summary__item">
        <span>目标版本</span>
        <strong>{{ versionNumber || "—" }}</strong>
      </div>
      <div class="summary__item">
        <span>设备数量</span>
        <strong>{{ deviceCount }}</strong>
      </div>
      <div class="summary__item">
        <span>固件 ID</span>
        <strong class="mono">{{ store.firmwareId || "—" }}</strong>
      </div>
    </div>

    <n-form label-placement="top" :show-feedback="false" class="form-stack">
      <n-form-item label="推送原因">
        <n-input v-model:value="reason" placeholder="如：测试文本，顺便改了一些 bug" />
      </n-form-item>
      <n-form-item label="设备 ID">
        <n-input
          v-model:value="deviceIdsText"
          type="textarea"
          placeholder="多个设备 ID 用逗号或换行分隔"
          :rows="4"
        />
      </n-form-item>
    </n-form>

    <section v-if="deviceChoices.length" class="history">
      <header class="history__head">
        <span class="section__title">推送过的设备（{{ historyDevices.length }}）</span>
        <n-button size="small" secondary type="info" :disabled="!lastPush" @click="useLastPush">
          上次推送
        </n-button>
      </header>
      <n-checkbox-group v-model:value="selectedIds">
        <n-checkbox v-for="id in deviceChoices" :key="id" :value="id" :label="id" class="history__item" />
      </n-checkbox-group>
      <p v-if="lastPush" class="history__meta">
        上次推送：{{ lastPush.versionNumber }} · {{ lastPush.time }} · {{ lastPush.deviceIds.length }} 台
      </p>
    </section>

    <template #footer>
      <n-button :disabled="pushing" @click="emit('prev')">上一步</n-button>
      <span class="spacer" />
      <n-button type="primary" :loading="pushing" @click="doPush">推送升级</n-button>
    </template>
  </StepShell>
</template>

<style scoped lang="scss">
.summary {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;

  &__item {
    padding: 12px 14px;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 4px;
    min-width: 0;

    span {
      display: block;
      font-size: 11px;
      color: #64748b;
      margin-bottom: 4px;
    }

    strong {
      display: block;
      font-size: 14px;
      font-weight: 600;
      color: #0f172a;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
  }
}

.history {
  padding: 12px 14px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 4px;

  &__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 10px;

    .section__title {
      margin: 0;
    }
  }

  .n-checkbox-group {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    max-height: 120px;
    overflow: auto;
  }

  &__item {
    font-variant-numeric: tabular-nums;
  }

  &__meta {
    margin: 10px 0 0;
    font-size: 11px;
    color: #64748b;
  }
}

.spacer {
  flex: 1;
}
</style>
