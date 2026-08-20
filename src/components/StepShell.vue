<script setup lang="ts">
defineProps<{
  title: string;
  desc: string;
}>();
</script>

<template>
  <section class="step-shell">
    <header class="step-shell__head">
      <div>
        <h2>{{ title }}</h2>
        <p>{{ desc }}</p>
      </div>
      <div class="step-shell__extra">
        <slot name="extra" />
      </div>
    </header>
    <div class="step-shell__body">
      <slot />
    </div>
    <footer v-if="$slots.footer" class="step-shell__foot">
      <slot name="footer" />
    </footer>
  </section>
</template>

<style scoped lang="scss">
.step-shell {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  width: 100%;
  background: #fff;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  box-shadow: 0 1px 2px rgba(15, 23, 42, 0.04);

  &__head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
    padding: 18px 20px 14px;
    border-bottom: 1px solid #f1f5f9;

    h2 {
      margin: 0 0 4px;
      font-size: 16px;
      font-weight: 600;
      color: #0f172a;
    }

    p {
      margin: 0;
      font-size: 12px;
      color: #64748b;
      line-height: 1.5;
    }
  }

  &__extra {
    flex-shrink: 0;
  }

  &__body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  &__foot {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 20px;
    border-top: 1px solid #f1f5f9;
    background: #f8fafc;
    border-radius: 0 0 6px 6px;
  }
}
</style>
