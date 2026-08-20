import type { GlobalThemeOverrides } from "naive-ui";

/** NaiveUI 主题覆盖：青石色工业风，贴合桌面运维工具 */
export const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: "#0e7490",
    primaryColorHover: "#155e75",
    primaryColorPressed: "#164e63",
    primaryColorSuppl: "#0891b2",
    infoColor: "#0284c7",
    successColor: "#059669",
    warningColor: "#d97706",
    errorColor: "#dc2626",
    borderRadius: "4px",
    borderRadiusSmall: "3px",
    fontFamily:
      '"Segoe UI", "PingFang SC", "Microsoft YaHei UI", "Microsoft YaHei", sans-serif',
    fontWeightStrong: "600",
  },
  Button: {
    fontWeight: "500",
    heightMedium: "34px",
    paddingMedium: "0 14px",
    borderRadiusMedium: "4px",
    borderRadiusSmall: "3px",
  },
  Card: {
    borderRadius: "6px",
  },
  Input: {
    heightMedium: "34px",
    borderRadius: "4px",
  },
  InputNumber: {
    borderRadius: "4px",
  },
  Tag: {
    borderRadius: "3px",
  },
  DataTable: {
    thColor: "#f8fafc",
    tdColor: "#ffffff",
    thTextColor: "#475569",
    tdColorHover: "#f0f9ff",
    borderColor: "#e2e8f0",
  },
  Drawer: {
    titleFontSize: "16px",
  },
  Form: {
    labelFontWeight: "500",
    labelTextColor: "#334155",
    feedbackPadding: "4px 0 0 2px",
    labelPaddingVertical: "0 12px 0 0",
  },
};
