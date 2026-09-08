<script setup lang="ts">
import { computed, ref } from "vue";
import { NButton, NForm, NFormItem, NInput, NTag, useMessage } from "naive-ui";
import StepShell from "./StepShell.vue";
import { tangeAddRule } from "../api";
import { store } from "../store";

const emit = defineEmits<{ (e: "prev"): void }>();
const message = useMessage();

const reason = ref(""); // 推送原因
const deviceIdsText = ref(""); // 设备 ID，逗号/换行分隔
const pushing = ref(false);

/** 不带 V 前缀的版本号 */
const versionNumber = computed(() => store.newVersion.replace(/^V/i, ""));

/** 解析设备 ID 列表（逗号或换行分隔） */
function parseDeviceIds(): string[] {
  return deviceIdsText.value
    .split(/[,\n，]/)
    .map((s) => s.trim())
    .filter(Boolean);
}

const deviceCount = computed(() => parseDeviceIds().length);

/** 推送升级 */
async function doPush() {
  if (!store.config) return message.error("请先配置账号信息");
  const ids = parseDeviceIds();
  if (ids.length === 0) return message.error("请填写设备 ID");
  if (!reason.value) return message.error("请填写推送原因");
  pushing.value = true;
  try {
    await tangeAddRule(store.config.tange, versionNumber.value, reason.value, ids);
    message.success("推送成功");
  } catch (e) {
    message.error(`推送失败：${e}`);
  } finally {
    pushing.value = false;
  }
}
</script>

<template>
  <StepShell title="推送升级" desc="填写设备 ID 后手动确认下发，这一步不会自动推送">
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
          :rows="5"
        />
      </n-form-item>
    </n-form>

    <template #footer>
      <n-button @click="emit('prev')">上一步</n-button>
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

.spacer {
  flex: 1;
}
</style>
