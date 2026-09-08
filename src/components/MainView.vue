<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { NButton, NDrawer, NDrawerContent, NTag, useMessage } from "naive-ui";
import ConfigPanel from "./ConfigPanel.vue";
import GenataStep from "./GenataStep.vue";
import PipelineStatus from "./PipelineStatus.vue";
import SftpStep from "./SftpStep.vue";
import TangePublishStep from "./TangePublishStep.vue";
import TangePushStep from "./TangePushStep.vue";
import { loadConfig } from "../api";
import { applyFirmware, store } from "../store";

const message = useMessage();
const showConfig = ref(false); // 配置抽屉显隐
const appVersion = ref(""); // 当前应用版本

/** 步骤定义（四步） */
const steps = [
  { title: "SFTP 上传", desc: "上传固件并计算 MD5" },
  { title: "CDN 登记", desc: "登录并上传到 CDN" },
  { title: "上传 Tange", desc: "登记固件版本到 Tange" },
  { title: "推送升级", desc: "推送设备升级" },
];

/** 切换步骤（自由切换，不限顺序） */
function goStep(step: number) {
  store.current = step;
}

/** 启动时加载本地配置 */
onMounted(async () => {
  try {
    store.config = await loadConfig();
    applyFirmware(store.config.tange.firmwareId);
    appVersion.value = await getVersion();
  } catch (e) {
    message.error(`初始化失败：${e}`);
  }
});
</script>

<template>
  <div class="app-shell">
    <header class="app-shell__header">
      <div class="brand">
        <span class="brand__mark" aria-hidden="true">
          <svg viewBox="0 0 24 24" width="22" height="22" fill="none">
            <path
              d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z"
              stroke="currentColor"
              stroke-width="1.6"
            />
            <path d="M12 8v8M8.5 10.5h7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
          </svg>
        </span>
        <div class="brand__text">
          <strong>固件版本推送</strong>
          <span>OTA 发布流水线</span>
        </div>
      </div>
      <div class="header-meta">
        <n-tag :bordered="false" size="small" class="version-tag">v{{ appVersion || "--" }}</n-tag>
        <n-tag :bordered="false" size="small" class="meta-tag">{{ store.serverDir || "未选固件" }}</n-tag>
        <n-tag :bordered="false" size="small" :type="store.config ? 'success' : 'warning'">
          {{ store.config ? "已配置" : "未配置" }}
        </n-tag>
        <n-button secondary size="small" @click="showConfig = true">账号配置</n-button>
      </div>
    </header>

    <nav class="step-rail">
      <button
        v-for="(s, i) in steps"
        :key="i"
        type="button"
        class="step-rail__item"
        :class="{
          'is-active': store.current === i + 1,
          'is-done': store.current > i + 1,
        }"
        @click="goStep(i + 1)"
      >
        <span class="step-rail__index">{{ String(i + 1).padStart(2, "0") }}</span>
        <span class="step-rail__body">
          <span class="step-rail__title">{{ s.title }}</span>
          <span class="step-rail__desc">{{ s.desc }}</span>
        </span>
      </button>
    </nav>

    <main class="app-shell__content">
      <SftpStep v-show="store.current === 1" @next="goStep(2)" />
      <GenataStep v-show="store.current === 2" @next="goStep(3)" @prev="goStep(1)" />
      <TangePublishStep v-show="store.current === 3" @next="goStep(4)" @prev="goStep(2)" />
      <TangePushStep v-show="store.current === 4" @prev="goStep(3)" />
    </main>

    <PipelineStatus />

    <n-drawer v-model:show="showConfig" :width="520">
      <n-drawer-content title="账号配置" closable>
        <ConfigPanel @saved="showConfig = false" />
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<style scoped lang="scss">
.app-shell {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #e8eef4;

  &__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 56px;
    padding: 0 20px;
    background: #0f172a;
    color: #f8fafc;
    flex-shrink: 0;
  }

  &__content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 12px 20px 16px;
    display: flex;
    flex-direction: column;
  }
}

.brand {
  display: flex;
  align-items: center;
  gap: 10px;

  &__mark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 34px;
    border-radius: 4px;
    background: #155e75;
    color: #a5f3fc;
  }

  &__text {
    display: flex;
    flex-direction: column;
    line-height: 1.2;

    strong {
      font-size: 14px;
      font-weight: 600;
    }

    span {
      font-size: 11px;
      color: #94a3b8;
    }
  }
}

.header-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.meta-tag {
  background: #1e293b !important;
  color: #67e8f9 !important;
}

.version-tag {
  background: #1e293b !important;
  color: #cbd5e1 !important;
  font-variant-numeric: tabular-nums;
}

.step-rail {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
  padding: 12px 20px 0;
  flex-shrink: 0;

  &__item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border: 1px solid #dbe4ee;
    border-radius: 6px;
    background: #fff;
    cursor: pointer;
    text-align: left;
    color: inherit;
    font-family: inherit;
    appearance: none;
    transition: border-color 0.15s ease, box-shadow 0.15s ease, background 0.15s ease;

    &:hover {
      border-color: #67e8f9;
    }

    &.is-active {
      border-color: #0e7490;
      background: #ecfeff;
      box-shadow: 0 0 0 3px rgba(14, 116, 144, 0.12);
    }

    &.is-done:not(.is-active) {
      border-color: #a7f3d0;
      background: #f0fdf4;
    }
  }

  &__index {
    font-size: 15px;
    font-weight: 700;
    color: #94a3b8;
    font-variant-numeric: tabular-nums;

    .is-active & {
      color: #0e7490;
    }

    .is-done:not(.is-active) & {
      color: #059669;
    }
  }

  &__body {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  &__title {
    font-size: 13px;
    font-weight: 600;
    color: #0f172a;
  }

  &__desc {
    font-size: 11px;
    color: #64748b;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}
</style>
