<script setup lang="ts">
import { computed } from "vue";
import { store } from "../store";

/** 从完整路径取出文件名 */
const fileName = computed(() => store.localPath.split(/[\\/]/).pop() || "");

/** 截断过长文本，方便底栏展示 */
function short(text: string, max = 28): string {
  if (!text) return "—";
  return text.length > max ? `${text.slice(0, max)}…` : text;
}
</script>

<template>
  <footer class="pipe">
    <div class="pipe__item" :class="{ 'is-on': !!store.localPath }">
      <span class="pipe__k">本地文件</span>
      <span class="pipe__v" :title="store.localPath">{{ short(fileName) }}</span>
    </div>
    <div class="pipe__item" :class="{ 'is-on': !!store.md5 }">
      <span class="pipe__k">MD5</span>
      <span class="pipe__v mono" :title="store.md5">{{ short(store.md5, 16) }}</span>
    </div>
    <div class="pipe__item" :class="{ 'is-on': !!store.newVersion }">
      <span class="pipe__k">版本</span>
      <span class="pipe__v">{{ store.newVersion || "—" }}</span>
    </div>
    <div class="pipe__item" :class="{ 'is-on': !!store.cdnUrl }">
      <span class="pipe__k">CDN</span>
      <span class="pipe__v" :title="store.cdnUrl">{{ short(store.cdnUrl, 36) }}</span>
    </div>
  </footer>
</template>

<style scoped lang="scss">
.pipe {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 1px;
  background: #1e293b;
  border-top: 1px solid #0f172a;
  flex-shrink: 0;

  &__item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 14px;
    background: #0f172a;
    min-width: 0;

    &.is-on {
      .pipe__k {
        color: #67e8f9;
      }

      .pipe__v {
        color: #f8fafc;
      }
    }
  }

  &__k {
    font-size: 10px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #64748b;
  }

  &__v {
    font-size: 12px;
    color: #94a3b8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
}
</style>
