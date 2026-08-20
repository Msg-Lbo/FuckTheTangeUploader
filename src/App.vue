<script setup lang="ts">
import { NConfigProvider, NDialogProvider, NGlobalStyle, NMessageProvider, zhCN, dateZhCN } from "naive-ui";
import MainView from "./components/MainView.vue";
import { themeOverrides } from "./theme";
</script>

<template>
  <n-config-provider :locale="zhCN" :date-locale="dateZhCN" :theme-overrides="themeOverrides">
    <n-global-style />
    <n-dialog-provider>
      <n-message-provider>
        <MainView />
      </n-message-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>
